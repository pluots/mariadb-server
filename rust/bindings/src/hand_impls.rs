use std::ffi::{c_char, c_double, c_int, c_long, c_longlong, c_uint, c_ulong, c_ulonglong};

use super::{mysql_var_check_func, mysql_var_update_func, TYPELIB};

// Defined in service_encryption.h but not imported because of tilde syntax
pub const ENCRYPTION_KEY_VERSION_INVALID: c_uint = !0;

#[allow(dead_code)] // not sure why this lint hits
pub const PLUGIN_VAR_MASK: u32 = super::PLUGIN_VAR_READONLY
    | super::PLUGIN_VAR_NOSYSVAR
    | super::PLUGIN_VAR_NOCMDOPT
    | super::PLUGIN_VAR_NOCMDARG
    | super::PLUGIN_VAR_OPCMDARG
    | super::PLUGIN_VAR_RQCMDARG
    | super::PLUGIN_VAR_DEPRECATED
    | super::PLUGIN_VAR_MEMALLOC;

/// Helper macro to create structs we use for sysvars
///
/// This is kind of a Rust version of the `DECLARE_MYSQL_SYSVAR_X` macros. We reimplement instead
/// of using the C version because the definition is tricky, not all fields are always present.
/// No support for THD yet.
macro_rules! declare_sysvar_type {
    (
        @common
        $name:ident: $(
            #[$field_doc:meta]
            $field_name:ident: $field_ty:ty
        ),*; $struct_doc:literal
    ) => {
        // Common implementation
        #[doc = $struct_doc]
        #[repr(C)]
        #[derive(Debug)]
        pub struct $name {
            /// Variable flags
            pub flags: c_int,
            /// Name of the variable
            pub name: *const c_char,
            /// Variable description
            pub comment: *const c_char,
            /// Function for getting the variable
            pub check: mysql_var_check_func,
            /// Function for setting the variable
            pub update: mysql_var_update_func,

            // Repeated fields
            $(
                #[$field_doc]
                pub $field_name: $field_ty
            ),*
        }
    };

    (basic: $name:ident, $c_ty:ty; $doc:literal) => {
        // A "basic" sysvar
        declare_sysvar_type!{
            @common $name:
            #[doc = "Pointer to the value"]
            value: *mut $c_ty,
            #[doc = "Default value"]
            def_val: $c_ty;
            $doc
        }
    };

    (const basic: $name:ident, $c_ty:ty; $doc:literal) => {
        // A "basic" sysvar
        declare_sysvar_type!{
            @common $name:
            #[doc = "Pointer to the value"]
            value: *const $c_ty,
            #[doc = "Default value"]
            def_val: $c_ty;
            $doc
        }
    };

    (simple: $name:ident, $c_ty:ty; $doc:literal) => {
        // A "simple" sysvar, with minimum maximum and block size
        declare_sysvar_type!{
            @common $name:
            #[doc = "Pointer to the value"]
            value: *mut $c_ty,
            #[doc = "Default value"]
            def_val: $c_ty,
            #[doc = "Minimum value"]
            min_val: $c_ty,
            #[doc = "Maximum value"]
            max_val: $c_ty,
            #[doc = "Block size"]
            blk_sz: $c_ty;
            $doc
        }
    };

    (typelib: $name:ident, $c_ty:ty; $doc:literal) => {
        // A "typelib" sysvar
        declare_sysvar_type!{
            @common $name:
            #[doc = "Pointer to the value"]
            value: *mut $c_ty,
            #[doc = "Default value"]
            def_val: $c_ty,
            #[doc = "Typelib"]
            typelib: *const TYPELIB;
            $doc
        }
    };

    // (typelib: $name:ident, $ty:ty) => {

    // };
    // (thd: $name:ident, $ty:ty) => {

    // };
}

declare_sysvar_type!(@common sysvar_common_t:; "");
declare_sysvar_type!(basic: sysvar_bool_t, bool; "");
declare_sysvar_type!(basic: sysvar_str_t, *mut c_char; "");
declare_sysvar_type!(typelib: sysvar_enum_t, c_ulong; "");
declare_sysvar_type!(typelib: sysvar_set_t, c_ulonglong; "");
declare_sysvar_type!(simple: sysvar_int_t, c_int; "Sysvar that stores an int");
declare_sysvar_type!(simple: sysvar_long_t, c_long; "Sysvar that stores a long");
declare_sysvar_type!(simple: sysvar_longlong_t, c_longlong; "Sysvar that stores a longlong");
declare_sysvar_type!(simple: sysvar_uint_t, c_uint; "Sysvar that stores a uint");
declare_sysvar_type!(simple: sysvar_ulong_t, c_ulong; "Sysvar that stores a ulong");
declare_sysvar_type!(simple: sysvar_ulonglong_t, c_ulonglong; "Sysvar that stores a ulonglong");
declare_sysvar_type!(simple: sysvar_double_t, c_double; "Sysvar that stores a double");

// declare_sysvar_type!(thdbasic: thdvar_bool_t, bool);
// declare_sysvar_type!(thdbasic: thdvar_str_t, *mut c_char);
// declare_sysvar_type!(typelib: sysvar_enum_t, c_ulong);
// declare_sysvar_type!(typelib: sysvar_set_t, c_ulonglong);

// type THDVAR_FUNC<T> = Option<unsafe extern "C" fn(thd: *const c_void, offset: c_int) -> *mut T>;
