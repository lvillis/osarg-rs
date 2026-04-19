use osarg::Parser;
use std::ffi::OsString;

pub fn parser(args: &[&str]) -> Parser<std::vec::IntoIter<OsString>> {
    Parser::from_args(args.iter().copied())
}
