#![allow(unused)]

use std::cell::UnsafeCell;
use std::cmp::Ordering;
use std::marker::PhantomData;

use mariadb_sys as bindings;

use crate::util::to_result;

/// Generic error emitted by ftparser functions
#[derive(Default)]
pub struct FtError;

// TODO: maybe these should take the &[u8] plus some other type that we design
// instead of passing Parameters directly.
//
// Also, I think I should make a separate parse_full_boolean method
pub trait FullTextParser<'a> {
    /// Initialize the full text parser for first use on a query
    fn init(params: &Parameters<'a>) -> Result<(), FtError>;
    /// Parse a document or query
    fn parse(params: &Parameters<'a>) -> Result<(), FtError>;
    /// Terminate the parser at the end of the query
    fn deinit(params: &Parameters<'a>) -> Result<(), FtError>;
}

pub enum ParserMode {
    Simple = bindings::enum_ftparser_mode::MYSQL_FTPARSER_SIMPLE_MODE as isize,
    WithStopwords = bindings::enum_ftparser_mode::MYSQL_FTPARSER_WITH_STOPWORDS as isize,
    FullBoolean = bindings::enum_ftparser_mode::MYSQL_FTPARSER_FULL_BOOLEAN_INFO as isize,
}

/// Token for boolean info
pub enum TokenType {
    Eof,
    Word,
    LeftParen,
    RightParen,
    Stopword,
}

#[repr(transparent)]
pub struct BooleanInfo<'a> {
    inner: UnsafeCell<mariadb_sys::st_mysql_ftparser_boolean_info>,
    phantom: PhantomData<&'a ()>,
}

pub enum Presence {
    /// The result of the `+` operator
    Required,
    /// The result of the `-` operator
    Disallowed,
    /// The default case
    Optional,
}

impl BooleanInfo<'_> {
    pub fn requires_presence(&self) -> Presence {
        let yesno = unsafe { (*self.inner.get()).yesno };
        match yesno.cmp(&0) {
            Ordering::Greater => Presence::Required,
            Ordering::Less => Presence::Disallowed,
            Ordering::Equal => Presence::Optional,
        }
    }

    /// If positive, increase the word's weight. If negative, decrease it.
    pub fn weight_adjust(&self) -> i32 {
        unsafe { (*self.inner.get()).weight_adjust }
    }

    pub fn trunc(&self) {}
}

#[repr(transparent)]
pub struct Parameters<'a> {
    inner: UnsafeCell<mariadb_sys::st_mysql_ftparser_param>,
    phantom: PhantomData<&'a ()>,
}

impl Parameters<'_> {
    /// Use the default parser to parse this data
    pub fn parse(&self, doc: &[u8]) -> Result<(), FtError> {
        let this = self.inner.get();
        let parse_fn = unsafe { (*this).mysql_parse.unwrap() };
        let res = unsafe { parse_fn(this, doc.as_ptr().cast(), doc.len().try_into().unwrap()) };
        to_result(res)
    }

    /// Call this to add a word to the ft index
    pub fn add_word(&self, word: &[u8]) {}

    pub fn add_boolean_word(&self, word: &[u8], boolean_info: ()) {}

    fn state() {}

    /// Data may be overwritten; needs to be copied to be persisted
    pub fn needs_copy(&self) -> bool {
        let flags = unsafe { (*self.inner.get()).flags };
        (flags & bindings::MYSQL_FTFLAGS_NEED_COPY) != 0
    }

    pub fn mode() {}
}
