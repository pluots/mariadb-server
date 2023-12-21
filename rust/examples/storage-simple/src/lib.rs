#![allow(unused)]

use std::ffi::c_int;

use mariadb::log::debug;
use mariadb::plugin::{License, Maturity};
use mariadb::register_plugin_storage;

// type Error = ();
// type Result<T = (), E = Error> = std::result::Result<T, E>;

// fn init(hton: *mut mariadb::bindings::handlerton) -> c_int {
//     debug!("example init function");
//     let hton = unsafe { &mut *hton };
//     // hton.create = c
//     // hton
//     todo!()
// }

// struct Mode(c_int);
// struct HandlerCtx(mariadb::bindings::handler);

// impl HandlerCtx {
//     fn table(&self) -> &Table {
//         // unsafe { &*self.0.table }
//         todo!()
//     }
// }

// struct Table(mariadb::bindings::TABLE);

// impl Table {
//     fn options(&self) {
//         // self.0.
//     }
// }

register_plugin_storage! {
    name: "example_storage",
    author: "Trevor Gross",
    description: "Sample storage engine plugin",
    license: License::Gpl,
    maturity: Maturity::Experimental,
    version: "0.1",

}
