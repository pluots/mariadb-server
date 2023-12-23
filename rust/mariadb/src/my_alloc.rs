#![allow(dead_code)]

use std::marker::PhantomData;

use crate::bindings;

pub struct MemRoot<'a> {
    root: bindings::MEM_ROOT,
    phantom: PhantomData<&'a ()>,
}
