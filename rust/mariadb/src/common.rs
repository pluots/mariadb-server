use std::ffi::{c_char, c_long};
use std::{slice, ptr};

use crate::bindings;

/// A SQL type and value
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub enum Value<'a> {
    Decimal(&'a [u8]),
    I8(i8),
    I16(i16),
    Long(i64),
    LongLong(i64),
    F32(f32),
    F64(f64),
    Null,
    Time,      // todo
    TimeStamp, // todo
    Date,      // todo
    DateTime,  // todo
    Year,      // todo
    String(&'a [u8]),
    Blob(&'a [u8]),
    Json(&'a [u8]),
}

impl<'a> Value<'a> {
    /// Supply a
    pub(crate) unsafe fn from_ptr(
        ty: bindings::enum_field_types,
        ptr: *const c_char,
        len: usize,
    ) -> Self {
        // helper function to make a slice
        let buf_callback = || slice::from_raw_parts(ptr.cast(), len);
        dbg!(ty, ptr, len);

        match ty {
            bindings::enum_field_types::MYSQL_TYPE_DECIMAL => Self::Decimal(buf_callback()),
            // TODO: seems like we get unaligned pointers?
            // bindings::enum_field_types::MYSQL_TYPE_TINY => Self::Tiny(ptr::read_unaligned(ptr.cast())),
            // bindings::enum_field_types::MYSQL_TYPE_SHORT => Self::Short(ptr::read_unaligned(ptr.cast())),
            // bindings::enum_field_types::MYSQL_TYPE_LONG => Self::Long(ptr::read_unaligned(ptr.cast())),
            // bindings::enum_field_types::MYSQL_TYPE_FLOAT => Self::Float(ptr::read_unaligned(ptr.cast())),
            // bindings::enum_field_types::MYSQL_TYPE_DOUBLE => Self::Double(ptr::read_unaligned(ptr.cast())),
            bindings::enum_field_types::MYSQL_TYPE_TINY => Self::I8(*ptr.cast()),
            bindings::enum_field_types::MYSQL_TYPE_SHORT => Self::I16(*ptr.cast()),
            // This is yucky, `long` is `i32` on Windows but `i64` on nix. So, we load it as a `long` but
            // always store it as `i64`.
            bindings::enum_field_types::MYSQL_TYPE_LONG => Self::Long((*ptr.cast::<c_long>()).into()),
            bindings::enum_field_types::MYSQL_TYPE_LONGLONG => Self::LongLong(*ptr.cast()),
            bindings::enum_field_types::MYSQL_TYPE_FLOAT => Self::F32(*ptr.cast()),
            bindings::enum_field_types::MYSQL_TYPE_DOUBLE => Self::F64(*ptr.cast()),
            bindings::enum_field_types::MYSQL_TYPE_NULL => Self::Null,
            bindings::enum_field_types::MYSQL_TYPE_TIMESTAMP => todo!(),
            bindings::enum_field_types::MYSQL_TYPE_INT24 => todo!(),
            bindings::enum_field_types::MYSQL_TYPE_DATE => todo!(),
            bindings::enum_field_types::MYSQL_TYPE_TIME => todo!(),
            bindings::enum_field_types::MYSQL_TYPE_DATETIME => todo!(),
            bindings::enum_field_types::MYSQL_TYPE_YEAR => todo!(),
            bindings::enum_field_types::MYSQL_TYPE_NEWDATE => todo!(),
            bindings::enum_field_types::MYSQL_TYPE_VARCHAR => todo!(),
            bindings::enum_field_types::MYSQL_TYPE_BIT => todo!(),
            bindings::enum_field_types::MYSQL_TYPE_TIMESTAMP2 => todo!(),
            bindings::enum_field_types::MYSQL_TYPE_DATETIME2 => todo!(),
            bindings::enum_field_types::MYSQL_TYPE_TIME2 => todo!(),
            bindings::enum_field_types::MYSQL_TYPE_BLOB_COMPRESSED => todo!(),
            bindings::enum_field_types::MYSQL_TYPE_VARCHAR_COMPRESSED => todo!(),
            bindings::enum_field_types::MYSQL_TYPE_NEWDECIMAL => todo!(),
            bindings::enum_field_types::MYSQL_TYPE_ENUM => todo!(),
            bindings::enum_field_types::MYSQL_TYPE_SET => todo!(),
            bindings::enum_field_types::MYSQL_TYPE_TINY_BLOB => Self::Blob(buf_callback()),
            bindings::enum_field_types::MYSQL_TYPE_MEDIUM_BLOB => Self::Blob(buf_callback()),
            bindings::enum_field_types::MYSQL_TYPE_LONG_BLOB => Self::Blob(buf_callback()),
            bindings::enum_field_types::MYSQL_TYPE_BLOB => Self::Blob(buf_callback()),
            bindings::enum_field_types::MYSQL_TYPE_VAR_STRING => Self::String(buf_callback()),
            bindings::enum_field_types::MYSQL_TYPE_STRING => Self::String(buf_callback()),
            bindings::enum_field_types::MYSQL_TYPE_GEOMETRY => todo!(),
            _ => todo!(),
        }
    }
}
