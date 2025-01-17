//! Crate representing safe abstractions over MariaDB bindings
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::str_to_string)]
#![warn(clippy::cast_lossless)]
#![allow(unknown_lints)] // we build across multiple versions
#![allow(clippy::option_if_let_else)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::useless_conversion)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::manual_c_str_literals)]
#![allow(clippy::missing_inline_in_public_items)]

use std::io::Write;

mod my_alloc;
pub mod plugin;
#[cfg(feature = "service-sql")]
pub mod service_sql;
pub mod sql;
#[cfg(feature = "storage")]
pub mod storage;
mod table;
mod thd;
mod util;
#[cfg(feature = "service-sql")]
mod value;

#[cfg(test)]
mod tests;

pub use log;
#[doc(hidden)]
pub use mariadb_sys as bindings;
pub use my_alloc::MemRoot;
pub use table::TableShare;
#[cfg(test)]
use tests::assert_layouts_eq;
pub use thd::Thd;
#[cfg(feature = "service-sql")]
#[doc(inline)]
pub use value::*;

#[doc(hidden)]
pub mod internals {
    pub use cstr::cstr;

    pub use super::util::{parse_version_str, UnsafeSyncCell};
}

pub mod dbug {
    //! Instrumentation for MariaDB's `dbug` log and backtrace module.
    pub use mariadb_macros::dbug_instrument as instrument;
}

/// Our main logger config
///
/// Writes a timestamp, log level, and message. For debug & trace, also log the
/// file name.
///
/// Defaults to `info` level logging unless overridden by env
#[doc(hidden)]
pub fn build_logger() -> env_logger::Logger {
    let mut builder =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"));
    builder.format(log_formatter);
    builder.build()
}

fn log_formatter(f: &mut env_logger::fmt::Formatter, record: &log::Record) -> std::io::Result<()> {
    let t = time::OffsetDateTime::now_utc();
    let tfmt = time::format_description::parse(
        "[year]-[month]-[day] [hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]",
    )
    .unwrap();

    // Write the time
    t.format_into(f, &tfmt).map_err(|e| match e {
        time::error::Format::StdIo(io_e) => io_e,
        _ => panic!("{e}"),
    })?;

    let level = record.level();
    write!(f, " [{level}]")?;

    if let Some(modpath) = record.module_path() {
        // Write the crate name
        let first = modpath.split_once(':').map_or(modpath, |v| v.0);
        write!(f, " {first}")?;
    }

    writeln!(f, ": {}", record.args())
}

/// Configure the default logger. This is currently invoked by default as part of plugin registration.
#[macro_export]
macro_rules! configure_logger {
    () => {
        // Taken from `env_logger::Builder::try_init`. We duplicate it here so failures are OK
        let logger = $crate::build_logger();
        let max_level = logger.filter();
        let res = $crate::log::set_boxed_logger(Box::new(logger));
        if res.is_ok() {
            $crate::log::set_max_level(max_level);
        } else {
            eprintln!("logger already initialized at {}", module_path!());
        }
    };
}

/// Print a warning a maximum of once
#[macro_export]
macro_rules! warn_once {
    ($($tt:tt)*) => {
        static WARNED: ::std::sync::atomic::AtomicBool =
            ::std::sync::atomic::AtomicBool::new(false);
        let warned = WARNED.swap(true, ::std::sync::atomic::Ordering::Relaxed);
        if !warned {
            $crate::log::warn!($($tt)*)
        }
    }
}

/// Print an error a maximum of once
#[macro_export]
macro_rules! error_once {
    ($($tt:tt)*) => {
        static ERRORED: ::std::sync::atomic::AtomicBool =
            ::std::sync::atomic::AtomicBool::new(false);
        let errored = ERRORED.swap(true, ::std::sync::atomic::Ordering::Relaxed);
        if !errored {
            $crate::log::error!($($tt)*)
        }
    }
}

/// Provide the name of the calling function (full path)
#[allow(unused)]
macro_rules! function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}
