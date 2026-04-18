use osarg::Parser;
use std::ffi::OsString;

pub fn parser(args: &[&str]) -> Parser<std::vec::IntoIter<OsString>> {
    Parser::new(
        args.iter()
            .copied()
            .map(OsString::from)
            .collect::<Vec<_>>()
            .into_iter(),
    )
}
