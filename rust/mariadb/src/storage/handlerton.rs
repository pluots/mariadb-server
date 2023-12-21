use super::Handler;
use crate::bindings;

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

// TODO: do we really have a `self`? I.e., can a `handlerton` contain arbitrary data?

/// A "handlerton" ("handler singleton") is the entrypoint for a storage engine handler.
///
/// This defines registration and creation information.
pub trait Handlerton {
    type Handler: Handler;
    /// A type of data that is stored during a savepoint.
    type SavePoint;

    /// Extensions of files created for a single table in the database directory
    /// (`datadir/db_name/`).
    const TABLEFILE_EXTENSIONS: &'static [&'static str] = &[];

    fn close_connection(&self, ctx: &HandlertonCtx) -> Result;
    fn kill_query(&self, ctx: &HandlertonCtx);

    // TODO: should Savepoint be its own trait?

    /// Create a new savepoint
    fn savepoint_set(&self, ctx: &HandlertonCtx) -> Result<Self::SavePoint>;
    /// Restore to a previous savepoint
    fn savepoint_rollback(&self, ctx: &HandlertonCtx, sv: &mut Self::SavePoint) -> Result;
    // TODO: should this be a const?
    fn savepoint_rollback_can_release_mdl(&self, ctx: &HandlertonCtx) -> bool;
    fn savepoint_release(&self, ctx: &HandlertonCtx, sv: &mut Self::SavePoint) -> Result;

    /// Perform the commit.
    ///
    /// If `is_true_commit` is false, we are in an end of statement within a transaction
    fn commit(&self, ctx: &HandlertonCtx, is_true_commit: bool) -> Result;
    fn commit_ordered(&self, ctx: &HandlertonCtx, is_true_commit: bool) -> Result;
    fn rollback(&self, ctx: &HandlertonCtx, is_true_commit: bool) -> Result;
    fn prepare(&self, ctx: &HandlertonCtx, is_true_commit: bool) -> Result;
    fn prepare_ordered(&self, ctx: &HandlertonCtx, is_true_commit: bool) -> Result;
}

// TODO: also take table_options, field_options, and index_options. Maybe we can put these
// into traits?
pub fn initialize_handlerton<T: Handlerton>(hton: &mut bindings::handlerton) {
    hton.kill_query = Some(wrap_kill_query::<T>);
}

unsafe extern "C" fn wrap_kill_query<H: Handlerton>(
    hton: *mut bindings::handlerton,
    thd: *mut bindings::THD,
    level: bindings::thd_kill_levels,
) {
    let ctx = unsafe { HandlertonCtx::new(hton, thd) };
    todo!()
}
