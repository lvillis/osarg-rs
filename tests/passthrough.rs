mod common;

use common::parser;
use osarg::{Arg, Error, ErrorKind};
use std::ffi::OsString;

#[derive(Debug, PartialEq, Eq)]
struct Config {
    envs: Vec<(String, String)>,
    command: OsString,
    args: Vec<OsString>,
}

fn parse_wrapper(args: &[&str]) -> Result<Config, Error> {
    let mut parser = parser(args);
    let mut envs = Vec::new();
    let mut command = None;
    let mut forwarded = Vec::new();

    while let Some(arg) = parser.next()? {
        match arg {
            Arg::Short('e') | Arg::Long("env") => {
                parser.push_split_once_nonempty_key_owned('=', &mut envs)?;
            }
            Arg::Value(_) => {
                let (cmd, args) = parser.current_value_and_remaining()?;
                command = Some(cmd);
                forwarded = args;
                break;
            }
            other => return Err(other.unexpected()),
        }
    }

    Ok(Config {
        envs,
        command: osarg::required(command, "<CMD>")?,
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
