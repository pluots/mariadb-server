#![allow(dead_code)]

use std::marker::PhantomData;

use crate::bindings;

#[repr(transparent)]
pub struct Table<'a> {
    tab: bindings::TABLE_SHARE,
    phantom: PhantomData<&'a ()>,
}
