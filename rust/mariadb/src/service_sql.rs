//! Safe API for `include/mysql/service_sql.h`

#![allow(unused)]
mod error;

use std::cell::{OnceCell, UnsafeCell};
use std::ffi::{c_char, c_longlong, c_ulonglong, CStr, CString};
use std::marker::PhantomData;
use std::mem::transmute;
use std::ptr::{self, NonNull};
use std::sync::Once;
use std::{fmt, mem, slice, str};

use log::{debug, error, trace};

pub use self::error::ClientError;
use crate::util::UnsafeSyncCell;
use crate::{bindings, Value};

/// Type wrapper for `Result` with a `ClientError` error variant
pub type ClientResult<T> = Result<T, ClientError>;

// HACK: we need to provide a symbols that is the version defined in service_versions.h
// On load, the symbol gets replaced with the real thing.
// The C plugins work around this with defines, but I couldn't find a good way to
// get similar results through bindgen.
//
// FIXME: I think this is different for static linking but I'm not sure how
//
// This all gets loaded in sql_plugin.cc `plugin_dl_add`
#[no_mangle]
#[cfg(not(make_static_lib))]
#[allow(non_upper_case_globals)]
pub static sql_service: UnsafeSyncCell<*mut bindings::sql_service_st> =
    unsafe { UnsafeSyncCell::new(0x0101 as _) };

/// Get a function from our global SQL service
macro_rules! global_func {
    ($fname:ident) => {{
        log::debug!("calling global function {}", stringify!($fname));
        unsafe { (**sql_service.get()).$fname.unwrap() }
    }};
}

/// A connection to a local or remote SQL server
pub struct Connection {
    /// INVARIANT: must always be valid
    inner: NonNull<bindings::MYSQL>,
}

impl Connection {
    /// Connect to the local server
    ///
    /// # Errors
    ///
    /// Error if the client could not connect
    pub fn connect_local() -> ClientResult<Self> {
        log::debug!("connecting to the local server");
        let mut this = Self::mysql_init()?;

        let res = unsafe { global_func!(mysql_real_connect_local_func)(this.inner.as_ptr()) };
        this.check_for_errors(ClientError::ConnectError)?;
        if res.is_null() {
            let msg = "connect error, are you already connected?".into();
            Err(ClientError::ConnectError(0, msg))
        } else {
            Ok(this)
        }
    }

    /// Connect to a remote server
    pub fn connect(&mut self, conn_opts: &ConnectionOpts) -> ClientResult<Self> {
        log::debug!("connecting to the remote server");
        let this = Self::mysql_init()?;

        let host = conn_opts.host.as_ref();
        let user = conn_opts.user.as_ref();
        let pw = conn_opts.password.as_ref();
        let db = conn_opts.database.as_ref();
        let port = conn_opts.port;
        let sock = conn_opts.unix_socket.as_ref();

        // TODO: see CLIENT_X flags in mariadb_com.h
        let res = unsafe {
            // Make sure you don't use the fake one!
            global_func!(mysql_real_connect_func)(
                self.inner.as_ptr(),
                host.map_or(ptr::null(), |c| c.as_ptr()),
                user.map_or(ptr::null(), |c| c.as_ptr()),
                pw.map_or(ptr::null(), |c| c.as_ptr()),
                db.map_or(ptr::null(), |c| c.as_ptr()),
                port.map_or(0, std::convert::Into::into),
                sock.map_or(ptr::null(), |c| c.as_ptr()),
                conn_opts.flags.into(),
            )
        };
        self.check_for_errors(ClientError::ConnectError)?;

        if res.is_null() {
            Ok(this)
        } else {
            let msg = "unspecified query error".into();
            Err(ClientError::QueryError(0, msg))
        }
    }

    /// Run a query and return the number of rows affected
    ///
    /// # Errors
    ///
    /// Error if the query could not be completed
    #[inline]
    pub fn execute(&mut self, q: &str) -> ClientResult<u64> {
        self.mysql_query(q)?;
        let count = self.mysql_affected_rows().unwrap_or(0);

        // FIXME: this seems hacky. If we have a field count, we need to store then drop the result,
        // otherwise it seems like we never get set back to `MYSQL_STATUS_READY`. If there is no
        // field count, we can't store the rows because that returns an error. But
        // `mysql_field_count_func` isn't even available...?
        let fields_count = self.mysql_field_count();
        if fields_count != 0 {
            // Can we just discard somehow since `execute` expectes no result?
            let _rows = unsafe { self.mysql_store_result()? };
        }

        Ok(count)
    }

    /// Run a query for lazily loaded results
    ///
    /// # Errors
    ///
    /// Error if the row could not be fetched
    #[inline]
    pub fn query<'conn>(&'conn mut self, q: &str) -> ClientResult<Rows<'conn>> {
        self.mysql_query(q)?;
        let fields_count = self.mysql_field_count();
        let rows = if fields_count == 0 {
            // See FIXMEs from execute
            Rows::empty(self)
        } else {
            unsafe { self.mysql_store_result()? }
        };

        Ok(rows)
    }

    /// Initialize the connection
    #[allow(clippy::unnecessary_wraps)]
    fn mysql_init() -> ClientResult<Self> {
        fn_thread_unsafe_lib_init();

        let p_conn = unsafe { global_func!(mysql_init_func)(ptr::null_mut()) };
        let p_conn = NonNull::new(p_conn).expect("OOM: connection allocation failed");

        // Validate we are using an expected charset
        let charset = unsafe {
            global_func!(mysql_options_func)(
                p_conn.as_ptr(),
                bindings::mysql_option::MYSQL_SET_CHARSET_NAME,
                b"utf8mb4\0".as_ptr().cast(),
            )
        };
        assert_eq!(0, charset, "MYSQL_SET_CHARSET_NAME not recognized");

        Ok(Self { inner: p_conn })
    }

    /// Execute a query
    fn mysql_query(&mut self, q: &str) -> ClientResult<()> {
        log::debug!("start query");
        // mysql_real_query in mariadb_lib.c. Real just means use buffers
        // instead of c strings
        let res = unsafe {
            global_func!(mysql_real_query_func)(
                self.inner.as_ptr(),
                q.as_ptr().cast(),
                q.len().try_into().unwrap(),
            )
        };

        self.check_for_errors(ClientError::QueryError)?;

        // Zero for success, nonzero for errors
        if res == 0 {
            Ok(())
        } else {
            let msg = "unspecified query error";
            Err(ClientError::QueryError(0, msg.into()))
        }
    }

    /// This is weird. It seems like if there are no rows, it returns -1? This gets set in
    /// `loc_advanced_command` and never set to something more useful. Weird.
    ///
    /// FIXME: get some clarification here
    fn mysql_affected_rows(&mut self) -> Option<u64> {
        let res = unsafe { global_func!(mysql_affected_rows_func)(self.inner.as_ptr()) };
        if res == c_ulonglong::MAX {
            None
        } else {
            Some(res)
        }
    }

    /// Doesn't seem like `mysql_field_count_func` is available
    fn mysql_field_count(&mut self) -> u32 {
        unsafe { (*self.inner.as_ptr()).field_count }
    }

    /// Prepare the result for iteration by storing them
    ///
    /// FIXME: we would rather use `mysql_use_result` so we don't need to store the whole set,
    /// but seems like it isn't available via `service_sql`?
    ///
    /// # Safety
    ///
    /// This may only be called after `mysql_query`
    unsafe fn mysql_store_result(&mut self) -> ClientResult<Rows<'_>> {
        debug!("use result");
        // SAFETY: we call use_result with a valid connection pointer
        debug!("mysql_use_result call");
        // debug!("MYSQL: {:#?}", unsafe { &*self.inner.as_ptr() });

        let res = unsafe { global_func!(mysql_store_result_func)(self.inner.as_ptr()) };
        // WHY DOESN'T THIS WORK
        // let res = unsafe { bindings::mysql_use_result(self.inner.as_ptr()) };
        debug!("res: {res:p}");

        if let Some(res_ptr) = NonNull::new(res) {
            // SAFETY: nonnull pointer from use_result is valid
            let mysql_res = unsafe { &mut *res_ptr.as_ptr() };
            let field_count = mysql_res.field_count;

            // FIXME: we don't seem to have mysql_fetch_fields. It's just an accessor though.
            // SAFETY: FFI call with a valid pointer
            // let field_ptr = unsafe { bindings::mysql_fetch_fields(mysql_res) };

            // if field_ptr.is_null() {
            //     // This function should never fail to my knowledge
            //     if let Err(e) = self.check_for_errors(ClientError::QueryError) {
            //         error!("fatal error: {e}");
            //     };
            //     // SAFETY: FFI call with valid pointer
            //     unsafe { global_func!(mysql_free_result_func)(mysql_res) };
            //     panic!("mysql_fetch_fields returned null! exiting");
            // }

            // SAFETY: FFI provided us a valid pointer and length
            let fields = unsafe {
                slice::from_raw_parts(mysql_res.fields, mysql_res.field_count.try_into().unwrap())
            };
            // let fields =
            //     unsafe { slice::from_raw_parts(field_ptr, field_count.try_into().unwrap()) };

            debug!("FIELDS: {fields:#?}");

            let field_meta = unsafe {
                slice::from_raw_parts(
                    mysql_res.fields.cast(),
                    mysql_res.field_count.try_into().unwrap(),
                )
            };

            let rows = Rows {
                conn: self,
                inner: Some(res_ptr),
                field_meta: Some(field_meta),
            };
            Ok(rows)
        } else {
            debug!("ERROR PATH");
            self.check_for_errors(ClientError::QueryError)?;
            let msg = "unspecified fetch error, maybe this shouldn't return any rows?".into();
            Err(ClientError::FetchError(0, msg))
        }
    }

    /// Get the last error message if available and if so, apply it to function `f`
    ///
    /// `f` is usually a variant of `ClientError::SomeError`, since those are functions
    fn check_for_errors<F>(&mut self, f: F) -> ClientResult<()>
    where
        F: FnOnce(u32, String) -> ClientError,
    {
        let emsg;
        let errno;
        unsafe {
            let cs = CStr::from_ptr(global_func!(mysql_error_func)(self.inner.as_ptr()));
            emsg = cs.to_string_lossy();
            errno = global_func!(mysql_errno_func)(self.inner.as_ptr());
        }

        if emsg.is_empty() && errno == 0 {
            Ok(())
        } else {
            Err(f(errno, emsg.into_owned()))
        }
    }
}

impl Drop for Connection {
    /// Close the connection
    fn drop(&mut self) {
        unsafe { global_func!(mysql_close_func)(self.inner.as_ptr()) };
    }
}

pub struct Rows<'res> {
    /// The parent connection
    conn: &'res Connection,
    /// Pointer to the result. If `None`, we have no rows
    inner: Option<NonNull<bindings::MYSQL_RES>>,
    /// The fields that were part of this row. Lazily initialized
    field_meta: Option<&'res [FieldMeta<'res>]>,
}

impl<'res> Rows<'res> {
    /// Create an empty rows iterator
    fn empty(conn: &'res Connection) -> Self {
        Self {
            conn,
            inner: None,
            field_meta: None,
        }
    }
}

impl Drop for Rows<'_> {
    fn drop(&mut self) {
        // SAFETY: we hold a valid pointer
        if let Some(ptr) = self.inner {
            unsafe { global_func!(mysql_free_result_func)(ptr.as_ptr()) };
        }
    }
}

impl<'res> Iterator for Rows<'res> {
    type Item = Row<'res>;

    // NOTE: this implementation works when all rows are stored. If each one needs to be fetched/
    // freed (e.g. mysql_use_result) then this will need different lifetime setup so you can't have
    // two rows existing at once
    fn next(&mut self) -> Option<Self::Item> {
        // type `bindings::MYSQL_ROW`, `*mut *mut c_char`
        let rptr = unsafe { global_func!(mysql_fetch_row_func)(self.inner?.as_ptr()) };
        let field_ptrs = unsafe { slice::from_raw_parts(rptr, self.field_meta.unwrap().len()) };

        if rptr.is_null() {
            None
        } else {
            Some(Row {
                field_ptrs,
                field_meta: self.field_meta.unwrap(),
            })
        }
    }
}

/// Representation of a single row, as part of a SQL query result
pub struct Row<'row> {
    /// This stores the actual data
    /// `*mut *mut c_char`
    field_ptrs: &'row [*mut c_char],
    /// Information about the fields in the result
    field_meta: &'row [FieldMeta<'row>],
}

impl Row<'_> {
    /// Get the field of a given index. Panics if out of range
    pub fn field(&self, index: usize) -> Value {
        let meta = &self.field_meta[index];
        let field_ptr = self.field_ptrs[index];
        unsafe { Value::from_str_ptr(meta.ftype(), field_ptr, meta.max_length()) }
    }

    pub const fn field_info(&self, index: usize) -> &FieldMeta {
        &self.field_meta[index]
    }

    /// Get the total number of fields
    pub const fn field_count(&self) -> usize {
        self.field_meta.len()
    }

    /// Iterator over values in the row
    pub fn fields(&self) -> impl Iterator<Item = Field> {
        self.field_meta.iter().enumerate().map(|(idx, meta)| Field {
            value: self.field(idx),
            meta,
        })
    }
}

impl fmt::Debug for Row<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_struct("FetchedRow");
        for field in self.fields() {
            f.field(field.meta.name(), &field.value);
        }
        f.finish()
    }
}

/// A value plus field metadata
pub struct Field<'row> {
    value: Value<'row>,
    meta: &'row FieldMeta<'row>,
}

impl Field<'_> {
    pub fn value(&self) -> Value {
        self.value
    }

    pub fn name(&self) -> &str {
        self.meta.name()
    }
}

/// Transparant wrapper around a `MYSQL_FIELD`. This does not contain data, only field meta
#[repr(transparent)]
#[derive(Debug)]
pub struct FieldMeta<'row> {
    inner: bindings::MYSQL_FIELD,
    phantom: PhantomData<&'row ()>,
}

impl FieldMeta<'_> {
    pub fn length(&self) -> usize {
        self.inner.length.try_into().unwrap()
    }

    pub fn max_length(&self) -> usize {
        self.inner.max_length.try_into().unwrap()
    }

    pub fn name(&self) -> &str {
        let name_ptr = self.inner.name.cast();
        let name_len = self.inner.name_length.try_into().unwrap();
        let name_slice = unsafe { slice::from_raw_parts(name_ptr, name_len) };
        str::from_utf8(name_slice).expect("non-utf8 identifier")
    }

    fn ftype(&self) -> bindings::enum_field_types {
        self.inner.type_
    }
}

/// Options for connecting to a remote SQL server
pub struct ConnectionOpts {
    host: Option<CString>,
    database: Option<CString>,
    user: Option<CString>,
    password: Option<CString>,
    port: Option<u16>,
    unix_socket: Option<CString>,
    flags: u32,
}

/// Given a query result and connection, find the number of fields
unsafe fn get_field_count(
    conn: &mut Connection,
    q_result: *const bindings::MYSQL_RES,
) -> ClientResult<u32> {
    debug!("get field count");
    let res = unsafe { global_func!(mysql_num_fields_func)(q_result.cast_mut()) };
    conn.check_for_errors(ClientError::QueryError)?;
    Ok(res)
}

fn fn_thread_unsafe_lib_init() {
    /// <https://dev.mysql.com/doc/refman/5.7/en/mysql-init.html>
    static MYSQL_THREAD_UNSAFE_INIT: Once = Once::new();

    MYSQL_THREAD_UNSAFE_INIT.call_once(|| {
        // FIXME: do we need anything here?
        // mysql_library_init is defined by `#define mysql_library_init mysql_server_init`
        // which isn't picked up by bindgen
        // let ret = unsafe { bindings::mysql_server_init(0, ptr::null_mut(), ptr::null_mut()) };
        // let ret = unsafe { global_func!(mysql_server_init)(0, ptr::null_mut(), ptr::null_mut()) };
        // assert_eq!(
        //     ret, 0,
        //     "Unable to perform MySQL global initialization. Return code: {ret}"
        // );
    });
}

// Can't do this for rows because we need to consume them. Maybe we could have a
// `.display() -> RowsDisplay` that can do this?
// impl fmt::Display for FetchedRow<'_> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let mut s = String::new();
//         let mut widths = Vec::new();
//         let mut last_len = s.len();

//         // Write the fields first; record lengths
//         for field in self.fields() {
//             write!(s, "| {}", field.value)
//             f.field(field.meta.name(), &field.value);
//         }
//         todo!()
//     }
// }
