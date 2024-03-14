#![allow(clippy::cast_precision_loss)]

use std::cmp::max;
use std::ffi::{c_int, c_uint, c_ulong, CStr};
use std::marker::PhantomData;
use std::ops::BitOr;
use std::path::Path;
use std::{mem, ptr};

use super::{Handlerton, StorageError, StorageResult, MAX_RECORD_LENGTH};
use crate::sql::{MAX_DATA_LENGTH_FOR_KEY, MAX_REFERENCE_PARTS};
use crate::{bindings, MemRoot, TableShare};

pub const IO_SIZE: usize = bindings::IO_SIZE as usize;

/// A type that shows what the storage engine can do
#[derive(Clone, Copy, Debug, Default)]
pub struct TableFlags(bindings::handler_Table_flags);

impl TableFlags {
    // FIXME: we may be able to infer a lot of these (e.g. repair table) by splitting their
    // required functions to a separate trait.

    /// Transactions are not supported
    pub const NO_TRANSACTIONS: Self = Self(bindings::HA_NO_TRANSACTIONS as u64);

    /// Read may not return all columns
    pub const PARTIAL_COLUMN_READ: Self = Self(bindings::HA_PARTIAL_COLUMN_READ as u64);
    /// Data and index are in separate files
    pub const TABLE_SCAN_ON_INDEX: Self = Self(bindings::HA_TABLE_SCAN_ON_INDEX as u64);

    /// Indicate that a record may not be in squence.
    ///
    /// This should be set if the following is not true when scanning
    /// a table with rnd_next()
    /// - We will see all rows (including deleted ones)
    /// - Row positions are 'table->s->db_record_offset' apart
    /// If this flag is not set, filesort will do a position() call for each matched
    /// row to be able to find the row later.
    pub const REC_NOT_IN_SEQ: Self = Self(bindings::HA_REC_NOT_IN_SEQ as u64);

    pub const CAN_GEOMETRY: Self = Self(bindings::HA_CAN_GEOMETRY as u64);

    /// Reading keys in random order is as fast as reading keys in sort order
    ///
    /// Used in records.cc to decide if we should use a record cache and by filesort to decide
    /// if we should sort key + data or key + pointer-to-row
    pub const FAST_KEY_READ: Self = Self(bindings::HA_FAST_KEY_READ as u64);

    /// Deletes should force all key to be read and updates should read all changed keys
    pub const REQUIRES_KEY_COLUMNS_FOR_DELETE: Self =
        Self(bindings::HA_REQUIRES_KEY_COLUMNS_FOR_DELETE as u64);

    /// One can have keys with NULL
    pub const NULL_IN_KEY: Self = Self(bindings::HA_NULL_IN_KEY as u64);

    /// `ha_position()` gives a duplicate row
    pub const DUPLICATE_POS: Self = Self(bindings::HA_DUPLICATE_POS as u64);

    /// Doesn't support blobs
    pub const NO_BLOBS: Self = Self(bindings::HA_NO_BLOBS as u64);

    /// Blobs can be added to the index
    pub const CAN_INDEX_BLOBS: Self = Self(bindings::HA_CAN_INDEX_BLOBS as u64);

    /// auto-increment in multi-part key
    pub const AUTO_PART_KEY: Self = Self(bindings::HA_AUTO_PART_KEY as u64);

    /// The engine requires every table to have a user-specified PRIMARY KEY.
    ///
    /// Do not set the flag if the engine can generate a hidden primary key internally.
    /// This flag is ignored if a SEQUENCE is created (which, in turn, needs the
    /// [`CAN_TABLES_WITHOUT_ROLLBACK`] flag).
    pub const REQUIRE_PRIMARY_KEY: Self = Self(bindings::HA_REQUIRE_PRIMARY_KEY as u64);

    pub const STATS_RECORDS_IS_EXACT: Self = Self(bindings::HA_STATS_RECORDS_IS_EXACT as u64); /* stats.records is exact */

    /// INSERT_DELAYED only works with handlers that uses MySQL internal table
    /// level locks
    pub const CAN_INSERT_DELAYED: Self = Self(bindings::HA_CAN_INSERT_DELAYED as u64);

    /// The primary key is part of the read index
    ///
    /// If we get the primary key columns for free when we do an index read
    /// (usually, it also implies that the [`PRIMARY_KEY_REQUIRED_FOR_POSITION`]
    /// flag is set).
    pub const PRIMARY_KEY_IN_READ_INDEX: Self = Self(bindings::HA_PRIMARY_KEY_IN_READ_INDEX as u64);

    /// [`position()`] uses a primary key given by the record argument.
    ///
    /// Without primary key, we can't call [`position()`].
    /// If not set, the position is returned as the current rows position
    /// regardless of what argument is given.
    pub const PRIMARY_KEY_REQUIRED_FOR_POSITION: Self =
        Self(bindings::HA_PRIMARY_KEY_REQUIRED_FOR_POSITION as u64);

    pub const CAN_RTREEKEYS: Self = Self(bindings::HA_CAN_RTREEKEYS as u64);
    pub const NOT_DELETE_WITH_CACHE: Self = Self(bindings::HA_NOT_DELETE_WITH_CACHE as u64); /* unused */

    /// A primary key is needed to delete and update a row.
    ///
    /// If there is no primary key, all columns needs to be read on update and delete
    pub const PRIMARY_KEY_REQUIRED_FOR_DELETE: Self =
        Self(bindings::HA_PRIMARY_KEY_REQUIRED_FOR_DELETE as u64);

    pub const NO_PREFIX_CHAR_KEYS: Self = Self(bindings::HA_NO_PREFIX_CHAR_KEYS as u64);

    pub const CAN_FULLTEXT: Self = Self(bindings::HA_CAN_FULLTEXT as u64);

    pub const CAN_SQL_HANDLER: Self = Self(bindings::HA_CAN_SQL_HANDLER as u64);

    pub const NO_AUTO_INCREMENT: Self = Self(bindings::HA_NO_AUTO_INCREMENT as u64);

    /// Has automatic checksums and uses the old checksum format
    pub const HAS_OLD_CHECKSUM: Self = Self(bindings::HA_HAS_OLD_CHECKSUM as u64);

    /// Table data are stored in separate files (for lower_case_table_names)
    pub const FILE_BASED: Self = Self(bindings::HA_FILE_BASED as u64);

    pub const CAN_BIT_FIELD: Self = Self(bindings::HA_CAN_BIT_FIELD as u64); /* supports bit fields */

    pub const NEED_READ_RANGE_BUFFER: Self = Self(bindings::HA_NEED_READ_RANGE_BUFFER as u64); /* for read_multi_range */

    pub const ANY_INDEX_MAY_BE_UNIQUE: Self = Self(bindings::HA_ANY_INDEX_MAY_BE_UNIQUE as u64);

    pub const NO_COPY_ON_ALTER: Self = Self(bindings::HA_NO_COPY_ON_ALTER as u64);

    pub const HAS_RECORDS: Self = Self(bindings::HA_HAS_RECORDS as u64); /* records() gives exact count*/

    /// Has its own method of binlog logging
    pub const HAS_OWN_BINLOGGING: Self = Self(bindings::HA_HAS_OWN_BINLOGGING as u64);

    /// Engine is capable of row-format logging
    pub const BINLOG_ROW_CAPABLE: Self = Self(bindings::HA_BINLOG_ROW_CAPABLE as u64);

    /// Engine is capable of statement-format logging
    pub const BINLOG_STMT_CAPABLE: Self = Self(bindings::HA_BINLOG_STMT_CAPABLE as u64);

    /// Duplicate keys may not be issued in order
    ///
    /// When a multiple key conflict happens in a REPLACE command, MariaDB expects the conflicts
    /// to be reported in the ascending order of key names.
    ///
    /// For e.g.
    ///
    /// ```sql
    /// CREATE TABLE t1 (a INT, UNIQUE (a), b INT NOT NULL, UNIQUE (b), c INT NOT
    ///                  NULL, INDEX(c));
    ///
    /// REPLACE INTO t1 VALUES (1,1,1), (2,2,2), (2,1,3);
    /// ```
    ///
    /// MySQL expects the conflict with 'a' to be reported before the conflict with 'b'.
    ///
    /// If the underlying storage engine does not report the conflicting keys in
    /// ascending order, it causes unexpected errors when the REPLACE command is
    /// executed.
    ///
    /// This flag helps the underlying storage engine to inform the server that the keys are not
    /// ordered.
    pub const DUPLICATE_KEY_NOT_IN_ORDER: Self =
        Self(bindings::HA_DUPLICATE_KEY_NOT_IN_ORDER as u64);

    /*
      Engine supports REPAIR TABLE. Used by CHECK TABLE FOR UPGRADE if an
      incompatible table is detected. If this flag is set, CHECK TABLE FOR UPGRADE
      will report ER_TABLE_NEEDS_UPGRADE, otherwise ER_TABLE_NEED_REBUILD.
    */
    pub const CAN_REPAIR: Self = Self(bindings::HA_CAN_REPAIR as u64);

    /* Has automatic checksums and uses the new checksum format */
    pub const HAS_NEW_CHECKSUM: Self = Self(bindings::HA_HAS_NEW_CHECKSUM as u64);
    pub const CAN_VIRTUAL_COLUMNS: Self = Self(bindings::HA_CAN_VIRTUAL_COLUMNS as u64);
    pub const MRR_CANT_SORT: Self = Self(bindings::HA_MRR_CANT_SORT as u64);
    /* All of VARCHAR is stored, including bytes after real varchar data */
    pub const RECORD_MUST_BE_CLEAN_ON_WRITE: Self =
        Self(bindings::HA_RECORD_MUST_BE_CLEAN_ON_WRITE as u64);

    /// Supports condition pushdown
    pub const CAN_TABLE_CONDITION_PUSHDOWN: Self =
        Self(bindings::HA_CAN_TABLE_CONDITION_PUSHDOWN as u64);

    /// The handler supports read before write removal optimization
    ///
    /// Read before write removal may be used for storage engines which support
    /// write without previous read of the row to be updated. Handler returning
    /// this flag must implement start_read_removal(HA_MUST_USE_TABLE_CONDITION_PUSHDOWN);.
    /// The handler may return "fake" rows constructed from the key of the row
    /// asked for. This is used to optimize UPDATE and DELETE by reducing the
    /// number of roundtrips between handler and storage engine.
    ///
    /// Example:
    ///
    /// ```sql
    /// UPDATE a=1 WHERE pk IN (<keys>)
    /// ```
    ///
    /// ```c
    /// Sql_cmd_update::update_single_table()
    /// {
    /// if (<conditions for starting read removal>)
    ///   start_read_removal()
    ///   -> handler returns true if read removal supported for this table/query
    ///
    /// while(read_record("pk=<key>"))
    ///   -> handler returns fake row with column "pk" set to <key>
    ///
    ///   ha_update_row()
    ///   -> handler sends write "a=1" for row with "pk=<key>"
    ///
    /// end_read_removal()
    /// -> handler returns the number of rows actually written
    /// }
    /// ```
    ///
    /// *Note*: This optimization in combination with batching may be used to
    /// remove even more roundtrips.
    pub const READ_BEFORE_WRITE_REMOVAL: Self = Self(bindings::HA_READ_BEFORE_WRITE_REMOVAL as u64);

    /// Engine supports extended fulltext API
    pub const CAN_FULLTEXT_EXT: Self = Self(bindings::HA_CAN_FULLTEXT_EXT as u64);

    /*
     Storage engine supports table export using the
     FLUSH TABLE <table_list> FOR EXPORT statement
     (meaning, after this statement one can copy table files out of the
     datadir and later "import" (somehow) in another MariaDB instance)
    */
    pub const CAN_EXPORT: Self = Self(bindings::HA_CAN_EXPORT as u64);

    /*
      Storage engine does not require an exclusive metadata lock
      on the table during optimize. (TODO and repair?).
      It can allow other connections to open the table.
      (it does not necessarily mean that other connections can
      read or modify the table - this is defined by THR locks and the
      ::store_lock() method).
    */
    pub const CONCURRENT_OPTIMIZE: Self = Self(bindings::HA_CONCURRENT_OPTIMIZE as u64);

    /*
      If the storage engine support tables that will not roll back on commit
      In addition the table should not lock rows and support READ and WRITE
      UNCOMMITTED.
      This is useful for implementing things like SEQUENCE but can also in
      the future be useful to do logging that should never roll back.
    */
    pub const CAN_TABLES_WITHOUT_ROLLBACK: Self =
        Self(bindings::HA_CAN_TABLES_WITHOUT_ROLLBACK as u64);

    /*
      Mainly for usage by SEQUENCE engine. Setting this flag means
      that the table will never roll back and that all operations
      for this table should stored in the non transactional log
      space that will always be written, even on rollback.
    */

    pub const PERSISTENT_TABLE: Self = Self(bindings::HA_PERSISTENT_TABLE as u64);

    /*
      If storage engine uses another engine as a base
      This flag is also needed if the table tries to open the .frm file
      as part of drop table.
    */
    pub const REUSES_FILE_NAMES: Self = Self(bindings::HA_REUSES_FILE_NAMES as u64);

    /*
     Set of all binlog flags. Currently only contain the capabilities
     flags.
    */
    pub const BINLOG_FLAGS: Self = Self(bindings::HA_BINLOG_FLAGS as u64);

    /* The following are used by Spider */
    pub const CAN_FORCE_BULK_UPDATE: Self = Self(bindings::HA_CAN_FORCE_BULK_UPDATE as u64);
    pub const CAN_FORCE_BULK_DELETE: Self = Self(bindings::HA_CAN_FORCE_BULK_DELETE as u64);
    pub const CAN_DIRECT_UPDATE_AND_DELETE: Self =
        Self(bindings::HA_CAN_DIRECT_UPDATE_AND_DELETE as u64);

    /* The following is for partition handler */
    pub const CAN_MULTISTEP_MERGE: Self = Self(bindings::HA_CAN_MULTISTEP_MERGE as u64);

    /* calling cmp_ref() on the engine is expensive */
    pub const SLOW_CMP_REF: Self = Self(bindings::HA_SLOW_CMP_REF as u64);
    pub const CMP_REF_IS_EXPENSIVE: Self = Self(bindings::HA_SLOW_CMP_REF as u64);

    /// `rnd_pos` is a slow operation
    ///
    /// Some engines are unable to provide an efficient implementation for rnd_pos().
    /// Server will try to avoid it, if possible
    pub const SLOW_RND_POS: Self = Self(bindings::HA_SLOW_RND_POS as u64);

    /* Safe for online backup */
    pub const CAN_ONLINE_BACKUPS: Self = Self(bindings::HA_CAN_ONLINE_BACKUPS as u64);

    /* Support native hash index */
    pub const CAN_HASH_KEYS: Self = Self(bindings::HA_CAN_HASH_KEYS as u64);
    pub const CRASH_SAFE: Self = Self(bindings::HA_CRASH_SAFE as u64);

    /*
     There is no need to evict the table from the table definition cache having
     run ANALYZE TABLE on it
    */
    pub const ONLINE_ANALYZE: Self = Self(bindings::HA_ONLINE_ANALYZE as u64);
    /*
      Rowid's are not comparable. This is set if the rowid is unique to the
      current open handler, like it is with federated where the rowid is a
      pointer to a local result set buffer. The effect of having this set is
      that the optimizer will not consider the following optimizations for
      the table:
      ror scans, filtering or duplicate weedout
    */
    pub const NON_COMPARABLE_ROWID: Self = Self(bindings::HA_NON_COMPARABLE_ROWID as u64);

    /* Implements SELECT ... FOR UPDATE SKIP LOCKED */
    pub const CAN_SKIP_LOCKED: Self = Self(bindings::HA_CAN_SKIP_LOCKED as u64);

    /* This engine is not compatible with Online ALTER TABLE */
    pub const NO_ONLINE_ALTER: Self = Self(bindings::HA_NO_ONLINE_ALTER as u64);

    pub const LAST_TABLE_FLAG: Self = Self(bindings::HA_NO_ONLINE_ALTER as u64);
}

impl BitOr for TableFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

#[derive(Clone, Default)]
pub struct IndexFlags(c_ulong);

#[derive(Debug)]
pub struct IoAndCpuCost(bindings::IO_AND_CPU_COST);

impl IoAndCpuCost {
    pub fn new(io: f64, cpu: f64) -> Self {
        Self(bindings::IO_AND_CPU_COST { io, cpu })
    }

    pub fn io(&self) -> f64 {
        self.0.io
    }

    pub fn cpu(&self) -> f64 {
        self.0.cpu
    }
}

#[repr(transparent)]
pub struct CreateInfo<'a> {
    inner: bindings::HA_CREATE_INFO,
    phantom: PhantomData<&'a ()>,
}

impl<'a> CreateInfo<'a> {
    pub(crate) unsafe fn from_raw(ptr: *const bindings::HA_CREATE_INFO) -> &'a Self {
        unsafe { &*ptr.cast() }
    }
}

#[repr(transparent)]
pub struct HandlerCtx<'a> {
    inner: bindings::handler,
    phantom: PhantomData<&'a ()>,
}

impl<'a> HandlerCtx<'a> {
    fn stats(&self) -> &'a Statistics {
        unsafe { *ptr::addr_of!(self.inner.stats).cast() }
    }
    fn costs(&self) -> &'a OptimizerCosts {
        unsafe { *ptr::addr_of!(self.inner.costs).cast() }
    }
}

#[repr(transparent)]
pub struct Statistics(bindings::ha_statistics);

impl Statistics {
    pub fn data_file_length(&self) -> usize {
        self.0.data_file_length.try_into().unwrap()
    }
    pub fn max_data_file_length(&self) -> usize {
        self.0.max_data_file_length.try_into().unwrap()
    }
    pub fn index_file_length(&self) -> usize {
        self.0.index_file_length.try_into().unwrap()
    }
    pub fn max_index_file_length(&self) -> usize {
        self.0.max_index_file_length.try_into().unwrap()
    }
    pub fn delete_length(&self) -> usize {
        self.0.delete_length.try_into().unwrap()
    }
    pub fn auto_increment_value(&self) -> usize {
        self.0.auto_increment_value.try_into().unwrap()
    }
    pub fn records(&self) -> usize {
        self.0.records.try_into().unwrap()
    }
    pub fn deleted(&self) -> usize {
        self.0.deleted.try_into().unwrap()
    }
    pub fn mean_rec_length(&self) -> usize {
        self.0.mean_rec_length.try_into().unwrap()
    }
    pub fn block_size(&self) -> usize {
        self.0.block_size.try_into().unwrap()
    }
    pub fn checksum(&self) -> u32 {
        self.0.checksum
    }
}

#[repr(transparent)]
pub struct OptimizerCosts(bindings::OPTIMIZER_COSTS);

impl OptimizerCosts {
    fn index_block_copy_cost(&self) -> f64 {
        self.0.index_block_copy_cost
    }
}

#[repr(transparent)]
pub struct Mode(c_int);

#[repr(transparent)]
pub struct OpenOp(c_uint);

/// A single instance of a table interface.
///
/// A handler may be closed and opened within the life of a server, but usually it persists
/// across multiple connections.
pub trait Handler: 'static {
    /// The related [`Handlerton`], which is used to build this `Handler`.
    type Handlerton: Handlerton;

    /// Set this to true if index support is available. If so, this type must
    /// also implement `IndexHandler`.
    const SUPPORTS_INDEX: bool = false;

    // const TABLE_FLAGS: TableFlags = TableFlags(0);
    const MAX_SUPPORTED_RECORD_LENGTH: usize = bindings::HA_MAX_REC_LENGTH as usize;

    /// Create a new handler
    ///
    /// # When is this called?
    ///
    /// - Every time there is a new connection
    fn new(table: &TableShare, mem_root: MemRoot) -> Self;

    /// Open a table, the name will be the name of the file.
    ///
    /// Closing happens by dropping this item.
    ///
    /// # When is this called?
    ///
    /// Not for every request, more to come...
    // TODO: figure out the interaction with ::extra in the C docs, maybe refactor
    fn open(name: &Path, mode: Mode, open_options: OpenOp) -> StorageResult;

    /// Create a new table and exit
    ///
    /// This should
    ///
    /// # When is this called?
    ///
    /// - SQL `CREATE TABLE` statements
    fn create(&self, name: &CStr, form: TableShare, create_info: &CreateInfo) {}

    fn table_flags(&self) -> TableFlags {
        TableFlags::default()
    }

    /// This is an INSERT statement
    fn write_row(buf: &CStr) {}

    fn update_row() {}

    fn delete_row() {}

    /// Delete all rows
    ///
    /// ## When is this called?
    ///
    /// - SQL `DELETE` with no `WHERE` clause
    fn delete_all_rows() {}

    fn store_lock(&self) {}

    /// Time for a full table data scan
    fn scan_time(&self, ctx: HandlerCtx) -> IoAndCpuCost {
        // Duplicated from handler.h
        let length = ctx.stats().data_file_length() as f64;
        let io = length / (IO_SIZE as f64);
        let bsize = ctx.stats().block_size() as f64;
        let cpu = ((length + bsize - 1.0) / bsize) + ctx.costs().index_block_copy_cost();
        let cpu = cpu.clamp(0.0, 1e200);
        IoAndCpuCost::new(io, cpu)
    }

    /// Cost of fetching `rows` records through `rnd_pos`.
    fn rnd_pos_time(&self, ctx: HandlerCtx, rows: usize) -> IoAndCpuCost {
        let r = rows as f64;
        let io = ((ctx.stats().block_size() + IO_SIZE).saturating_sub(1) as f64) / (IO_SIZE as f64);
        let cpu = r + ctx.costs().index_block_copy_cost();

        IoAndCpuCost::new(io, cpu)
    }
}

pub trait RepairingHandler: Handler {
    fn pre_calculate_checksum(&self) -> u32 {
        0
    }

    fn calculate_checksum(&self) -> u32 {
        0
    }

    fn is_crashed(&self) -> bool {
        false
    }

    fn auto_repair(error: i32) -> bool {
        false
    }
}

pub trait IndexableHandler: Handler {
    fn max_supported_record_length(&self) -> usize {
        MAX_RECORD_LENGTH
    }
    fn max_supported_keys(&self) -> usize {
        0
    }

    fn max_supported_key_parts(&self) -> usize {
        MAX_REFERENCE_PARTS
    }

    fn max_supported_key_length(&self) -> usize {
        MAX_DATA_LENGTH_FOR_KEY
    }

    /// Return the display name of the index, if any
    fn index_type(&self, index_number: usize) -> Option<&'static CStr> {
        None
    }

    fn index_flags(&self, index: usize, part: usize, all_parts: bool) -> IndexFlags {
        IndexFlags::default()
    }

    /// Calculate the cost of `index_only` scan for a given index, a number of ranges,
    /// and a number of records.
    ///
    /// - `index`: the index to read
    /// - `rows`: number of records to read
    /// - `blocks`: number of IO blocks that need to be accessed, or 0 if not known.
    fn keyread_time(
        &self,
        ctx: &HandlerCtx,
        index: usize,
        ranges: usize,
        rows: usize,
        blocks: usize,
    ) -> IoAndCpuCost {
        todo!()
    }

    /// Time for a full table index scan without copy or compare cost.
    fn key_scan_time(&self, ctx: &HandlerCtx, index: usize, rows: usize) -> IoAndCpuCost {
        Self::keyread_time(self, ctx, index, 1, max(rows, 1), 0)
    }

    fn index_read_map() {}
    fn index_next() {}
    fn index_prev() {}
    fn index_first() {}
    fn index_last() {}
}

pub trait InplaceAlterTable {}

// fn foo<T>(a: Box<dyn Handler<Handlerton = T>>) {
//     todo!()
// }
// fn bar<T>(a: Box<dyn IndexHandler<Handlerton = T>>) {
// todo!()
// }
