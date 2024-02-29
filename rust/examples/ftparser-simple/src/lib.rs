#![allow(unused)]

use mariadb::plugin::ftparser::{FtError, FullTextParser, Parameters};

struct FtSimple;

impl FullTextParser<'_> for FtSimple {
    fn init(params: &Parameters<'_>) -> Result<(), FtError> {
        todo!()
    }

    fn parse(params: &Parameters<'_>) -> Result<(), FtError> {
        todo!()
    }

    fn deinit(params: &Parameters<'_>) -> Result<(), FtError> {
        todo!()
    }
}
