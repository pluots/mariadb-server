use std::ffi::{c_int, c_void};
use std::{env, ptr};

use super::{Init, InitError};
use crate::{bindings, configure_logger};

/// Meta that we generate in the proc macro, which we can use to get information about our type in
/// wrappers
pub trait PluginMeta {
    const NAME: &'static str;
}

/// Wrap the init call
#[must_use]
pub unsafe extern "C" fn wrap_init_fn<P: PluginMeta, I: Init>(_: *mut c_void) -> c_int {
    init_common();
    match I::init() {
        Ok(()) => {
            log::info!("loaded plugin {}", P::NAME);
            0
        }
        Err(InitError) => {
            log::error!("failed to load plugin {}", P::NAME);
            1
        }
    }
}

/// Wrap the deinit call
#[must_use]
pub unsafe extern "C" fn wrap_deinit_fn<P: PluginMeta, I: Init>(_: *mut c_void) -> c_int {
    match I::deinit() {
        Ok(()) => {
            log::info!("unloaded plugin {}", P::NAME);
            0
        }
        Err(InitError) => {
            log::error!("failed to unload plugin {}", P::NAME);
            1
        }
    }
}

/// Init call for plugins that don't provide a custom init function
#[must_use]
pub unsafe extern "C" fn default_init_notype<P: PluginMeta>(_: *mut c_void) -> c_int {
    init_common();
    log::info!("loaded plugin {}", P::NAME);
    0
}

#[must_use]
pub unsafe extern "C" fn default_deinit_notype<P: PluginMeta>(_: *mut c_void) -> c_int {
    log::info!("unloaded plugin {}", P::NAME);
    0
}

/// What to run when every plugin is loaded.
pub(super) fn init_common() {
    configure_logger!();
    env::set_var("RUST_BACKTRACE", "1");
}

/// New struct with all null values
#[must_use]
#[doc(hidden)]
pub const fn new_null_plugin_st() -> bindings::st_maria_plugin {
    bindings::st_maria_plugin {
        type_: 0,
        info: ptr::null_mut(),
        name: ptr::null(),
        author: ptr::null(),
        descr: ptr::null(),
        license: 0,
        init: None,
        deinit: None,
        version: 0,
        status_vars: ptr::null_mut(),
        system_vars: ptr::null_mut(),
        version_info: ptr::null(),
        maturity: 0,
    }
}
