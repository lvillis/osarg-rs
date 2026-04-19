mod common;

use common::parser;
use osarg::{Arg, Error, ErrorKind, Value};
use std::ffi::OsString;

#[derive(Debug, PartialEq, Eq)]
struct Config {
    envs: Vec<(String, String)>,
    command: OsString,
    args: Vec<OsString>,
}

fn parse_env_pair(value: Value<'_>) -> Result<(String, String), Error> {
    let text = value.to_str()?;
    let Some((key, raw_value)) = text.split_once('=') else {
        return Err(value.invalid());
    };

    if key.is_empty() {
        return Err(value.invalid());
    }

    Ok((key.to_owned(), raw_value.to_owned()))
}

fn parse_wrapper(args: &[&str]) -> Result<Config, Error> {
    let mut parser = parser(args);
    let mut envs = Vec::new();
    let mut command = None;
    let mut forwarded = Vec::new();

    while let Some(arg) = parser.next()? {
        match arg {
            Arg::Short('e') | Arg::Long("env") => {
                envs.push(parse_env_pair(parser.value()?)?);
            }
            Arg::Value(value) => {
                command = Some(value.to_os_string());
                forwarded = parser.remaining_vec();
                break;
            }
            other => return Err(other.unexpected()),
        }
    }

    Ok(Config {
        envs,
        command: command.ok_or_else(|| Error::missing_argument_for("<CMD>".into()))?,
        args: forwarded,
    })
}

#[test]
fn wrapper_can_forward_remaining_arguments() {
    let config = parse_wrapper(&[
        "--env",
        "RUST_LOG=debug",
        "cargo",
        "test",
        "--",
        "--nocapture",
    ])
    .unwrap();

    assert_eq!(
        config,
        Config {
            envs: vec![(String::from("RUST_LOG"), String::from("debug"))],
            command: OsString::from("cargo"),
            args: vec![
                OsString::from("test"),
                OsString::from("--"),
                OsString::from("--nocapture"),
            ],
        }
    );
}

#[test]
fn remaining_vec_preserves_grouped_short_tail() {
    let mut parser = parser(&["-abc", "tail"]);
    assert_eq!(parser.next().unwrap(), Some(Arg::Short('a')));

    let remaining = parser.remaining_vec();
    assert_eq!(
        remaining,
        vec![OsString::from("-bc"), OsString::from("tail")]
    );
}

#[test]
fn wrapper_requires_a_command() {
    let error = parse_wrapper(&["--env", "RUST_LOG=debug"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::MissingArgument);
    assert_eq!(error.argument().unwrap().to_string_lossy(), "<CMD>");
}
