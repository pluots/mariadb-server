use std::cmp::max;
use std::ffi::{c_int, c_uint, c_ulong, CStr};
use std::marker::PhantomData;
use std::mem;
use std::path::Path;

use super::{Handlerton, StorageError, StorageResult, MAX_RECORD_LENGTH};
use crate::sql::{MAX_DATA_LENGTH_FOR_KEY, MAX_REFERENCE_PARTS};
use crate::{bindings, MemRoot, TableShare};

pub const IO_SIZE: usize = bindings::IO_SIZE as usize;

#[derive(Clone, Default)]
pub struct TableFlags(bindings::handler_Table_flags);
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
        self.0.io
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
        unsafe { mem::transmute(&self.inner.stats) }
    }
    fn costs(&self) -> &'a OptimizerCosts {
        unsafe { mem::transmute(&self.inner.costs) }
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

pub trait Handler: 'static {
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
