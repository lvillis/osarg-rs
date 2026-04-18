// Wrapper / passthrough pattern using `remaining_vec()`.
//
// Try:
//   cargo run --example passthrough -- --dry-run echo hello world
//   cargo run --example passthrough -- -e RUST_LOG=debug cargo test -- --nocapture

use osarg::{Arg, Parser, Value, help, standard};
use standard::Flag;
use std::ffi::OsString;

const HELP_SECTIONS: &[help::Section<'static>] = &[
    help::Section::new(
        "options:",
        "  -h, --help         show help\n  -V, --version      show version\n  -n, --dry-run      print the wrapped command\n  -e, --env KEY=VAL  inject an environment pair",
    ),
    help::Section::new(
        "examples:",
        "  passthrough --dry-run echo hello\n  passthrough -e RUST_LOG=debug cargo test -- --nocapture",
    ),
];
const HELP_DOC: help::Help<'static> =
    help::Help::new("passthrough [OPTIONS] <CMD> [ARGS...]", HELP_SECTIONS);
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
struct Config {
    dry_run: bool,
    envs: Vec<(String, String)>,
    command: OsString,
    args: Vec<OsString>,
}

fn parse_env_pair(value: Value<'_>) -> Result<(String, String), osarg::Error> {
    let text = value.to_str()?;
    let Some((key, raw_value)) = text.split_once('=') else {
        return Err(osarg::Error::invalid_value_for(value.to_os_string()));
    };

    if key.is_empty() {
        return Err(osarg::Error::invalid_value_for(value.to_os_string()));
    }

    Ok((key.to_owned(), raw_value.to_owned()))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::from_env();
    let mut dry_run = false;
    let mut envs = Vec::new();
    let mut command = None;
    let mut args = Vec::new();

    while let Some(arg) = parser.next()? {
        if let Some(flag) = standard::classify(arg) {
            match flag {
                Flag::Help => {
                    HELP_DOC.write(&mut std::io::stdout())?;
                    return Ok(());
                }
                Flag::Version => {
                    standard::write(&mut std::io::stdout(), flag, "", VERSION)?;
                    return Ok(());
                }
            }
        }

        match arg {
            Arg::Short('n') | Arg::Long("dry-run") => {
                dry_run = true;
            }
            Arg::Short('e') | Arg::Long("env") => {
                envs.push(parse_env_pair(parser.value()?)?);
            }
            Arg::Value(value) => {
                command = Some(value.to_os_string());
                args = parser.remaining_vec();
                break;
            }
            other => return Err(other.unexpected().into()),
        }
    }

    let config = Config {
        dry_run,
        envs,
        command: command.ok_or_else(|| osarg::Error::unexpected_argument("<CMD>".into()))?,
        args,
    };

    println!("dry_run={}", config.dry_run);
    println!("command={}", config.command.to_string_lossy());

    for (key, value) in &config.envs {
        println!("env={key}={value}");
    }

    for arg in &config.args {
        println!("arg={}", arg.to_string_lossy());
    }

    Ok(())
}
