#![allow(dead_code)]

use std::marker::PhantomData;

use crate::bindings;

#[repr(transparent)]
pub struct Thd<'a>(pub(crate) bindings::THD, PhantomData<&'a ()>);

impl<'a> Thd<'a> {
    pub(crate) unsafe fn new_mut(ptr: *mut bindings::THD) -> &'a mut Self {
        unsafe { &mut *ptr.cast::<Self>() }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ThdKillLevel {
    /// No action needed
    NotKilled = bindings::thd_kill_levels::THD_IS_NOT_KILLED as isize,
    /// Abort when possible, don't corrupt any data
    AbortSoftly = bindings::thd_kill_levels::THD_ABORT_SOFTLY as isize,
    /// Abort as soon as possible
    AbortAsap = bindings::thd_kill_levels::THD_ABORT_ASAP as isize,
}
