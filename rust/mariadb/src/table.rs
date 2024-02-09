//! Things related to tables
//!
//! # Basic Design
//!
//! MariaDB/MySQL use huge classes like `TABLE` and `TABLE_SHARE` to represent all kinds of
//! information for various different states. The problem here is that you more or less have to
//! reverse engineer every possible usage of these types to make sure you don't use them
//! incorrectly. Incorrect use would be something like calling a method during `CREATE TABLE` that
//! assumes some variable is initialized, but this variable does not actually get initialized
//! until an index is created.
//!
//! Part of my goal with creating this Rust API is to "make incorrect things impossible", which
//! hopefully means that the barrier to entry for writing a storage engine will be significantly
//! lowered.
//!
//! To do this, I use types implementing the [`TableState`] marker trait to implement something
//! like the consumer side of a state machine. All [`Table`] or [`TableShare`] instances have
//! the exact same in-memory representation (their equivalent C++ types) but certain methods will
//! only be available for certain states.

#![allow(dead_code)]

use std::cell::UnsafeCell;
use std::marker::PhantomData;

use crate::{bindings, Sealed};

pub trait TableState: Sealed {}

pub mod state {
    use crate::sealed::Sealed;
    use crate::TableState;

    /// State of a table during creation
    pub struct Create;
    impl Sealed for Create {}
    impl TableState for Create {}
}

#[repr(transparent)]
pub struct Table<'a, S: TableState> {
    table: UnsafeCell<bindings::TABLE>,
    phantom: PhantomData<&'a ()>,
    state: PhantomData<S>,
}

impl<'a, S: TableState> Table<'a, S> {
    pub(crate) unsafe fn from_raw(tab: *const bindings::TABLE) -> &'a Self {
        unsafe { &*tab.cast() }
    }

    fn share(&self) -> &TableShare<S> {
        unsafe { TableShare::from_raw((*self.table.get()).s) }
    }
}

/// A structure shared among all table objects. Has lots of internal locking.
#[repr(transparent)]
pub struct TableShare<'a, S: TableState> {
    share: UnsafeCell<bindings::TABLE_SHARE>,
    phantom: PhantomData<&'a ()>,
    state: PhantomData<S>,
}

impl<'a, S: TableState> TableShare<'a, S> {
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
