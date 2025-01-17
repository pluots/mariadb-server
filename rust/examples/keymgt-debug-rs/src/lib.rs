//! Debug key management
//!
//! Use to debug the encryption code with a fixed key that changes only on user
//! request. The only valid key ID is 1.
//!
//! EXAMPLE ONLY: DO NOT USE IN PRODUCTION!

use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};

use mariadb::log::{self, debug, trace};
use mariadb::plugin::encryption::{KeyError, KeyManager};
use mariadb::plugin::{
    register_plugin, Init, InitError, License, Maturity, PluginType, SysVarConstString, SysVarOpt,
    SysVarString,
};

const KEY_LENGTH: usize = 4;
static KEY_VERSION: AtomicU32 = AtomicU32::new(1);
static TEST_SYSVAR_CONST_STR: SysVarConstString = SysVarConstString::new();
static TEST_SYSVAR_STR: SysVarString = SysVarString::new();
static TEST_SYSVAR_I32: AtomicI32 = AtomicI32::new(10);

struct DebugKeyMgmt;

impl Init for DebugKeyMgmt {
    fn init() -> Result<(), InitError> {
        log::set_max_level(log::LevelFilter::Trace);
        debug!("DebugKeyMgmt get_latest_key_version");
        trace!(
            "current const str sysvar: {:?}",
            TEST_SYSVAR_CONST_STR.get()
        );
        trace!("current str sysvar: {:?}", TEST_SYSVAR_STR.get());
        trace!(
            "current sysvar: {}",
            TEST_SYSVAR_I32.load(Ordering::Relaxed)
        );
        Ok(())
    }

    fn deinit() -> Result<(), InitError> {
        eprintln!("deinit for DebugKeyMgmt");
        Ok(())
    }
}

impl KeyManager for DebugKeyMgmt {
    fn get_latest_key_version(key_id: u32) -> Result<u32, KeyError> {
        debug!("DebugKeyMgmt get_latest_key_version");
        if key_id != 1 {
            Err(KeyError::InvalidVersion)
        } else {
            Ok(KEY_VERSION.load(Ordering::Relaxed))
        }
    }

    fn get_key(key_id: u32, key_version: u32, dst: &mut [u8]) -> Result<(), KeyError> {
        debug!("DebugKeyMgmt get_key, id {key_id}, version {key_version}");
        if key_id != 1 {
            return Err(KeyError::InvalidVersion);
        }

        // Convert our integer to a native endian byte array
        let key_buf = KEY_VERSION.load(Ordering::Relaxed).to_ne_bytes();

        if dst.len() < key_buf.len() {
            return Err(KeyError::BufferTooSmall);
        }

        // Copy our slice to the buffer, return the copied length
        dst[..key_buf.len()].copy_from_slice(key_buf.as_slice());
        Ok(())
    }

    fn key_length(key_id: u32, key_version: u32) -> Result<usize, KeyError> {
        debug!("DebugKeyMgmt key_length, id {key_id}, version {key_version}");
        // Return the length of our u32 in bytes
        // Just verify our types don't change
        debug_assert_eq!(
            KEY_LENGTH,
            KEY_VERSION.load(Ordering::Relaxed).to_ne_bytes().len()
        );
        Ok(KEY_LENGTH)
    }
}

register_plugin! {
    DebugKeyMgmt,
    ptype: PluginType::MariaEncryption,
    name: "debug_key_management",
    author: "Trevor Gross",
    description: "Debug key management plugin",
    license: License::Gpl,
    maturity: Maturity::Experimental,
    version: "0.2",
    init: DebugKeyMgmt, // optional
    encryption: false,
    variables: [
        SysVar {
            ident: TEST_SYSVAR_CONST_STR,
            vtype: SysVarConstString,
            name: "test_sysvar_const_string",
            description: "this is a description",
            options: [SysVarOpt::OptionalCliArg],
            default: "default value"
        },
        SysVar {
            ident: TEST_SYSVAR_STR,
            vtype: SysVarString,
            name: "test_sysvar_string",
            description: "this is a description",
            options: [SysVarOpt::OptionalCliArg],
            default: "other default value"
        },
        SysVar {
            ident: TEST_SYSVAR_I32,
            vtype: AtomicI32,
            name: "test_sysvar_i32",
            description: "this is a description",
            options: [SysVarOpt::OptionalCliArg],
            default: 67
        }
    ]
}
