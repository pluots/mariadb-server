#![allow(unused)]

use std::any::TypeId;
use std::ffi::{c_char, c_int, c_uchar, c_uint, c_ulong, c_ulonglong, c_void, CStr, CString};
use std::sync::Mutex;
use std::{mem, ptr};

use log::info;

use super::wrapper::{init_common, PluginMeta};
use crate::storage::{Handler, Handlerton};
use crate::{bindings, MemRoot, TableShare};

/// Trait implemented by the macro for an easy
pub trait HandlertonMeta: Handlerton + PluginMeta {
    /// This function should return the vtable, which should be located in statics.
    fn get_vtable() -> &'static bindings::handler_bridge_vt;
}

// fn build_cstring(s: &str) -> &'static CStr {
//     static ALL: Mutex<Vec<CString>> = Mutex::new(Vec::new());
//     let cs = CString::try_from(s).unwrap();
//     ALL.lock().unwrap().push(cs);
//     todo!()
// }

/// Initialize the handlerton
pub extern "C" fn wrap_storage_init_fn<P: HandlertonMeta>(hton: *mut c_void) -> c_int {
    /// Wrapper to create a handler from this handlerton
    #[allow(improper_ctypes_definitions)]
    unsafe extern "C" fn create_handler<P: HandlertonMeta>(
        hton: *mut bindings::handlerton,
        table: *mut bindings::TABLE_SHARE,
        mem_root: *mut bindings::MEM_ROOT,
    ) -> *mut bindings::handler {
        let vt = P::get_vtable();
        unsafe { bindings::ha_bridge_construct(hton, table, mem_root, vt) }
    }

    init_common();

    let hton = hton.cast::<bindings::handlerton>();
    unsafe {
        (*hton).create = Some(create_handler::<P>);
        (*hton).flags = P::FLAGS;
        // (*hton).tablefile_extensions =
    }

    log::info!("loaded storage engine {}", P::NAME);
    0
}

/// Deinitialize the handlerton. Nothing to do here for now.
pub extern "C" fn wrap_storage_deinit_fn<P: Handlerton>(hton: *mut c_void) -> c_int {
    0
}

pub const fn build_handler_vtable<H: Handlerton>() -> bindings::handler_bridge_vt {
    bindings::handler_bridge_vt {
        constructor: Some(wrap_constructor::<H::Handler>),
        destructor: Some(wrap_destructor::<H::Handler>),
        index_type: Some(wrap_index_type::<H::Handler>),
        table_flags: Some(wrap_table_flags::<H::Handler>),
        index_flags: Some(wrap_index_flags::<H::Handler>),
        max_supported_record_length: Some(wrap_max_supported_record_length::<H::Handler>),
        max_supported_keys: Some(wrap_max_supported_keys::<H::Handler>),
        max_supported_key_parts: Some(wrap_max_supported_key_parts::<H::Handler>),
        max_supported_key_length: Some(wrap_max_supported_key_length::<H::Handler>),
        scan_time: Some(wrap_scan_time::<H::Handler>),
        keyread_time: Some(wrap_keyread_time::<H::Handler>),
        rnd_pos_time: Some(wrap_rnd_pos_time::<H::Handler>),
        open: Some(wrap_open::<H::Handler>),
        close: Some(wrap_close::<H::Handler>),
        write_row: Some(wrap_write_row::<H::Handler>),
        update_row: Some(wrap_update_row::<H::Handler>),
        delete_row: Some(wrap_delete_row::<H::Handler>),
        index_read_map: Some(wrap_index_read_map::<H::Handler>),
        index_next: Some(wrap_index_next::<H::Handler>),
        index_prev: Some(wrap_index_prev::<H::Handler>),
        index_first: Some(wrap_index_first::<H::Handler>),
        index_last: Some(wrap_index_last::<H::Handler>),
        rnd_init: Some(wrap_rnd_init::<H::Handler>),
        rnd_end: Some(wrap_rnd_end::<H::Handler>),
        rnd_next: Some(wrap_rnd_next::<H::Handler>),
        rnd_pos: Some(wrap_rnd_pos::<H::Handler>),
        position: Some(wrap_position::<H::Handler>),
        info: Some(wrap_info::<H::Handler>),
        extra: Some(wrap_extra::<H::Handler>),
        external_lock: Some(wrap_external_lock::<H::Handler>),
        delete_all_rows: Some(wrap_delete_all_rows::<H::Handler>),
        records_in_range: Some(wrap_records_in_range::<H::Handler>),
        delete_table: Some(wrap_delete_table::<H::Handler>),
        create: Some(wrap_create::<H::Handler>),
        check_if_supported_inplace_alter: Some(wrap_check_if_supported_inplace_alter::<H::Handler>),
        store_lock: Some(wrap_store_lock::<H::Handler>),
    }
}

/// This is the data that we store in the spare pointer of our class
struct HandlerWrapper<H> {
    handler: H,
    type_id: TypeId,
}

unsafe extern "C" fn wrap_constructor<H: Handler>(
    this: *mut bindings::handler_bridge,
    _hton: *mut bindings::handlerton,
    mem_root: *mut bindings::MEM_ROOT,
    table: *mut bindings::TABLE_SHARE,
) {
    let ha_rs = unsafe { H::new(TableShare::from_raw(table), MemRoot::from_raw(mem_root)) };
    unsafe { (*this).data = Box::into_raw(Box::new(ha_rs)).cast() };

    (*this).type_id = mem::transmute(TypeId::of::<H>());
}

unsafe extern "C" fn wrap_destructor<H: Handler>(this: *mut bindings::handler_bridge) {
    // Sanity check we have the expected type
    let tid: TypeId = mem::transmute((*this).type_id);
    debug_assert_eq!(tid, TypeId::of::<H>());

    let ha_rs = unsafe { Box::from_raw((*this).data.cast::<H>()) };
    (*this).data = ptr::null_mut();

    drop(ha_rs)
}

unsafe extern "C" fn wrap_index_type<H: Handler>(
    this: *mut bindings::handler_bridge,
    arg1: c_uint,
) -> *const c_char {
    todo!()
}

unsafe extern "C" fn wrap_table_flags<H: Handler>(
    this: *const bindings::handler_bridge,
) -> c_ulonglong {
    todo!()
}

unsafe extern "C" fn wrap_index_flags<H: Handler>(
    this: *const bindings::handler_bridge,
    arg1: c_uint,
    arg2: c_uint,
    arg3: bool,
) -> c_ulong {
    todo!()
}

unsafe extern "C" fn wrap_max_supported_record_length<H: Handler>(
    this: *const bindings::handler_bridge,
) -> c_uint {
    todo!()
}

unsafe extern "C" fn wrap_max_supported_keys<H: Handler>(
    this: *const bindings::handler_bridge,
) -> c_uint {
    todo!()
}

unsafe extern "C" fn wrap_max_supported_key_parts<H: Handler>(
    this: *const bindings::handler_bridge,
) -> c_uint {
    todo!()
}

unsafe extern "C" fn wrap_max_supported_key_length<H: Handler>(
    this: *const bindings::handler_bridge,
) -> c_uint {
    todo!()
}

unsafe extern "C" fn wrap_scan_time<H: Handler>(
    arg1: *mut bindings::handler_bridge,
) -> bindings::IO_AND_CPU_COST {
    todo!()
}

unsafe extern "C" fn wrap_keyread_time<H: Handler>(
    this: *mut bindings::handler_bridge,
    arg2: c_uint,
    arg3: c_ulong,
    arg4: bindings::ha_rows,
    arg5: c_ulonglong,
) -> bindings::IO_AND_CPU_COST {
    todo!()
}

unsafe extern "C" fn wrap_rnd_pos_time<H: Handler>(
    this: *mut bindings::handler_bridge,
    arg2: bindings::ha_rows,
) -> bindings::IO_AND_CPU_COST {
    todo!()
}

unsafe extern "C" fn wrap_open<H: Handler>(
    this: *mut bindings::handler_bridge,
    arg2: *const c_char,
    arg3: c_int,
    arg4: c_uint,
) -> c_int {
    todo!()
}

unsafe extern "C" fn wrap_close<H: Handler>(arg1: *mut bindings::handler_bridge) -> c_int {
    todo!()
}

unsafe extern "C" fn wrap_write_row<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: *const c_uchar,
) -> c_int {
    todo!()
}

unsafe extern "C" fn wrap_update_row<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: *const c_uchar,
    arg3: *const c_uchar,
) -> c_int {
    todo!()
}

unsafe extern "C" fn wrap_delete_row<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: *const c_uchar,
) -> c_int {
    todo!()
}

unsafe extern "C" fn wrap_index_read_map<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: *mut c_uchar,
    arg3: *const c_uchar,
    arg4: bindings::key_part_map,
    arg5: bindings::ha_rkey_function::Type,
) -> c_int {
    todo!()
}

unsafe extern "C" fn wrap_index_next<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: *mut c_uchar,
) -> c_int {
    todo!()
}

unsafe extern "C" fn wrap_index_prev<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: *mut c_uchar,
) -> c_int {
    todo!()
}

unsafe extern "C" fn wrap_index_first<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: *mut c_uchar,
) -> c_int {
    todo!()
}

unsafe extern "C" fn wrap_index_last<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: *mut c_uchar,
) -> c_int {
    todo!()
}

unsafe extern "C" fn wrap_rnd_init<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: bool,
) -> c_int {
    todo!()
}

unsafe extern "C" fn wrap_rnd_end<H: Handler>(arg1: *mut bindings::handler_bridge) -> c_int {
    todo!()
}

unsafe extern "C" fn wrap_rnd_next<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: *mut c_uchar,
) -> c_int {
    todo!()
}

unsafe extern "C" fn wrap_rnd_pos<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: *mut c_uchar,
    arg3: *mut c_uchar,
) -> c_int {
    todo!()
}
unsafe extern "C" fn wrap_position<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: *const c_uchar,
) {
    todo!()
}

unsafe extern "C" fn wrap_info<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: c_uint,
) -> c_int {
    todo!()
}

unsafe extern "C" fn wrap_extra<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: bindings::ha_extra_function::Type,
) -> c_int {
    todo!()
}

unsafe extern "C" fn wrap_external_lock<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: *mut bindings::THD,
    arg3: c_int,
) -> c_int {
    todo!()
}

unsafe extern "C" fn wrap_delete_all_rows<H: Handler>(
    arg1: *mut bindings::handler_bridge,
) -> c_int {
    todo!()
}

unsafe extern "C" fn wrap_records_in_range<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: c_uint,
    arg3: *const bindings::key_range,
    arg4: *const bindings::key_range,
    arg5: *mut bindings::page_range,
) -> bindings::ha_rows {
    todo!()
}

unsafe extern "C" fn wrap_delete_table<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: *const c_char,
) -> c_int {
    todo!()
}

pub unsafe extern "C" fn wrap_create<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: *const c_char,
    arg3: *mut bindings::TABLE,
    arg4: *mut bindings::HA_CREATE_INFO,
) -> c_int {
    todo!()
}

pub unsafe extern "C" fn wrap_check_if_supported_inplace_alter<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: *mut bindings::TABLE,
    arg3: *mut bindings::Alter_inplace_info,
) -> bindings::enum_alter_inplace_result::Type {
    todo!()
}

pub unsafe extern "C" fn wrap_store_lock<H: Handler>(
    arg1: *mut bindings::handler_bridge,
    arg2: *mut bindings::THD,
    arg3: *mut *mut bindings::THR_LOCK_DATA,
    arg4: bindings::thr_lock_type::Type,
) -> *mut *mut bindings::THR_LOCK_DATA {
    todo!()
}
