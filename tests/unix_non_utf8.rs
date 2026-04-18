#![cfg(unix)]

use osarg::{Arg, ErrorKind, Parser};
use std::ffi::OsString;
use std::os::unix::ffi::{OsStrExt, OsStringExt};

#[test]
fn positional_non_utf8_values_are_preserved_via_public_api() {
    let value = OsString::from_vec(vec![0xff, b'a', b't', b'h']);
    let mut parser = Parser::new(vec![value].into_iter());

    match parser.next().unwrap() {
        Some(Arg::Value(value)) => {
            assert_eq!(value.as_os_str().as_bytes(), &[0xff, b'a', b't', b'h']);
            assert_eq!(value.to_str().unwrap_err().kind(), ErrorKind::InvalidUtf8);
        }
        other => panic!("unexpected argument: {other:?}"),
    }
}

#[test]
fn invalid_non_utf8_long_option_names_are_rejected_via_public_api() {
    let mut parser = Parser::new(vec![OsString::from_vec(vec![b'-', b'-', 0xff])].into_iter());
    let error = parser.next().unwrap_err();

    assert_eq!(error.kind(), ErrorKind::InvalidOptionName);
}
