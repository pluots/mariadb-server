use core::ffi::c_int;

use crate::EmptyResult;

pub(crate) fn to_result(val: c_int) -> EmptyResult {
    if val == 0 {
        EmptyResult::Ok(())
    } else {
        EmptyResult::Err(())
    }
}
