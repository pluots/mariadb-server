use crate::bindings;

/// Max parts used as ref
pub const MAX_REFERENCE_PARTS: usize = bindings::MAX_REF_PARTS as usize;

/// Maximum length of the data portion of an index lookup key
pub const MAX_DATA_LENGTH_FOR_KEY: usize = bindings::MAX_DATA_LENGTH_FOR_KEY as usize;
