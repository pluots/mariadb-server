#![allow(dead_code)]

use std::cell::UnsafeCell;
use std::marker::PhantomData;

use crate::bindings;

#[repr(transparent)]
pub struct Table<'a> {
    share: UnsafeCell<bindings::TABLE>,
    phantom: PhantomData<&'a ()>,
}

impl<'a> Table<'a> {
    pub(crate) unsafe fn from_raw(tab: *const bindings::TABLE) -> &'a Self {
        unsafe { &*tab.cast() }
    }
}

/// A structure shared among all table objects. Has lots of internal locking.
#[repr(transparent)]
pub struct TableShare<'a> {
    share: UnsafeCell<bindings::TABLE_SHARE>,
    phantom: PhantomData<&'a ()>,
}

impl<'a> TableShare<'a> {
    pub(crate) unsafe fn from_raw(tab: *const bindings::TABLE_SHARE) -> &'a Self {
        unsafe { &*tab.cast() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout() {
        crate::assert_layouts_eq!(bindings::TABLE, Table);
        crate::assert_layouts_eq!(bindings::TABLE_SHARE, TableShare);
    }
}
