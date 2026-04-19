mod common;

use common::parser;
use osarg::{Arg, Error, ErrorKind};
use std::path::PathBuf;
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
    path: PathBuf,
}

fn parse_config(args: &[&str]) -> Result<Config, Error> {
    let mut parser = parser(args);
    let mut color = ColorChoice::Auto;
    let mut verbose = 0u8;
    let mut path = None;

    while let Some(arg) = parser.next()? {
        match arg {
            Arg::Short('C') | Arg::Long("color") => {
                color = parser.parse_opt_or_default::<ColorChoice>()?;
            }
            Arg::Short('v') | Arg::Long("verbose") => {
                osarg::count_flag(&mut verbose);
            }
            Arg::Value(value) => {
                value.store_path_buf(&mut path)?;
            }
            other => return Err(other.unexpected()),
        }
    }

    Ok(Config {
        color,
        verbose,
        path: osarg::required(path, "<PATH>")?,
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
            path: PathBuf::from("./file.txt"),
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
            path: PathBuf::from("./file.txt"),
        }
    );
}

#[test]
fn optional_value_reports_invalid_choice() {
    let error = parse_config(&["--color=blue", "./file.txt"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidValue);
    assert_eq!(error.argument().unwrap().to_string_lossy(), "blue");
}
