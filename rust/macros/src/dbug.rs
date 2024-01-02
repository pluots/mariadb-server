use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// This moves the function body to an inner function, then wraps a call to the
/// inner function with equivalent to MariaDB's `DBUG_ENTER` and `DBUG_RETURN`.
///
/// This only happens if the rust flag `--cfg dbug_trace` is passed.
pub fn instrument(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    if let Some(tt) = attr.into_iter().next() {
        return syn::Error::new(tt.span().into(), "No attribute arguments expected")
            .into_compile_error()
            .into();
    }

    let ItemFn {
        ref attrs,
        ref vis,
        ref sig,
        ref block,
    } = input;

    let inner_ident = Ident::new("inner", Span::call_site());
    let mut inner_sig = sig.clone();
    inner_sig.ident = inner_ident;

    let _crate_name = if std::env::var("CARGO_CRATE_NAME").unwrap() == "mariadb" {
        "crate"
    } else {
        "::mariadb"
    };

    quote! {
        // #[cfg(not(dbug_trace))]
        // #input

        // #[cfg(dbug_trace)]
        #(#attrs)*
        #vis #sig {
            #inner_sig #block

            // let name = #crate_name :: internal :: cstr!(module_path!());
            let x = std :: file!();
            // let file = #crate_name::internal::cstr!(file!());
            // let line = line!();
            // let frame = std::mem::MaybeUninit<::mariadb::bindings::_db_stack_frame_>::zeroed();
            // ::mariadb::bindings::_db_enter_(name, file.as_ptr(), line frame.as_mut_ptr());

            let ret = inner();

            // ::mariadb::bindings::_db_return_(frame.as_mut_ptr());

            ret
        }
    }
    .into()
}
