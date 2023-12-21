//! Interfaces for storage engines
//!
//! Most of this is for plugin use

mod handler;
mod handlerton;

pub use handler::Handler;
pub use handlerton::{Handlerton, HandlertonCtx};
