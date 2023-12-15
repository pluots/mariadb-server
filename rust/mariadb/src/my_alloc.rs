#![allow(dead_code)]

use crate::bindings;

pub struct MemRoot<'a> {
    root: &'a bindings::MEM_ROOT,
}

impl<'a> MemRoot<'a> {
    pub(crate) unsafe fn from_raw(root: *mut bindings::MEM_ROOT) -> Self {
        Self {
            root: unsafe { &*root },
        }
    }
}
