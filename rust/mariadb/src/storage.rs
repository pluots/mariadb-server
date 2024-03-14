//! Interfaces for storage engines
//!
//! Most of this is for plugin use

#![allow(unused)]

mod error;
mod handler;
mod handlerton;

pub use error::{StorageError, StorageResult};
pub use handler::{Handler, Mode, OpenOp, TableFlags};
pub use handlerton::{Handlerton, HandlertonCtx};

use crate::bindings;

pub const MAX_RECORD_LENGTH: usize = bindings::HA_MAX_REC_LENGTH as usize;
