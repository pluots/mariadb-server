//! Macro to register a plugin

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Error, Expr, FieldValue, Ident, Token};

use crate::fields::plugin::{ALL_FIELDS, ALWAYS_REQ_FIELDS, ENCR_OPT_FIELDS, ENCR_REQ_FIELDS};
use crate::helpers::{expect_bool, expect_litstr, expect_ty, make_ident};
use crate::parse_vars::Variables;

/// Entrypoint for this proc macro
pub fn entry(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as PluginInfo);
    let plugindef = input.into_encryption_struct();
    match plugindef {
        Ok(ts) => ts.into_output().into(),
        Err(e) => e.into_compile_error().into(),
    }
}

/// A representation of the contents of a registration macro. This macro will be
/// the same for
#[derive(Clone, Debug)]
struct PluginInfo {
    /// The main type that has required methods implemented on it
    main_ty: Ident,
    ptype: Option<Expr>,
    name: Option<Expr>,
    author: Option<Expr>,
    description: Option<Expr>,
    license: Option<Expr>,
    maturity: Option<Expr>,
    version: Option<Expr>,
    init: Option<Expr>,
    encryption: Option<Expr>,
    decryption: Option<Expr>,
    variables: Option<Expr>,
}

impl Parse for PluginInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let main_ty = input.parse()?;
        let mut ret = Self::new(main_ty);
        let _: Token![,] = input.parse()?;

        let fields = Punctuated::<FieldValue, Token![,]>::parse_terminated(input)?;
        let mut field_order: Vec<String> = Vec::new();
        for field in fields.clone() {
            let syn::Member::Named(name) = &field.member else {
                return Err(Error::new_spanned(field, "missing field name"));
            };

            let name_str = name.to_string();
            let expr = field.expr;

            match name_str.as_str() {
                "ptype" => ret.ptype = Some(expr),
                "name" => ret.name = Some(expr),
                "author" => ret.author = Some(expr),
                "description" => ret.description = Some(expr),
                "license" => ret.license = Some(expr),
                "maturity" => ret.maturity = Some(expr),
                "version" => ret.version = Some(expr),
                "init" => ret.init = Some(expr),
                "encryption" => ret.encryption = Some(expr),
                "decryption" => ret.decryption = Some(expr),
                "variables" => ret.variables = Some(expr),
                _ => {
                    return Err(Error::new_spanned(
                        name,
                        format!("unexpected field '{name_str}'"),
                    ))
                }
            }
            field_order.push(name_str);
        }

        if let Err(msg) = verify_field_order(field_order.as_slice()) {
            return Err(Error::new_spanned(fields, msg));
        }
        Ok(ret)
    }
}

impl PluginInfo {
    const fn new(main_ty: Ident) -> Self {
        Self {
            main_ty,
            ptype: None,
            name: None,
            author: None,
            description: None,
            license: None,
            maturity: None,
            version: None,
            init: None,
            encryption: None,
            decryption: None,
            variables: None,
        }
    }

    /// Ensure we have the fields that are required for all plugin types
    fn validate_correct_fields(
        &self,
        required: &[&str],
        optional: &[&str],
        ptype: &str,
    ) -> syn::Result<()> {
        // These are all required for all plugin types
        let name_map = [
            (&self.ptype, "ptype"),
            (&self.name, "name"),
            (&self.author, "author"),
            (&self.description, "description"),
            (&self.license, "license"),
            (&self.maturity, "maturity"),
            (&self.version, "version"),
            (&self.init, "init"),
            (&self.encryption, "encryption"),
            (&self.decryption, "decryption"),
            (&self.variables, "sysvars"),
        ];

        let mut req = ALWAYS_REQ_FIELDS.to_vec();
        req.extend_from_slice(required);

        for req_field in &req {
            let (field_val, fname) = name_map.iter().find(|f| f.1 == *req_field).unwrap();

            if field_val.is_none() {
                let msg = format!("field '{fname}' is expected for {ptype} plugins, but not provided\n(in macro 'register_plugin')");
                return Err(Error::new(Span::call_site(), msg));
            }
        }

        for (field, fname) in name_map {
            if field.is_some() && !req.contains(&fname) && !optional.contains(&fname) {
                let msg = format!("field '{fname}' is not expected for {ptype} plugins\n(in macro 'register_plugin')");
                return Err(Error::new_spanned(field.as_ref().unwrap(), msg));
            }
        }

        Ok(())
    }

    /// Validate the sysvars definition and create structure. Returns
    fn make_variables(&self) -> syn::Result<VariableBodies> {
        let mut ret = VariableBodies {
            sysvar_body: TokenStream::new(),
            sysvar_field: quote! { ::std::ptr::null_mut() },
        };
        let Some(vars_decl) = &self.variables else {
            return Ok(ret);
        };
        let vars: Variables = syn::parse(vars_decl.to_token_stream().into())?;
        let name = expect_litstr(self.name.as_ref())?.value();
        let sysvar_arr_ident = Ident::new(&format!("_plugin_{name}_sysvars"), Span::call_site());

        let sysvar_bodies = &vars.sys;
        let sysvar_idents = &vars.sys_idents;
        assert_eq!(sysvar_bodies.len(), sysvar_idents.len());

        if sysvar_bodies.is_empty() {
            return Ok(ret);
        }

        let len = sysvar_bodies.len() + 1;
        let usynccell = quote! { ::mariadb::internals::UnsafeSyncCell };

        ret.sysvar_body = quote! {
            #( #sysvar_bodies )*

            #[allow(non_upper_case_globals)]
            pub static #sysvar_arr_ident:
                [#usynccell<*mut ::mariadb::bindings::sysvar_common_t>; #len]
                =
                unsafe { [
                    #( #usynccell::new(#sysvar_idents.get().cast()), )*
                    #usynccell::new(::std::ptr::null_mut())
                ] };
        };
        // panic!("{}", ret.sysvar_body);
        ret.sysvar_field = quote! { #sysvar_arr_ident.as_ptr().cast_mut().cast() };
        Ok(ret)
    }

    /// Ensure we have the fields required for an encryption plugin
    fn validate_as_encryption(&self) -> syn::Result<()> {
        self.validate_correct_fields(ENCR_REQ_FIELDS, ENCR_OPT_FIELDS, "encryption")?;
        Ok(())
    }

    /// Turn `self` into a tokenstream of a single `st_maria_plugin` for an
    /// encryption struct. Uses `idx` to mangle the name and avoid conflicts
    #[allow(clippy::too_many_lines)]
    fn into_encryption_struct(self) -> syn::Result<PluginDef> {
        self.validate_as_encryption()?;

        let main_ty = &self.main_ty;
        let name = expect_litstr(self.name.as_ref())?;
        let plugin_st_name = Ident::new(&format!("_ST_PLUGIN_{}", name.value()), Span::call_site());

        let ty_as_wkeymgt = quote! { <#main_ty as ::mariadb::plugin::internals::WrapKeyMgr> };
        let interface_version = quote! { ::mariadb::bindings::MariaDB_ENCRYPTION_INTERFACE_VERSION as ::std::ffi::c_int };
        let get_key_vers = quote! { Some(#ty_as_wkeymgt::wrap_get_latest_key_version) };
        let get_key = quote! { Some(#ty_as_wkeymgt::wrap_get_key) };
        let variables = self.make_variables()?;
        let variable_body = variables.sysvar_body;

        let meta_impl = quote! {
            impl ::mariadb::plugin::internals::PluginMeta for #main_ty {
                const NAME: &'static str = #name;
            }
        };

        let mut enc_dec_types = None;

        if let Some(encr_value) = self.encryption {
            // expect_bool(&self.encryption)? {
            let encr_bool_res = expect_bool(Some(&encr_value));
            if encr_bool_res.is_ok() {
                assert!(
                    self.decryption.is_none(),
                    "cannot specify decryption type but not encryption"
                );
            };

            if matches!(encr_bool_res, Ok(true)) {
                // Use encryption on the main type if specified
                let main_tokens = main_ty.to_token_stream();
                enc_dec_types = Some((main_tokens.clone(), main_tokens));
            } else if encr_bool_res.is_err() {
                let enc_ty = expect_ty(&encr_value)?;
                let dec_ty = match self.decryption {
                    Some(ref v) => expect_ty(v)?,
                    None => enc_ty,
                };
                enc_dec_types = Some((enc_ty.to_token_stream(), dec_ty.to_token_stream()));
            }
        }

        let (crypt_size, crypt_init, crypt_update, crypt_finish, crypt_len);
        if let Some((enc_ty, dec_ty)) = enc_dec_types {
            crypt_size = quote! { Some(::mariadb::plugin::internals::wrap_crypt_ctx_size::<#enc_ty, #dec_ty>) };
            crypt_init = quote! { Some(::mariadb::plugin::internals::wrap_crypt_ctx_init::<#enc_ty, #dec_ty>) };
            crypt_update = quote! { Some(::mariadb::plugin::internals::wrap_crypt_ctx_update::<#enc_ty, #dec_ty>) };
            crypt_finish = quote! { Some(::mariadb::plugin::internals::wrap_crypt_ctx_finish::<#enc_ty, #dec_ty>) };
            crypt_len =
                quote! { Some(::mariadb::plugin::internals::wrap_encrypted_length::<#enc_ty>) };
        } else {
            // Default to builtin encryption
            let none = quote! { None };
            crypt_size = none.clone();
            crypt_init = none.clone();
            crypt_update = none.clone();
            crypt_finish = none.clone();
            crypt_len = none;
        }

        let info_struct = quote! {
            #[allow(non_upper_case_globals)]
            static #plugin_st_name: ::mariadb::internals::UnsafeSyncCell<
                ::mariadb::bindings::st_mariadb_encryption,
            > = unsafe {
                ::mariadb::internals::UnsafeSyncCell::new(
                    ::mariadb::bindings::st_mariadb_encryption {
                        interface_version: #interface_version,
                        get_latest_key_version: #get_key_vers,
                        get_key: #get_key,
                        crypt_ctx_size: #crypt_size,
                        crypt_ctx_init: #crypt_init,
                        crypt_ctx_update: #crypt_update,
                        crypt_ctx_finish: #crypt_finish,
                        encrypted_length: #crypt_len,
                    }
                )
            };
        };

        let version_str = &expect_litstr(self.version.as_ref())?.value();
        let version_int =
            version_int(version_str).map_err(|e| Error::new_spanned(&self.version, e))?;
        let author = expect_litstr(self.author.as_ref())?;
        let description = expect_litstr(self.description.as_ref())?;
        let license = self.license.unwrap();
        let maturity = self.maturity.unwrap();
        let ptype = self.ptype.unwrap();
        let system_vars_ptr = variables.sysvar_field;

        // We always initialize the logger, maybe do init/deinit if struct requires
        let (fn_deinit, fn_init);
        if let Some(init_ty) = self.init {
            fn_init =
                quote! { Some(::mariadb::plugin::internals::wrap_init_fn::<#main_ty, #init_ty>) };
            fn_deinit =
                quote! { Some(::mariadb::plugin::internals::wrap_deinit_fn::<#main_ty, #init_ty>) };
        } else {
            fn_init =
                quote! { Some(::mariadb::plugin::internals::default_init_notype::<#main_ty>) };
            fn_deinit =
                quote! { Some(::mariadb::plugin::internals::default_deinit_notype::<#main_ty>) };
        }

        let plugin_struct = quote! {
            ::mariadb::bindings::st_maria_plugin {
                type_: #ptype.to_ptype_registration(),
                info: #plugin_st_name.as_ptr().cast_mut().cast(),
                name: ::mariadb::internals::cstr!(#name).as_ptr(),
                author: ::mariadb::internals::cstr!(#author).as_ptr(),
                descr: ::mariadb::internals::cstr!(#description).as_ptr(),
                license: #license.to_license_registration(),
                init: #fn_init,
                deinit: #fn_deinit,
                version: #version_int as ::std::ffi::c_uint,
                status_vars: ::std::ptr::null_mut(),
                system_vars: #system_vars_ptr,
                version_info: ::mariadb::internals::cstr!(#version_str).as_ptr(),
                maturity: #maturity.to_maturity_registration(),
            },
        };

        Ok(PluginDef {
            name: name.value(),
            meta_impl,
            info_struct,
            plugin_struct,
            variable_body,
        })
    }
}

struct VariableBodies {
    /// This body will be added
    sysvar_body: TokenStream,
    /// What to put in the registering `st_mariadb_plugin`
    sysvar_field: TokenStream,
}

/// Contains a struct definition of type `st_mariadb_encryption` or whatever is
/// applicable, plus the struct of `st_maria_plugin` that references it
struct PluginDef {
    name: String,
    meta_impl: TokenStream,
    info_struct: TokenStream,
    plugin_struct: TokenStream,
    variable_body: TokenStream,
}

impl PluginDef {
    fn into_output(self) -> TokenStream {
        // static and dynamic identifiers
        let vers_ident_stc = make_ident(&format!("builtin_{}_plugin_interface_version", self.name));
        let vers_ident_dyn = make_ident("_maria_plugin_interface_version_");
        let size_ident_stc = make_ident(&format!("builtin_{}_sizeof_struct_st_plugin", self.name));
        let size_ident_dyn = make_ident("_maria_sizeof_struct_st_plugin_");
        let decl_ident_stc = make_ident(&format!("builtin_{}_plugin", self.name));
        let decl_ident_dyn = make_ident("_maria_plugin_declarations_");

        let plugin_ty = quote! { ::mariadb::bindings::st_maria_plugin };
        let version_val =
            quote! { mariadb::bindings::MARIA_PLUGIN_INTERFACE_VERSION as ::std::ffi::c_int };
        let size_val = quote! { ::std::mem::size_of::<#plugin_ty>() as ::std::ffi::c_int };

        let usynccell = quote! { ::mariadb::internals::UnsafeSyncCell };
        let null_ps = quote! { ::mariadb::plugin::internals::new_null_plugin_st() };

        let info_st = self.info_struct;
        let meta_impl = self.meta_impl;
        let plugin_st = self.plugin_struct;
        let variable_body = self.variable_body;

        quote! { ::std::ptr::null_mut() };

        let ret: TokenStream = quote! {
            #info_st
            #meta_impl
            #variable_body

            // Different config based on statically or dynamically lynked
            #[no_mangle]
            #[cfg(make_static_lib)]
            #[allow(non_upper_case_globals)]
            static #vers_ident_stc: ::std::ffi::c_int = #version_val;

            #[no_mangle]
            #[cfg(not(make_static_lib))]
            #[allow(non_upper_case_globals)]
            static #vers_ident_dyn: ::std::ffi::c_int = #version_val;

            #[no_mangle]
            #[cfg(make_static_lib)]
            #[allow(non_upper_case_globals)]
            static #size_ident_stc: ::std::ffi::c_int = #size_val;

            #[no_mangle]
            #[cfg(not(make_static_lib))]
            #[allow(non_upper_case_globals)]
            static #size_ident_dyn: ::std::ffi::c_int = #size_val;

            #[no_mangle]
            #[cfg(make_static_lib)]
            #[allow(non_upper_case_globals)]
            static #decl_ident_stc: [#usynccell<#plugin_ty>; 2] = unsafe { [
                #usynccell::new(#plugin_st),
                #usynccell::new(#null_ps),
            ] };

            #[no_mangle]
            #[cfg(not(make_static_lib))]
            #[allow(non_upper_case_globals)]
            static #decl_ident_dyn: [#usynccell<#plugin_ty>; 2] = unsafe { [
                #usynccell::new(#plugin_st),
                #usynccell::new(#null_ps),
            ] };
        };
        ret
    }
}

/// Verify attribute order
fn verify_field_order(fields: &[String]) -> Result<(), String> {
    let mut expected_order = ALL_FIELDS.to_vec();

    expected_order.retain(|expected| fields.iter().any(|f| f == expected));

    if expected_order == fields {
        return Ok(());
    }

    Err(format!(
        "fields not in expected order. reorder as:\n{expected_order:?}",
    ))
}

/// Convert a string like "1.2" to a hex like "0x0102". Error if no decimal, or
/// if either value exceeds a u8.
fn version_int(s: &str) -> Result<u16, String> {
    const USAGE_MSG: &str = r#"expected a two position semvar string, e.g. "1.2""#;
    if s.chars().filter(|x| *x == '.').count() != 1 {
        return Err(USAGE_MSG.to_owned());
    }

    let splt = s.split_once('.').unwrap();
    let fmt_err = |e| format!("{e}\n{USAGE_MSG}");

    let major: u16 = splt.0.parse::<u8>().map_err(fmt_err)?.into();
    let minor: u16 = splt.1.parse::<u8>().map_err(fmt_err)?.into();
    let res: u16 = (major << 8) + minor;

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_int() {
        assert_eq!(version_int("5.2"), Ok(0x0502));
        assert_eq!(version_int("11.0"), Ok(0x0b00));
    }
}
