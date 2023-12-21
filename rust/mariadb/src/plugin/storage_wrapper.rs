use std::ffi::{c_int, c_void};

use super::wrapper::{init_common, PluginMeta};
use crate::storage::Handlerton;

pub extern "C" fn wrap_storage_init_fn<P: Handlerton>(hton: *mut c_void) -> c_int {
    init_common();
    todo!()
}

pub extern "C" fn wrap_storage_deinit_fn<P: Handlerton>(hton: *mut c_void) -> c_int {
    todo!()
}
