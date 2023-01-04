//! Basic encryption plugin using:
//! 
//! - SHA256 as the hasher

#![allow(unused)]

use std::time::{Instant, Duration};
use std::sync::Mutex;
use rand::{Rng};
use sha2::{Sha256 as Hasher, Digest};

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce // Or `Aes128Gcm`
};

use mariadb_server::plugin::encryption::{KeyError, Encryption, Flags};
use mariadb_server::plugin::Init;
// use mariadb_server::plugin::Init;
// use mariadb_server::plugin::prelude::*;

// plugin_encryption!{
//     type: RustEncryption,
//     init: RustEncryptionInit, // optional
//     name: "example_key_management",
//     author: "MariaDB Team",
//     description: "Example key management plugin using AES",
//     license: GPL,
//     stability: EXPERIMENTAL
// }


/// Range of key rotations, as seconds
const KEY_ROTATION_MIN: f32 = 45.0;
const KEY_ROTATION_MAX: f32 = 90.0;
const KEY_ROTATION_INTERVAL: f32 = KEY_ROTATION_MAX - KEY_ROTATION_MIN;
const SHA256_SIZE: usize = 32;
// const KEY_ROTATION_INTERVAL: Duration =
//     KEY_ROTATION_MAX - KEY_ROTATION_MIN;

/// Our global key version state
static KEY_VERSIONS: Mutex<Option<KeyVersions>> = Mutex::new(None);

/// Contain the state of our keys. We use `Instant` (the monotonically)
/// increasing clock) instead of `SystemTime` (which may occasionally go
/// backwards)
struct KeyVersions {
    /// Initialization time of the struct, reference point for key version
    start: Instant,
    /// Most recent key update time
    current: Instant,
    /// Next time for a key update
    next: Instant
}

impl KeyVersions {
    /// Initialize with a new value. Returns the struct 
    fn new_now() -> Self {
        let now = Instant::now();
        let mut ret = Self {
            start: now,
            current: now,
            next: now
        };
        ret.update_next();
        ret
    }

    fn update_next(&mut self) {
        let mult = rand::thread_rng().gen_range(0.0..1.0);
        let add_duration = KEY_ROTATION_MIN + mult * KEY_ROTATION_INTERVAL;
        self.next += Duration::from_secs_f32(add_duration);
    }

    /// Update the internal duration if needed, and return the elapsed time
    fn update_returning_version(&mut self) -> u64 {
        let now = Instant::now();
        if now > self.next {
            self.current = now;
            self.update_next();
        }
        (self.next - self.start).as_secs()
    }
}

// Uninitialized 


struct RustEncryptionInit;

impl Init for RustEncryptionInit {
    fn init() {
        let mut guard = KEY_VERSIONS.lock().unwrap();
        *guard = Some(KeyVersions::new_now());
    }
}


struct RustEncryption;

impl Encryption for RustEncryption {
    fn get_latest_key_version(_key_id: u32) -> Result<u32, KeyError> {
        let mut guard = KEY_VERSIONS.lock().unwrap();
        let mut vers = guard.unwrap();
        Ok(vers.update_returning_version() as u32)
    }
    
    /// Given a key ID and a version, create its hash
    fn get_key(key_id: u32, key_version: u32, dst: &mut [u8]) -> Result<usize, KeyError>  {
        let output_size = Hasher::output_size();
        if dst.len() < output_size {
            return Err(KeyError::BufferTooSmall);
        }
        let mut hasher = Hasher::new();
        hasher.update(key_id.to_ne_bytes());
        hasher.update(key_version.to_ne_bytes());
        dst[..output_size].copy_from_slice(&hasher.finalize());
        Ok(output_size)
    }
    
    fn get_key_length(key_id: u32, key_version: u32) -> Result<usize, KeyError> {
        // All keys have the same length
        Ok(Hasher::output_size())
    }

    /// Initialize
    fn init(key_id: u32, key_version: u32, key: &[u8], iv: &[u8], flags: Flags) -> Self {
        todo!()

    }

    /// Initialize
    fn update(ctx: &mut Self, src: &[u8], dst: &mut [u8]) -> usize {
        todo!()
    }

    fn finish(ctx: &mut Self, dst: &mut [u8]) -> usize{
        todo!()
    }
    
    fn encrypted_length(key_id: u32, key_version: u32, src_len: usize) {
        todo!()
    }
}
