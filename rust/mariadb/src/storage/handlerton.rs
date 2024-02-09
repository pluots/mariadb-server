use super::Handler;
use crate::thd::ThdKillLevel;
use crate::{bindings, MemRoot, TableShare, Thd};

pub enum Error {}
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

pub struct HandlertonCtx<'a> {
    hton: &'a mut bindings::handlerton,
    thd: &'a mut bindings::THD,
}

impl<'a> HandlertonCtx<'a> {
    unsafe fn new(hton: *mut bindings::handlerton, thd: *mut bindings::THD) -> Self {
        debug_assert!(!hton.is_null());
        debug_assert!(!thd.is_null());
        Self {
            hton: unsafe { &mut *hton },
            thd: unsafe { &mut *thd },
        }
    }
}

pub struct HandlertonThd<'a> {
    thd: &'a mut Thd<'a>,
    slot: usize,
}

impl<'a> HandlertonThd<'a> {
    unsafe fn new(hton: *mut bindings::handlerton, thd: *mut bindings::THD) -> Self {
        debug_assert!(!hton.is_null());
        debug_assert!(!thd.is_null());
        Self {
            thd: unsafe { Thd::new_mut(thd) },
            slot: unsafe { (*hton).slot }.try_into().unwrap(),
        }
    }

    fn data(&self) {
        // let x = self.thd.0.ha_data[self.slot];
    }
}

// TODO: do we really have a `self`? I.e., can a `handlerton` contain arbitrary data?

/// A "handlerton" ("handler singleton") is the entrypoint for a storage engine handler.
///
/// This defines registration and creation information.
pub trait Handlerton: Send + Sync {
    type Handler: Handler;
    /// A type of data that is stored during a savepoint.
    type SavePoint;
    const FLAGS: u32 = 0;

    /// Extensions of files created for a single table in the database directory
    /// (`datadir/db_name/`).
    const TABLEFILE_EXTENSIONS: &'static [&'static str] = &[];

    // fn close_connection(thd: &HandlertonThd) -> Result;
    // fn kill_query(thd: &HandlertonThd, level: ThdKillLevel);

    // TODO: should Savepoint be its own trait?

    // /// Create a new savepoint
    // fn savepoint_set(thd: &HandlertonThd) -> Result<Self::SavePoint>;
    // /// Restore to a previous savepoint
    // fn savepoint_rollback(thd: &HandlertonThd, sv: &mut Self::SavePoint) -> Result;
    // fn savepoint_rollback_can_release_mdl(thd: &HandlertonThd) -> bool {
    //     false
    // }
    // fn savepoint_release(thd: &HandlertonThd, sv: &mut Self::SavePoint) -> Result;

    // /// Perform the commit.
    // ///
    // /// If `is_true_commit` is false, we are in an end of statement within a transaction
    // fn commit(thd: &HandlertonThd, is_true_commit: bool) -> Result;
    // fn commit_ordered(thd: &HandlertonThd, is_true_commit: bool) -> Result;
    // fn rollback(thd: &HandlertonThd, is_true_commit: bool) -> Result;
    // fn prepare(thd: &HandlertonThd, is_true_commit: bool) -> Result;
    // fn prepare_ordered(thd: &HandlertonThd, is_true_commit: bool) -> Result;

    //... more to do
}

// TODO: also take table_options, field_options, and index_options. Maybe we can put these
// into traits?
pub fn initialize_handlerton<T: Handlerton>(hton: &mut bindings::handlerton) {
    hton.kill_query = Some(wrap_kill_query::<T>);
}

#[allow(improper_ctypes_definitions)] // level is not FFI-safe
unsafe extern "C" fn wrap_kill_query<H: Handlerton>(
    hton: *mut bindings::handlerton,
    thd: *mut bindings::THD,
    level: bindings::thd_kill_levels::Type,
) {
    let ctx = unsafe { HandlertonCtx::new(hton, thd) };
    todo!()
}
