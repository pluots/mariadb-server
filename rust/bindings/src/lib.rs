//! Bindings module
//!
//! Autogenerated bindings to C interfaces for Rust
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::useless_transmute)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_safety_doc)]

// Bindings are autogenerated at build time using build.rs
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

mod cpp_classes;
mod hand_impls;

pub use hand_impls::*;
