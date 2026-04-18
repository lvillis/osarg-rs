mod common;

use common::parser;
use osarg::{Arg, Error, ErrorKind};
use std::ffi::OsString;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ColorChoice {
    Always,
    #[default]
    Auto,
    Never,
}

impl FromStr for ColorChoice {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "always" => Ok(Self::Always),
            "auto" => Ok(Self::Auto),
            "never" => Ok(Self::Never),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Config {
    color: ColorChoice,
    verbose: u8,
    path: OsString,
}

fn parse_config(args: &[&str]) -> Result<Config, Error> {
    let mut parser = parser(args);
    let mut color = ColorChoice::Auto;
    let mut verbose = 0u8;
    let mut path = None;

    while let Some(arg) = parser.next()? {
        match arg {
            Arg::Short('C') | Arg::Long("color") => {
                color = parser
                    .parse_opt::<ColorChoice>()?
                    .unwrap_or(ColorChoice::Auto);
            }
            Arg::Short('v') | Arg::Long("verbose") => {
                verbose = verbose.saturating_add(1);
            }
            Arg::Value(value) => {
                if path.is_some() {
                    return Err(value.unexpected());
                }
                path = Some(value.to_os_string());
            }
            other => return Err(other.unexpected()),
        }
    }

    Ok(Config {
        color,
        verbose,
        path: path.ok_or_else(|| Error::unexpected_argument("<PATH>".into()))?,
    })
}

#[test]
fn optional_value_defaults_when_next_token_is_another_option() {
    let config = parse_config(&["--color", "--verbose", "./file.txt"]).unwrap();

    assert_eq!(
        config,
        Config {
            color: ColorChoice::Auto,
            verbose: 1,
            path: OsString::from("./file.txt"),
        }
    );
}

#[test]
fn optional_value_accepts_short_attached_tail() {
    let config = parse_config(&["-Calways", "./file.txt"]).unwrap();

    assert_eq!(
        config,
        Config {
            color: ColorChoice::Always,
            verbose: 0,
            path: OsString::from("./file.txt"),
        }
    );
}

#[test]
fn optional_value_reports_invalid_choice() {
    let error = parse_config(&["--color=blue", "./file.txt"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidValue);
    assert_eq!(error.argument().unwrap().to_string_lossy(), "blue");
}
