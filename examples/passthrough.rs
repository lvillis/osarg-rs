// Wrapper / passthrough pattern using `current_value_and_remaining()`.
//
// Try:
//   cargo run --example passthrough -- --dry-run echo hello world
//   cargo run --example passthrough -- -e RUST_LOG=debug cargo test -- --nocapture

use osarg::{Arg, Parser, help, standard};
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::from_env();
    let mut dry_run = false;
    let mut envs = Vec::new();
    let mut command = None;
    let mut args = Vec::new();

    while let Some(arg) = parser.next()? {
        if standard::try_print(arg, HELP_DOC, VERSION)? {
            return Ok(());
        }

        match arg {
            Arg::Short('n') | Arg::Long("dry-run") => {
                osarg::set_flag(&mut dry_run);
            }
            Arg::Short('e') | Arg::Long("env") => {
                parser.push_split_once_nonempty_key_owned('=', &mut envs)?;
            }
            Arg::Value(_) => {
                let (cmd, forwarded) = parser.current_value_and_remaining()?;
                command = Some(cmd);
                args = forwarded;
                break;
            }
            other => return Err(other.unexpected().into()),
        }
    }

    let config = Config {
        dry_run,
        envs,
        command: osarg::required(command, "<CMD>")?,
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
