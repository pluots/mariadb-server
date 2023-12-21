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
                    info: ::std::ptr::null_mut(),
                    name: $crate::internals::cstr!($name).as_ptr(),
                    author: $crate::internals::cstr!($author).as_ptr(),
                    descr: $crate::internals::cstr!($description).as_ptr(),
                    license: $license.to_license_registration(),
                    init: None,
                    deinit: None,
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
    };
}
