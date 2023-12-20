#![allow(unused)]

use std::ffi::c_int;

use mariadb::log::debug;

type Error = ();
type Result<T = (), E = Error> = std::result::Result<T, E>;

fn init(hton: *mut mariadb::bindings::handlerton) -> c_int {
    debug!("example init function");
    let hton = unsafe { &mut *hton };
    // hton.create = c
    // hton
    todo!()
}

struct Mode(c_int);
struct HandlerCtx(mariadb::bindings::handler);

impl HandlerCtx {
    fn table(&self) -> &Table {
        // unsafe { &*self.0.table }
        todo!()
    }
}

struct Table(mariadb::bindings::TABLE);

impl Table {
    fn options(&self) {
        // self.0.
    }
}

// BIG QUESTION: how do we provide Rust implementations to C++ classes?
// One option could be to use the `cpp` crate, but we should be able
// to do something better. Maybe figure out what symbols it looks for.
//
// After all, it must use symbols via a handlerton

/// This is our way of faking inheritance
#[repr(C)]
struct HandlerWrapper<T: Handler> {
    handler: mariadb::bindings::handler,
    this: T,
}

trait Handler {
    fn open(&self, ctx: &HandlerCtx, name: &str, mode: Mode, test_if_locked: bool) -> Result;
    // TODO: should this just be drop?
    fn close() {}
    fn write_row(&self, ctx: &HandlerCtx, buf: &[u8]) -> Result {
        Ok(())
    }
    fn update_row(&self, ctx: &HandlerCtx, buf: &[u8]) -> Result {
        Ok(())
    }
    fn delete_row(&self, ctx: &HandlerCtx, buf: &[u8]) -> Result {
        Ok(())
    }
}

struct Example {
    //
}

impl Handler for Example {
    fn open(&self, ctx: &HandlerCtx, name: &str, mode: Mode, test_if_locked: bool) -> Result {
        debug!("example open");
        Ok(())
    }
}
