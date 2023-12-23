#![allow(unused)]

use std::ffi::c_int;

use mariadb::log::debug;
use mariadb::plugin::{License, Maturity};
use mariadb::register_plugin_storage;
use mariadb::storage::{Handler, Handlerton};

register_plugin_storage! {
    name: "EXAMPLE_RUST",
    author: "Trevor Gross",
    description: "Sample storage engine plugin",
    license: License::Gpl,
    maturity: Maturity::Experimental,
    version: "0.1",
    handlerton: ExampleHton,
}

struct ExampleHton;
struct ExampleHandler {}

impl Handlerton for ExampleHton {
    type Handler = ExampleHandler;

    type SavePoint = ();

    fn create_handler(table: mariadb::Table, mem_root: mariadb::MemRoot) -> Self::Handler {
        ExampleHandler {}
    }
}

impl Handler for ExampleHandler {}
