#![allow(unused)]

use mariadb::plugin::ftparser::{FullTextParser, Parameters};
use mariadb::EmptyResult;

struct FtSimple;

impl FullTextParser<'_> for FtSimple {
    fn init(params: &Parameters<'_>) -> EmptyResult {
        todo!()
    }

    fn parse(params: &Parameters<'_>) -> EmptyResult {
        todo!()
    }

    fn deinit(params: &Parameters<'_>) -> EmptyResult {
        todo!()
    }
}
