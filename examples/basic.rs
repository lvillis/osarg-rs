// Basic single-command CLI with caller-owned help/version text.
//
// Try:
//   cargo run --example basic -- --help
//   cargo run --example basic -- -b 0.0.0.0 -p8080 ./data

use osarg::{Arg, Parser, help, standard};
use standard::Flag;
use std::ffi::OsString;

const HELP_SECTIONS: &[help::Section<'static>] = &[
    help::Section::new(
        "options:",
        "  -h, --help       show help\n  -V, --version    show version\n  -b, --bind ADDR  listen address\n  -p, --port PORT  listen port",
    ),
    help::Section::new(
        "examples:",
        "  basic ./data\n  basic --bind 0.0.0.0 -p8080 ./data",
    ),
];
const HELP_DOC: help::Help<'static> = help::Help::new("basic [OPTIONS] <PATH>", HELP_SECTIONS);

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
struct Config {
    bind: String,
    port: u16,
    path: OsString,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::from_env();
    let mut bind = String::from("127.0.0.1");
    let mut port = 8080;
    let mut path = None;

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
            Arg::Short('b') | Arg::Long("bind") => {
                bind = parser.value()?.to_str()?.to_owned();
            }
            Arg::Short('p') | Arg::Long("port") => {
                port = parser.parse::<u16>()?;
            }
            Arg::Value(value) => {
                if path.is_some() {
                    return Err(value.unexpected().into());
                }
                path = Some(value.to_os_string());
            }
            other => return Err(other.unexpected().into()),
        }
    }

    let config = Config {
        bind,
        port,
        path: path.ok_or_else(|| osarg::Error::unexpected_argument("<PATH>".into()))?,
    };

    println!(
        "bind={} port={} path={}",
        config.bind,
        config.port,
        config.path.to_string_lossy()
    );

    Ok(())
}
