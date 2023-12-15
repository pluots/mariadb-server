#![allow(unused)]

use std::ffi::{c_int, CStr};
use std::path::Path;

use mariadb::log::debug;
use mariadb::plugin::{License, Maturity};
use mariadb::storage::{Handler, Handlerton, Mode, OpenOp, StorageResult};
use mariadb::{register_plugin_storage, MemRoot, TableShare};

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
}

impl Handler for ExampleHandler {
    type Handlerton = ExampleHton;

    // #[mariadb::dbug::instrument]
    fn new(table: &TableShare, mem_root: MemRoot) -> Self {
        Self {}
    }

    fn open(name: &Path, mode: Mode, open_options: OpenOp) -> StorageResult {
        // init share
        todo!()
    }

    fn write_row(buf: &CStr) {
        // Nothing to do here
    }
}
