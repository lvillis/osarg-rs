mod common;

use common::parser;
use osarg::{Arg, Error, ErrorKind};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
struct Config {
    bind: String,
    port: u16,
    path: PathBuf,
}

fn parse_config(args: &[&str]) -> Result<Config, Error> {
    let mut parser = parser(args);
    let mut bind = String::from("127.0.0.1");
    let mut port = 8080;
    let mut path = None;

    while let Some(arg) = parser.next()? {
        match arg {
            Arg::Short('b') | Arg::Long("bind") => {
                bind = parser.string_owned()?;
            }
            Arg::Short('p') | Arg::Long("port") => {
                port = parser.parse::<u16>()?;
            }
            Arg::Value(value) => {
                value.store_path_buf(&mut path)?;
            }
            other => return Err(other.unexpected()),
        }
    }

    Ok(Config {
        bind,
        port,
        path: osarg::required(path, "<PATH>")?,
    })
}

#[test]
fn parses_basic_cli_with_short_and_long_forms() {
    let config = parse_config(&["--bind", "0.0.0.0", "-p8080", "./data"]).unwrap();

    assert_eq!(
        config,
        Config {
            bind: String::from("0.0.0.0"),
            port: 8080,
            path: PathBuf::from("./data"),
        }
    );
}

#[test]
fn rejects_extra_positionals_in_basic_cli() {
    let error = parse_config(&["./data", "extra"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::UnexpectedPositional);
    assert_eq!(error.argument().unwrap().to_string_lossy(), "extra");
}

#[test]
fn reports_missing_required_positional_in_basic_cli() {
    let error = parse_config(&["--bind", "0.0.0.0"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::MissingArgument);
    assert_eq!(error.argument().unwrap().to_string_lossy(), "<PATH>");
}
