mod common;

use common::parser;
use osarg::{Arg, Error, ErrorKind};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
struct Config {
    verbose: u8,
    includes: Vec<PathBuf>,
    defines: Vec<(String, String)>,
    input: PathBuf,
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
                osarg::count_flag(&mut verbose);
            }
            Arg::Short('I') | Arg::Long("include") => {
                parser.push_path_buf(&mut includes)?;
            }
            Arg::Short('D') | Arg::Long("define") => {
                parser.push_split_once_nonempty_key_owned('=', &mut defines)?;
            }
            Arg::Value(value) => {
                value.store_path_buf(&mut input)?;
            }
            other => return Err(other.unexpected()),
        }
    }

    Ok(Config {
        verbose,
        includes,
        defines,
        input: osarg::required(input, "<INPUT>")?,
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
            includes: vec![PathBuf::from("include"), PathBuf::from("generated")],
            defines: vec![(String::from("MODE"), String::from("release"))],
            input: PathBuf::from("src/main.c"),
        }
    );
}

#[test]
fn repeated_options_reject_invalid_define_values() {
    let error = parse_config(&["-D", "MODE", "src/main.c"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidValue);
    assert_eq!(error.argument().unwrap().to_string_lossy(), "MODE");
}
