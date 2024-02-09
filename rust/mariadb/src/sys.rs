#![allow(dead_code)]

/// A nice little helper trait that lets us perform various actions with errors
pub trait ResultExt {
    /// Bring down the server if this error occurs
    fn err_fatal(self) -> Self;

    /// Log the error and continue
    fn err_log(self) -> Self;

    /// Log the error as a earning and continue
    fn err_warn(self) -> Self;
}

impl<T, E: std::error::Error> ResultExt for Result<T, E> {
    /// Bring down the server if this error occurs
    fn err_fatal(self) -> Self {
        if let Err(e) = self {
            log::error!("FATAL!!! {e}");
            panic!("shutting down due to previous fatal error");
        };
        self
    }

    /// Log the error and continue
    fn err_log(self) -> Self {
        if let Err(e) = &self {
            log::error!("{e}");
        };
        self
    }

    /// Log the error as a earning and continue
    fn err_warn(self) -> Self {
        if let Err(e) = &self {
            log::warn!("{e}");
        };
        self
    }
}
