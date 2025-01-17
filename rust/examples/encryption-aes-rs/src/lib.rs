//! Basic encryption plugin using:
//!
//! - SHA256 as the hasher to create a key
//! - Randomized key update times
//! - No encryption, just copies from src to dst
//!
//! This is noisy, prints a ton so it is easy to see what is going on

// FIXME: need to use AES ctr when nopad is true

use std::cmp::min;

use aes_gcm::aead::KeyInit;
use aes_gcm::{AeadInPlace, Aes256Gcm};
use encryption_common::trunc_or_extend;
use mariadb::log::{error, info};
use mariadb::plugin::encryption::{Decryption, Encryption, EncryptionError, KeyError, KeyManager};
use mariadb::plugin::*;
use mariadb::warn_once;

/// Length of the AES tag
const AES256_TAG_LEN: usize = 16;
/// Length of the AES nonce
const AES256_NONCE_LEN: usize = 12;
/// Length of the AES key
const AES256_KEY_LEN: usize = 32;

register_plugin! {
    EncryptionExampleAes,
    ptype: PluginType::MariaEncryption,
    name: "encryption_example_aes",
    author: "Trevor Gross",
    description: "Example key management / encryption plugin using AES",
    license: License::Gpl,
    maturity: Maturity::Experimental,
    version: "0.1",
    encryption: TestAes ,
}

struct EncryptionExampleAes;

impl KeyManager for EncryptionExampleAes {
    /// Key version is always 1
    fn get_latest_key_version(_key_id: u32) -> Result<u32, KeyError> {
        Ok(1)
    }

    /// Dummy key manager just uses the key id and version as the key
    fn get_key(key_id: u32, key_version: u32, dst: &mut [u8]) -> Result<(), KeyError> {
        dst[..4].copy_from_slice(&key_id.to_ne_bytes());
        dst[4..8].copy_from_slice(&key_version.to_ne_bytes());
        Ok(())
    }

    fn key_length(_key_id: u32, _key_version: u32) -> Result<usize, KeyError> {
        Ok(8)
    }
}

struct TestAes {
    update_called_times: usize,
    cipher: Aes256Gcm,
    nonce: [u8; AES256_NONCE_LEN],
    tag: [u8; AES256_TAG_LEN],
}

impl Encryption for TestAes {
    fn init(
        _key_id: u32,
        _key_version: u32,
        key: &[u8],
        iv: &[u8],
        _same_size: bool,
    ) -> Result<Self, EncryptionError> {
        info!("encrypt init");
        let (cipher, nonce) = init_cipher(key, iv)?;
        Ok(Self {
            update_called_times: 0,
            cipher,
            nonce,
            tag: [0u8; AES256_TAG_LEN],
        })
    }

    fn update(&mut self, src: &[u8], dst: &mut [u8]) -> Result<usize, EncryptionError> {
        info!(
            "encrypt update, src_len {} dst_len {}",
            src.len(),
            dst.len()
        );
        assert!(
            self.update_called_times == 0,
            "AES cannot call update more than once!"
        );

        // Save enough room for the tag
        let data_len = min(dst.len() - AES256_TAG_LEN, src.len());
        let src = &src[..data_len];
        let data_dst = &mut dst[..data_len];
        // Copy data and then encrypt in place
        data_dst.copy_from_slice(src);
        let tag = self
            .cipher
            .encrypt_in_place_detached(&self.nonce.into(), b"", data_dst)
            .map_err(|e| {
                error!("AES encryption error: {e}");
                EncryptionError::Other
            })?;

        self.update_called_times += 1;
        self.tag = tag.into();

        Ok(data_dst.len())
    }

    /// BUG: I don't know what to do with the tagl we get a dst length of 0
    /// Write the tag to the buffer
    fn finish(&mut self, dst: &mut [u8]) -> Result<usize, EncryptionError> {
        info!("encrypt finish, dst_len {}", dst.len());
        assert!(self.update_called_times == 1);

        if dst.len() < AES256_TAG_LEN {
            error!(
                "AES encrypt finish requires {AES256_TAG_LEN} bytes but got {}",
                dst.len()
            );
            return Err(EncryptionError::Other);
        }

        dst[..AES256_TAG_LEN].copy_from_slice(&self.tag);
        Ok(AES256_TAG_LEN)
    }

    fn encrypted_length(_key_id: u32, _key_version: u32, src_len: usize) -> usize {
        src_len + AES256_TAG_LEN
    }
}

impl Decryption for TestAes {
    fn init(
        _key_id: u32,
        _key_version: u32,
        key: &[u8],
        iv: &[u8],
        _same_size: bool,
    ) -> Result<Self, EncryptionError> {
        info!("decrypt init");
        let (cipher, nonce) = init_cipher(key, iv)?;
        Ok(Self {
            update_called_times: 0,
            cipher,
            nonce,
            tag: [0u8; AES256_TAG_LEN],
        })
    }

    fn update(&mut self, src: &[u8], dst: &mut [u8]) -> Result<usize, EncryptionError> {
        info!("decrypt, src_len {} dst_len {}", src.len(), dst.len());
        assert!(
            self.update_called_times == 0,
            "AES cannot call update more than once!"
        );

        if src.len() < AES256_TAG_LEN {
            error!(
                "AES decryption requires {AES256_TAG_LEN} bytes but got {}",
                dst.len()
            );
        }

        let src_data = &src[..src.len() - AES256_TAG_LEN];
        let src_tag = &src[src.len() - AES256_TAG_LEN..];
        // Dst winds up with the entire source minus the tag
        let use_dst = &mut dst[..src_data.len()];

        use_dst.copy_from_slice(src_data);
        self.cipher
            .decrypt_in_place_detached(&self.nonce.into(), b"", use_dst, src_tag.into())
            .map_err(|e| {
                error!("AES decryption update error {e}");
                EncryptionError::Other
            })?;
        self.update_called_times += 1;
        dbg!(Ok(src_data.len()))
    }

    fn finish(&mut self, _dst: &mut [u8]) -> Result<usize, EncryptionError> {
        Ok(0)
    }
}

fn init_cipher(
    in_key: &[u8],
    in_iv: &[u8],
) -> Result<(Aes256Gcm, [u8; AES256_NONCE_LEN]), EncryptionError> {
    info!(
        "init_cypher, key_len {}, iv_len {}",
        in_key.len(),
        in_iv.len()
    );

    let (key, key_ok, key_action) = trunc_or_extend::<AES256_KEY_LEN>(in_key);
    let (nonce, nonce_ok, nonce_action) = trunc_or_extend::<AES256_NONCE_LEN>(in_iv);

    if !key_ok {
        warn_once!(
            "AES256 expects {AES256_KEY_LEN}-byte key but got {}. \
            {key_action} to meet requirements.",
            in_key.len()
        );
    }
    if !nonce_ok {
        warn_once!(
            "AES256 expects {AES256_NONCE_LEN}-byte nonce but got {}. \
            {nonce_action} to meet requirements.",
            in_iv.len()
        );
    }

    let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
    Ok((cipher, nonce))
}
