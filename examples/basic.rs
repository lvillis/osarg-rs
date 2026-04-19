// Basic single-command CLI with caller-owned help/version text.
//
// Try:
//   cargo run --example basic -- --help
//   cargo run --example basic -- -b 0.0.0.0 -p8080 ./data

use osarg::{Arg, Parser, help, standard};
use std::path::PathBuf;

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
    path: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::from_env();
    let mut bind = String::from("127.0.0.1");
    let mut port = 8080;
    let mut path = None;

    while let Some(arg) = parser.next()? {
        if standard::try_print(arg, HELP_DOC, VERSION)? {
            return Ok(());
        }

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
            other => return Err(other.unexpected().into()),
        }
    }

    let config = Config {
        bind,
        port,
        path: osarg::required(path, "<PATH>")?,
    };

    println!(
        "bind={} port={} path={}",
        config.bind,
        config.port,
        config.path.display()
    );

    Ok(())
}
