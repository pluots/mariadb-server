//! Utilities common to encryption plugins

use std::cmp::Ordering;

#[macro_use]
pub mod file_key_mgmt;

/// Create an array of a specific size from a buffer by either zero-extending or truncating
///
/// Returns: `(array, input_is_ok, action)`
pub fn trunc_or_extend<const N: usize>(buf: &[u8]) -> ([u8; N], bool, &'static str) {
    match buf.len().cmp(&N) {
        Ordering::Equal => (buf.try_into().unwrap(), true, ""),
        Ordering::Less => {
            let mut tmp = [0u8; N];
            tmp[..buf.len()].copy_from_slice(buf);
            (tmp, false, "Zero extending")
        }
        Ordering::Greater => (buf[..N].try_into().unwrap(), false, "Truncating"),
    }
}
