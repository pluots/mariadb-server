use std::io;

use strum::{EnumIter, IntoEnumIterator};

use crate::bindings;

pub type StorageResult<T = ()> = Result<T, StorageError>;

/// Storage handler error types, as defined in `my_base.h`
#[derive(Debug, EnumIter)]
pub enum StorageError {
    /// Didn't find key on read or update
    KeyNotFound = bindings::HA_ERR_KEY_NOT_FOUND as isize,
    /// Duplicate key on write
    FoundDuppKey = bindings::HA_ERR_FOUND_DUPP_KEY as isize,
    /// Internal error
    InternalError = bindings::HA_ERR_INTERNAL_ERROR as isize,
    /// Update with is recoverable
    RecordChanged = bindings::HA_ERR_RECORD_CHANGED as isize,
    /// Wrong index given to function
    WrongIndex = bindings::HA_ERR_WRONG_INDEX as isize,
    /// Indexfile is crashed
    Crashed = bindings::HA_ERR_CRASHED as isize,
    /// Record-file is crashed or table is corrupt
    WrongInRecord = bindings::HA_ERR_WRONG_IN_RECORD as isize,
    /// Out of memory
    OutOfMemory = bindings::HA_ERR_OUT_OF_MEM as isize,
    /// Initialization failed and should be retried
    RetryInit = bindings::HA_ERR_RETRY_INIT as isize,
    /// not a MYI file - no signature
    NotAtAble = bindings::HA_ERR_NOT_A_TABLE as isize,
    /// Command not supported
    WrongCommand = bindings::HA_ERR_WRONG_COMMAND as isize,
    /// old databasfile
    OldFile = bindings::HA_ERR_OLD_FILE as isize,
    /// No record read in update()
    NoActiveRecord = bindings::HA_ERR_NO_ACTIVE_RECORD as isize,
    /// A record is not there
    RecordDeleted = bindings::HA_ERR_RECORD_DELETED as isize,
    /// No more room in file
    RecordFileFull = bindings::HA_ERR_RECORD_FILE_FULL as isize,
    /// No more room in file
    IndexFileFull = bindings::HA_ERR_INDEX_FILE_FULL as isize,
    /// end in next/prev/first/last
    EndOfFile = bindings::HA_ERR_END_OF_FILE as isize,
    /// unsupported extension used
    Unsupported = bindings::HA_ERR_UNSUPPORTED as isize,
    /// Too big row
    ToBigRow = bindings::HA_ERR_TO_BIG_ROW as isize,
    /// Wrong create option
    GCreateOption = bindings::HA_WRONG_CREATE_OPTION as isize,
    /// Duplicate unique on write
    FoundDuppUnique = bindings::HA_ERR_FOUND_DUPP_UNIQUE as isize,
    /// Can't open charset
    UnknownCharset = bindings::HA_ERR_UNKNOWN_CHARSET as isize,
    /// conflicting tables in MERGE
    WrongMrgTableDef = bindings::HA_ERR_WRONG_MRG_TABLE_DEF as isize,
    /// Last (automatic?) repair failed
    CrashedOnRepair = bindings::HA_ERR_CRASHED_ON_REPAIR as isize,
    /// Table must be repaired
    CrashedOnUsage = bindings::HA_ERR_CRASHED_ON_USAGE as isize,
    LockWaitTimeout = bindings::HA_ERR_LOCK_WAIT_TIMEOUT as isize,
    LockTableFull = bindings::HA_ERR_LOCK_TABLE_FULL as isize,
    /// Updates not allowed
    ReadOnlyTransaction = bindings::HA_ERR_READ_ONLY_TRANSACTION as isize,
    LockDeadlock = bindings::HA_ERR_LOCK_DEADLOCK as isize,
    /// Cannot add a foreign key constr.
    CannotAddForeign = bindings::HA_ERR_CANNOT_ADD_FOREIGN as isize,
    /// Cannot add a child row
    NoReferencedRow = bindings::HA_ERR_NO_REFERENCED_ROW as isize,
    /// Cannot delete a parent row
    RowIsReferenced = bindings::HA_ERR_ROW_IS_REFERENCED as isize,
    /// No savepoint with that name
    NoSavepoint = bindings::HA_ERR_NO_SAVEPOINT as isize,
    /// Non unique key block size
    NonUniqueBlockSize = bindings::HA_ERR_NON_UNIQUE_BLOCK_SIZE as isize,
    /// The table does not exist in engine
    NoSuchTable = bindings::HA_ERR_NO_SUCH_TABLE as isize,
    /// The table existed in storage engine
    TableExist = bindings::HA_ERR_TABLE_EXIST as isize,
    /// Could not connect to storage engine
    NoConnection = bindings::HA_ERR_NO_CONNECTION as isize,
    /// NULLs are not supported in spatial index
    NullInSpatial = bindings::HA_ERR_NULL_IN_SPATIAL as isize,
    /// The table changed in storage engine
    TableDefChanged = bindings::HA_ERR_TABLE_DEF_CHANGED as isize,
    /// There's no partition in table for given value
    NoPartitionFound = bindings::HA_ERR_NO_PARTITION_FOUND as isize,
    /// Row-based binlogging of row failed
    RbrLoggingFailed = bindings::HA_ERR_RBR_LOGGING_FAILED as isize,
    /// Index needed in foreign key constr
    DropIndexFk = bindings::HA_ERR_DROP_INDEX_FK as isize,
    /// Upholding foreign key constraints would lead to a duplicate key error in e other table.
    ForeignDuplicateKey = bindings::HA_ERR_FOREIGN_DUPLICATE_KEY as isize,
    /// The table changed in storage engine
    TableNeedsUpgrade = bindings::HA_ERR_TABLE_NEEDS_UPGRADE as isize,
    /// The table is not writable
    TableReadonly = bindings::HA_ERR_TABLE_READONLY as isize,
    /// Failed to get next autoinc value
    AutoincReadFailed = bindings::HA_ERR_AUTOINC_READ_FAILED as isize,
    /// Failed to set row autoinc value
    AutoincErange = bindings::HA_ERR_AUTOINC_ERANGE as isize,
    /// Generic error
    Generic = bindings::HA_ERR_GENERIC as isize,
    /// row not actually updated: new values same as the old values
    RecordIsTheSame = bindings::HA_ERR_RECORD_IS_THE_SAME as isize,
    /// It is not possible to log this statement
    LoggingImpossible = bindings::HA_ERR_LOGGING_IMPOSSIBLE as isize,
    /// The event was corrupt, leading to illegal data being read
    CorruptEvent = bindings::HA_ERR_CORRUPT_EVENT as isize,
    /// New file format
    NewFile = bindings::HA_ERR_NEW_FILE as isize,
    /// The event could not be processed. No other handler error happene.
    RowsEventApply = bindings::HA_ERR_ROWS_EVENT_APPLY as isize,
    /// Error during initialization
    Initialization = bindings::HA_ERR_INITIALIZATION as isize,
    /// File too short
    FileTooShort = bindings::HA_ERR_FILE_TOO_SHORT as isize,
    /// Wrong CRC on page
    WrongCrc = bindings::HA_ERR_WRONG_CRC as isize,
    /// oo many active concurrent transactions
    TooManyConcurrentTrxs = bindings::HA_ERR_TOO_MANY_CONCURRENT_TRXS as isize,
    /// There's no explicitly listed partition in table for the given value
    NotInLockPartitions = bindings::HA_ERR_NOT_IN_LOCK_PARTITIONS as isize,
    /// Index column length exceeds limit
    IndexColTooLong = bindings::HA_ERR_INDEX_COL_TOO_LONG as isize,
    /// Index corrupted
    IndexCorrupt = bindings::HA_ERR_INDEX_CORRUPT as isize,
    /// Undo log record too big
    UndoRecTooBig = bindings::HA_ERR_UNDO_REC_TOO_BIG as isize,
    /// Invalid InnoDB Doc ID
    InvalidDocid = bindings::HA_FTS_INVALID_DOCID as isize,
    // /// Table being used in foreign key check (disabled in `my_base.h`)
    // TableInFkCheck = bindings::HA_ERR_TABLE_IN_FK_CHECK as isize,
    /// The tablespace existed in storage engine
    TablespaceExists = bindings::HA_ERR_TABLESPACE_EXISTS as isize,
    /// Table has too many columns
    TooManyFields = bindings::HA_ERR_TOO_MANY_FIELDS as isize,
    /// Row in wrong partition
    RowInWrongPartition = bindings::HA_ERR_ROW_IN_WRONG_PARTITION as isize,
    RowNotVisible = bindings::HA_ERR_ROW_NOT_VISIBLE as isize,
    AbortedByUser = bindings::HA_ERR_ABORTED_BY_USER as isize,
    DiskFull = bindings::HA_ERR_DISK_FULL as isize,
    IncompatibleDefinition = bindings::HA_ERR_INCOMPATIBLE_DEFINITION as isize,
    /// Too many words in a phrase
    FtsTooManyWordsInPhrase = bindings::HA_ERR_FTS_TOO_MANY_WORDS_IN_PHRASE as isize,
    /// Table encrypted but decrypt failed
    DecryptionFailed = bindings::HA_ERR_DECRYPTION_FAILED as isize,
    /// FK cascade depth exceeded
    FkDepthExceeded = bindings::HA_ERR_FK_DEPTH_EXCEEDED as isize,
    /// Missing Tablespace
    TablespaceMissing = bindings::HA_ERR_TABLESPACE_MISSING as isize,
    SequenceInvalidData = bindings::HA_ERR_SEQUENCE_INVALID_DATA as isize,
    SequenceRunOut = bindings::HA_ERR_SEQUENCE_RUN_OUT as isize,
    CommitError = bindings::HA_ERR_COMMIT_ERROR as isize,
    PartitionList = bindings::HA_ERR_PARTITION_LIST as isize,
    NoEncryption = bindings::HA_ERR_NO_ENCRYPTION as isize,
}

/// A lot of storage errors are IO related. We provide an automated conversion that works with `?`.
impl From<io::Error> for StorageError {
    fn from(e: io::Error) -> Self {
        log::trace!("{e}"); // Caller probably logs the error but log it here just in case
        match e.kind() {
            io::ErrorKind::OutOfMemory => Self::OutOfMemory,
            _ => Self::InternalError,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_error_counts() {
        // Verify errors are synced between here and MDB
        // numbers 125 (unnamed) and 183 (`HA_ERR_TABLE_IN_FK_CHECK`) are missing
        const SKIPPED_ERR_COUNT: usize = 2;
        assert_eq!(
            StorageError::iter().count() + SKIPPED_ERR_COUNT,
            bindings::HA_ERR_ERRORS.try_into().unwrap()
        );
    }
}
