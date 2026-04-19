// Repeated options and custom value validation.
//
// Try:
//   cargo run --example repeated_options -- -vv -Iinclude -I generated -D MODE=release src/main.c
//   cargo run --example repeated_options -- --include include --define FEATURE=1 src/main.c

use osarg::{Arg, Parser};
use std::path::PathBuf;

#[derive(Debug)]
struct Config {
    verbose: u8,
    includes: Vec<PathBuf>,
    defines: Vec<(String, String)>,
    input: PathBuf,
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
                osarg::count_flag(&mut verbose);
            }
            Arg::Short('I') | Arg::Long("include") => {
                parser.push_path_buf(&mut includes)?;
            }
            Arg::Short('D') | Arg::Long("define") => {
                parser.push_split_once_nonempty_key_owned('=', &mut defines)?;
            }
            Arg::Value(value) => {
                value.store_path_buf(&mut input)?;
            }
            other => return Err(other.unexpected().into()),
        }
    }

    let config = Config {
        verbose,
        includes,
        defines,
        input: osarg::required(input, "<INPUT>")?,
    };

    println!("verbose={}", config.verbose);
    println!("input={}", config.input.display());

    for include in &config.includes {
        println!("include={}", include.display());
    }

    for (key, value) in &config.defines {
        println!("define={key}={value}");
    }

    Ok(())
}
