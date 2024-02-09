#![allow(unused)]

use std::collections::HashMap;
use std::ffi::{c_int, CStr, CString};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use std::str;
use std::sync::Mutex;

use bytemuck::{Pod, Zeroable};
use mariadb::log::debug;
use mariadb::plugin::{License, Maturity};
use mariadb::storage::{CreateInfo, Handler, Handlerton, Mode, OpenOp, StorageResult};
use mariadb::sys::ResultExt;
use mariadb::table::state::Create;
use mariadb::{register_plugin_storage, MemRoot, TableShare};

/// Extension for the main data file
const EXT_CSV: &str = ".CSV";
/// Extension for files used for repair and upate
const EXT_UPDATE: &str = ".CSN";
/// Extension for meta files
const EXT_META: &str = ".CSN";

const CHECK_HEADER: u8 = 254;
const TINA_VERSION: u8 = 1;

register_plugin_storage! {
    name: "EXAMPLE_RUST_CSV",
    author: "Trevor Gross",
    description: "Reimplementation of ha_tina for debugging",
    license: License::Gpl,
    maturity: Maturity::Experimental,
    version: "0.1",
    handlerton: CsvHton,
    // features: {
    //     can_recreate: true,
    //     supports_log_tables: true,
    //     no_partition: true,
    // },
    // ^ use this to determine traits and to specify create flags
    // Or, idea: use the
}

// static SHARE: Mutex<HashMap<CString, CsvShared>> = Mutex::new(HashMap::new());

struct CsvHton;
struct CsvHandler {}

/// Data about a single table that is open
struct CsvShared {}

impl CsvShared {
    const fn new() -> Self {
        Self {}
    }
}

impl Handlerton for CsvHton {
    type Handler = CsvHandler;

    type SavePoint = ();
}

impl Handler for CsvHandler {
    type Handlerton = CsvHton;

    fn new(table: &TableShare<Create>, mem_root: MemRoot) -> Self {
        Self {}
    }

    fn open(name: &Path, mode: Mode, open_options: OpenOp) -> StorageResult {
        // init share
        todo!()
    }

    fn create(
        &self,
        name: &CStr,
        table: mariadb::Table<Create>,
        create_info: &CreateInfo,
    ) -> StorageResult {
        // TODO: check for null columns

        let name = Path::new(str::from_utf8(name.to_bytes()).unwrap());

        let mut meta_file = File::options()
            .read(true)
            .write(true)
            .truncate(true)
            .open(name.with_extension(EXT_META))
            .err_log()?;
        Meta::write_to_file(&mut meta_file, 0, false)?;
        drop(meta_file);

        File::options()
            .read(true)
            .write(true)
            .truncate(true)
            .open(name.with_extension(EXT_CSV))
            .err_log()?;

        Ok(())
    }

    fn write_row(buf: &CStr) {
        // Nothing to do here
    }
}

#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
struct Meta {
    header: u8,
    version: u8,
    dirty: u8,
    _pad: [u8; 5],
    rows: u64,
    other: [u64; 3],
}

impl Meta {
    fn write_to_file(f: &mut File, rows: usize, dirty: bool) -> StorageResult {
        let mut data = Self {
            header: CHECK_HEADER,
            version: TINA_VERSION,
            dirty: dirty.into(),
            rows: rows.try_into().unwrap(),
            ..Default::default()
        };
        data.swap_if_be();
        f.write_all(bytemuck::bytes_of(&data)).err_log()?;
        f.seek(SeekFrom::Start(0)).err_log()?;
        f.sync_data().err_log()?;
        Ok(())
    }

    fn swap_if_be(&mut self) {
        if cfg!(target_endian = "big") {
            let mut new = Self {
                header: self.header.swap_bytes(),
                version: self.version.swap_bytes(),
                dirty: self.dirty.swap_bytes(),
                _pad: self._pad,
                rows: self.rows.swap_bytes(),
                other: self.other,
            };
            for i in new.other {
                i.swap_bytes();
            }
            *self = new;
        }
    }
}
