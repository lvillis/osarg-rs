mod common;

use common::parser;
use osarg::{Arg, Error, ErrorKind, Value};
use std::ffi::OsString;

#[derive(Debug, PartialEq, Eq)]
struct Config {
    verbose: u8,
    includes: Vec<OsString>,
    defines: Vec<(String, String)>,
    input: OsString,
}

fn parse_define(value: Value<'_>) -> Result<(String, String), Error> {
    let text = value.to_str()?;
    let Some((key, raw_value)) = text.split_once('=') else {
        return Err(Error::invalid_value_for(value.to_os_string()));
    };

    if key.is_empty() {
        return Err(Error::invalid_value_for(value.to_os_string()));
    }

    Ok((key.to_owned(), raw_value.to_owned()))
}

fn parse_config(args: &[&str]) -> Result<Config, Error> {
    let mut parser = parser(args);
    let mut verbose = 0u8;
    let mut includes = Vec::new();
    let mut defines = Vec::new();
    let mut input = None;

    while let Some(arg) = parser.next()? {
        match arg {
            Arg::Short('v') | Arg::Long("verbose") => {
                verbose = verbose.saturating_add(1);
            }
            Arg::Short('I') | Arg::Long("include") => {
                includes.push(parser.value()?.to_os_string());
            }
            Arg::Short('D') | Arg::Long("define") => {
                defines.push(parse_define(parser.value()?)?);
            }
            Arg::Value(value) => {
                if input.is_some() {
                    return Err(value.unexpected());
                }
                input = Some(value.to_os_string());
            }
            other => return Err(other.unexpected()),
        }
    }

    Ok(Config {
        verbose,
        includes,
        defines,
        input: input.ok_or_else(|| Error::unexpected_argument("<INPUT>".into()))?,
    })
}

#[test]
fn repeated_options_are_collected_in_order() {
    let config = parse_config(&[
        "-vv",
        "-Iinclude",
        "-I",
        "generated",
        "-D",
        "MODE=release",
        "src/main.c",
    ])
    .unwrap();

    assert_eq!(
        config,
        Config {
            verbose: 2,
            includes: vec![OsString::from("include"), OsString::from("generated")],
            defines: vec![(String::from("MODE"), String::from("release"))],
            input: OsString::from("src/main.c"),
        }
    );
}

#[test]
fn repeated_options_reject_invalid_define_values() {
    let error = parse_config(&["-D", "MODE", "src/main.c"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidValue);
    assert_eq!(error.argument().unwrap().to_string_lossy(), "MODE");
}
