// Repeated options and custom value validation.
//
// Try:
//   cargo run --example repeated_options -- -vv -Iinclude -I generated -D MODE=release src/main.c
//   cargo run --example repeated_options -- --include include --define FEATURE=1 src/main.c

use osarg::{Arg, Parser, Value};
use std::ffi::OsString;

#[derive(Debug)]
struct Config {
    verbose: u8,
    includes: Vec<OsString>,
    defines: Vec<(String, String)>,
    input: OsString,
}

fn parse_define(value: Value<'_>) -> Result<(String, String), osarg::Error> {
    let text = value.to_str()?;
    let Some((key, raw_value)) = text.split_once('=') else {
        return Err(value.invalid());
    };

    if key.is_empty() {
        return Err(value.invalid());
    }

    Ok((key.to_owned(), raw_value.to_owned()))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::from_env();
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
                includes.push(parser.os_string()?);
            }
            Arg::Short('D') | Arg::Long("define") => {
                defines.push(parse_define(parser.value()?)?);
            }
            Arg::Value(value) => {
                if input.is_some() {
                    return Err(value.unexpected().into());
                }
                input = Some(value.to_os_string());
            }
            other => return Err(other.unexpected().into()),
        }
    }

    let config = Config {
        verbose,
        includes,
        defines,
        input: input.ok_or_else(|| osarg::Error::missing_argument_for("<INPUT>".into()))?,
    };

    println!("verbose={}", config.verbose);
    println!("input={}", config.input.to_string_lossy());

    for include in &config.includes {
        println!("include={}", include.to_string_lossy());
    }

    for (key, value) in &config.defines {
        println!("define={key}={value}");
    }

    Ok(())
}
