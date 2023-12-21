use std::cell::UnsafeCell;
use std::ffi::{c_int, c_uint};

use crate::EmptyResult;

/// Used for plugin registrations, which are in global scope.
#[doc(hidden)]
#[derive(Debug)]
#[repr(transparent)]
pub struct UnsafeSyncCell<T>(UnsafeCell<T>);

impl<T> UnsafeSyncCell<T> {
    /// # Safety
    ///
    /// This inner value be used in a Sync/Send way
    pub const unsafe fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    pub const fn as_ptr(&self) -> *const T {
        self.0.get()
    }

    pub const fn get(&self) -> *mut T {
        self.0.get()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.0.get_mut()
    }
}

#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T> Send for UnsafeSyncCell<T> {}
unsafe impl<T> Sync for UnsafeSyncCell<T> {}

pub(crate) fn to_result(val: c_int) -> EmptyResult {
    if val == 0 {
        EmptyResult::Ok(())
    } else {
        EmptyResult::Err(())
    }
}

#[allow(dead_code)]
pub fn str2bool(s: &str) -> Option<bool> {
    const TRUE_VALS: [&str; 3] = ["on", "true", "1"];
    const FALSE_VALS: [&str; 3] = ["off", "false", "0"];
    let lower = s.to_lowercase();
    if TRUE_VALS.contains(&lower.as_str()) {
        Some(true)
    } else if FALSE_VALS.contains(&lower.as_str()) {
        Some(false)
    } else {
        None
    }
}

/// Turn a "a.b" version string into a C integer
///
/// This is ugly and panicky since we need it to be const
pub const fn parse_version_str(s: &str) -> c_uint {
    const fn get_single_val(buf: &[u8], start: usize, end: usize) -> u16 {
        let mut i = start;
        let mut ret: u16 = 0;

        loop {
            if i >= end {
                break;
            }

            let ch = buf[i];
            assert!(ch >= b'0' && ch <= b'9');
            ret *= 10;
            ret += (ch - b'0') as u16;

            i += 1;
        }

        ret
    }

    let buf = s.as_bytes();

    let mut i = 0;
    let mut dot_pos = None;
    loop {
        if i > buf.len() {
            break;
        }
        if buf[i] == b'.' {
            dot_pos = Some(i);
            break;
        }
        i += 1;
    }

    let Some(dot_pos) = dot_pos else {
        panic!("expected a version string in the form 'a.b'");
    };

    let major = get_single_val(buf, 0, dot_pos);
    let minor = get_single_val(buf, dot_pos + 1, buf.len());

    assert!(major <= 0xff);
    assert!(minor <= 0xff);

    ((major << 8) | minor) as c_uint
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_str() {
        assert_eq!(parse_version_str("1.2"), 0x0102);
        assert_eq!(parse_version_str("0.1"), 0x0001);
        assert_eq!(parse_version_str("100.255"), 0x64ff);
    }
}
