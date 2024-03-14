#![allow(unused)]

use std::ffi::{c_int, CStr};
use std::path::Path;

use mariadb::log::debug;
use mariadb::plugin::{License, Maturity};
use mariadb::storage::{Handler, Handlerton, Mode, OpenOp, StorageResult, TableFlags};
use mariadb::{register_plugin_storage, MemRoot, TableShare};

register_plugin_storage! {
    name: "example_rust_csv",
    author: "Trevor Gross",
    description: "Reimplementation of ha_tina for debugging",
    license: License::Gpl,
    maturity: Maturity::Experimental,
    version: "0.1",
    handlerton: CsvHton,
}

/// Data file extension
const DATA_EXT: &str = ".CSV";
/// Files for repair and update
const REPAIR_UPDATE_EXT: &str = ".CSN";
/// Meta file
const META_EXT: &str = ".CSM";

struct CsvHton;
struct CsvHandler {}

impl Handlerton for CsvHton {
    type Handler = CsvHandler;
    type SavePoint = ();

    const TABLEFILE_EXTENSIONS: &'static [&'static str] = &[DATA_EXT, REPAIR_UPDATE_EXT, META_EXT];
}

impl Handler for CsvHandler {
    type Handlerton = CsvHton;

    // #[mariadb::dbug::instrument]
    fn new(table: &TableShare, mem_root: MemRoot) -> Self {
        Self {}
    }

    fn open(name: &Path, mode: Mode, open_options: OpenOp) -> StorageResult {
        // init share
        todo!()
    }

    fn table_flags(&self) -> TableFlags {
        TableFlags::NO_TRANSACTIONS
            | TableFlags::REC_NOT_IN_SEQ
            | TableFlags::NO_AUTO_INCREMENT
            | TableFlags::BINLOG_ROW_CAPABLE
            | TableFlags::BINLOG_STMT_CAPABLE
            | TableFlags::CAN_EXPORT
            | TableFlags::CAN_REPAIR
            | TableFlags::SLOW_RND_POS
    }

    fn write_row(buf: &CStr) {
        // Nothing to do here
    }
}

impl Drop for CsvHandler {
    fn drop(&mut self) {
        // assert!();
        // assert!();
    }
}
