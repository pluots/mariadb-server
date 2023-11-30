//! EXAMPLE ONLY: DO NOT USE IN PRODUCTION!
#![allow(unused)]

use std::cell::UnsafeCell;
use std::ffi::c_void;
use std::fmt::Write;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Mutex;

use josekit::jws;
use mariadb::log::{debug, error, info};
use mariadb::plugin::encryption::{Encryption, KeyError, KeyManager};
use mariadb::plugin::{
    register_plugin, Init, InitError, License, Maturity, PluginType, SysVarConstString, SysVarOpt,
};
use mariadb::service_sql::{ClientError, Connection, Rows};

/// Table to store key data
const KEY_TABLE: &str = "mysql.clevis_keys";

/// Max length a key can be, we allow for an AES256-GCM key.
// FIXME: it seems like the encryption plugin should really determine this - what happens if it
// gets the wrong size? Does too long get truncated? Does too short go through a KDF?
const KEY_BYTES: usize = 32;

/// String system variable to set server address
// TODO: when recovering keys, do we want to use the stored URL or this variable?
static TANG_SERVER: SysVarConstString = SysVarConstString::new();

register_plugin! {
    KeyMgtClevis,
    ptype: PluginType::MariaEncryption,
    name: "clevis_key_management",
    author: "Daniel Black & Trevor Gross",
    description: "Clevis key management plugin",
    license: License::Gpl,
    maturity: Maturity::Experimental,
    version: "0.1",
    init: KeyMgtClevis,
    encryption: false,
    variables: [
        SysVar {
            ident: TANG_SERVER,
            vtype: SysVarConstString,
            name: "tang_server",
            description: "the tang server to use for key exchange",
            options: [SysVarOpt::OptionalCliArg],
            default: "localhost"
        }
    ]
}

struct KeyMgtClevis;

impl Init for KeyMgtClevis {
    /// Create needed tables
    fn init() -> Result<(), InitError> {
        let mut conn = Connection::connect_local().map_err(|e| {
            error!("error with local connection: {e}");
            InitError
        })?;

        conn.execute(&format!(
            "CREATE TABLE IF NOT EXISTS {KEY_TABLE} (
                key_id INT UNSIGNED NOT NULL COMMENT 'MariaDB key_id',
                key_version INT UNSIGNED NOT NULL COMMENT 'MariaDB key_version',
                metadata TEXT NOT NULL COMMENT '[WARNING: SENSITIVE!] information to recover the key',
                PRIMARY KEY (key_id, key_version)
            ) ENGINE=InnoDB"
        ))
        .map_err(|e| {
            error!("error creating table {KEY_TABLE}: {e}");
            InitError
        })?;

        Ok(())
    }
}

impl KeyManager for KeyMgtClevis {
    fn get_latest_key_version(key_id: u32) -> Result<u32, KeyError> {
        let mut conn = key_connect()?;

        execute_with_transaction(&mut conn, key_id, |conn| {
            // This takes a row-level lock
            let mut q = format!(
                "SELECT key_version FROM {KEY_TABLE}
                WHERE key_id = {key_id}
                ORDER BY key_version DESC
                LIMIT 1
                FOR UPDATE"
            );

            {
                let mut rows = key_query(conn, &q, key_id)?;

                // Key exists, return the version
                if let Some(row) = rows.next() {
                    let version_field = row.field(0);
                    let version = version_field.as_int().unwrap().try_into().unwrap();
                    assert!(rows.next().is_none(), "should only return one row");
                    return Ok(version);
                }
            }

            // TODO: no key rotation yet so all key versions are 1 for now
            let key_version: u32 = 1;
            let new_key_meta = tang_make_new_key()?;

            let q = format!(
                "INSERT INTO {KEY_TABLE}
                    (key_id, key_version, metadata)
                VALUES
                    ({key_id}, {key_version}, '{new_key_meta}')"
            );

            key_execute(conn, &q, key_id)?;

            Ok(key_version)
        })
    }

    fn get_key(key_id: u32, key_version: u32, dst: &mut [u8]) -> Result<(), KeyError> {
        let mut conn = key_connect()?;

        execute_with_transaction(&mut conn, key_id, |conn| {
            let q = format!(
                "SELECT key_id, key_version, metadata FROM {KEY_TABLE}
                WHERE key_id = {key_id}
                AND key_version = {key_version}"
            );

            let mut rows = key_query(conn, &q, key_id)?;

            let Some(row) = rows.next() else {
                error!("missing row for key ID {key_id} version {key_version}");
                return Err(KeyError::Other);
            };

            if rows.next().is_some() {
                error!("expected a single row for key ID {key_id} version {key_version}");
                return Err(KeyError::Other);
            }

            let key_id_col = row.field(0).as_int();
            let key_version_col = row.field(1).as_int();
            let meta_str_col = row.field(2).as_bytes();

            if key_id_col.is_none() || key_version_col.is_none() || meta_str_col.is_none() {
                error!("invalid columns for kid {key_id} version {key_version}");
                return Err(KeyError::Other);
            }

            tang_retrieve_key(
                key_id_col.unwrap(),
                key_version_col.unwrap(),
                meta_str_col.unwrap(),
                dst,
            )
        })
    }

    fn key_length(_key_id: u32, _key_version: u32) -> Result<usize, KeyError> {
        Ok(KEY_BYTES)
    }
}

/// Wrap some action in a transaction (`START TRANSACTION ... COMMIT/ROLLBACK`).
fn execute_with_transaction<F, T>(conn: &mut Connection, key_id: u32, f: F) -> Result<T, KeyError>
where
    F: FnOnce(&mut Connection) -> Result<T, KeyError>,
{
    key_execute(conn, "START TRANSACTION", key_id)?;

    let res = f(conn);

    match res {
        Ok(_) => key_execute(conn, "COMMIT", key_id)?,
        Err(_) => key_execute(conn, "ROLLBACK", key_id)?,
    };

    res
}

/// Contact the Tang server to provision a new private key. The key gets dropped, return the
/// metadata to store.
fn tang_make_new_key() -> Result<String, KeyError> {
    let client = clevis::TangClient::new(TANG_SERVER.get(), None);

    let generated = client.create_secure_key::<KEY_BYTES>().map_err(|e| {
        error!("failure generating key: {e}");
        KeyError::Other
    })?;

    Ok(generated.meta.to_json())
}

fn tang_retrieve_key(
    key_id: i64,
    key_version: i64,
    meta_str: &[u8],
    dst: &mut [u8],
) -> Result<(), KeyError> {
    // The rust wrapper handles the failure mode
    assert!(
        dst.len() == KEY_BYTES,
        "received incorrect key destination buffer size"
    );

    let meta = clevis::KeyMeta::from_json_bytes(meta_str).map_err(|e| {
        error!("failure parsing metadata at id {key_id} version {key_version}: {e}");
        KeyError::Other
    })?;

    let client = meta.client(None);

    let new_key = client.recover_secure_key::<KEY_BYTES>(&meta).map_err(|e| {
        error!("failure to recover key id {key_id} version {key_version}: {e}");
        KeyError::Other
    })?;

    dst.copy_from_slice(new_key.as_bytes());
    Ok(())
}

/// Connect to the local server. Return a KeyError on failure and print a message
fn key_connect() -> Result<Connection, KeyError> {
    Connection::connect_local().map_err(|e| {
        error!("error connecting: {e}");
        KeyError::Other
    })
}

/// Helper to execute a query, printing an error and returning KeyError if needed. Returns number
/// of updated rows.
fn key_execute(conn: &mut Connection, q: &str, key_id: u32) -> Result<u64, KeyError> {
    // FIXME: don't print actual values
    conn.execute(q).map_err(|e| {
        error!("execute key_id: {key_id}: SQL error: {e}. Query:\n{q}");
        KeyError::Other
    })
}

/// Helper to execute a query, printing an error, return the result
fn key_query<'conn>(
    conn: &'conn mut Connection,
    q: &str,
    key_id: u32,
) -> Result<Rows<'conn>, KeyError> {
    conn.query(q).map_err(|e| {
        error!("query key_id {key_id}: SQL error: {e}. Query:\n{q}");
        KeyError::Other
    })
}
