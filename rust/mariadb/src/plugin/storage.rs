#[macro_export]
macro_rules! register_plugin_storage {
    (
        name:
        $name:literal,author:
        $author:literal,description:
        $description:literal,license:
        $license:expr,maturity:
        $maturity:expr,version:
        $version:literal,handlerton:
        $hton:ty $(,)?
    ) => {
        static STORAGE_ENGINE: $crate::bindings::st_mysql_storage_engine =
            $crate::bindings::st_mysql_storage_engine {
                interface_version: $crate::bindings::MYSQL_HANDLERTON_INTERFACE_VERSION,
            };

        #[no_mangle]
        #[cfg(not(make_static_lib))]
        #[allow(non_upper_case_globals)]
        static _maria_plugin_interface_version_: ::std::ffi::c_int =
            $crate::bindings::MARIA_PLUGIN_INTERFACE_VERSION;

        #[no_mangle]
        #[cfg(not(make_static_lib))]
        #[allow(non_upper_case_globals)]
        static _maria_sizeof_struct_st_plugin_: ::std::ffi::c_int =
            std::mem::size_of::<$crate::bindings::st_maria_plugin>() as ::std::ffi::c_int;

        #[no_mangle]
        #[cfg(not(make_static_lib))]
        #[allow(non_upper_case_globals)]
        static _maria_plugin_declarations_: [$crate::internals::UnsafeSyncCell<
            $crate::bindings::st_maria_plugin,
        >; 2] = unsafe {
            [
                $crate::internals::UnsafeSyncCell::new($crate::bindings::st_maria_plugin {
                    type_: $crate::plugin::PluginType::MyStorageEngine.to_ptype_registration(),
                    info: std::ptr::addr_of!(STORAGE_ENGINE).cast_mut().cast(),
                    name: $crate::internals::cstr!($name).as_ptr(),
                    author: $crate::internals::cstr!($author).as_ptr(),
                    descr: $crate::internals::cstr!($description).as_ptr(),
                    license: $license.to_license_registration(),
                    init: Some($crate::plugin::internals::wrap_storage_init_fn::<$hton>),
                    deinit: Some($crate::plugin::internals::wrap_storage_deinit_fn::<$hton>),
                    version: $crate::internals::parse_version_str($version),
                    status_vars: ::std::ptr::null_mut(),
                    system_vars: ::std::ptr::null_mut(),
                    version_info: $crate::internals::cstr!($version).as_ptr(),
                    maturity: $maturity.to_maturity_registration(),
                }),
                $crate::internals::UnsafeSyncCell::new(
                    $crate::plugin::internals::new_null_plugin_st(),
                ),
            ]
        };

        impl $crate::plugin::internals::PluginMeta for $hton {
            const NAME: &'static str = $name;
        }

        impl $crate::plugin::internals::HandlertonMeta for $hton {
            fn get_vtable() -> &'static $crate::bindings::handler_bridge_vt {
                static VTABLE: $crate::bindings::handler_bridge_vt =
                    $crate::plugin::internals::build_handler_vtable::<$hton>();
                &VTABLE
            }
        }
    };
}
