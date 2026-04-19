// Optional-value pattern using `parse_opt()`.
//
// In a schema-free parser, the optional value is only left alone when the next
// token already looks like another option. That keeps control flow explicit.
//
// Try:
//   cargo run --example optional_values -- --color=never ./file.txt
//   cargo run --example optional_values -- --color --verbose ./file.txt
//   cargo run --example optional_values -- -Calways ./file.txt

use osarg::{Arg, Parser};
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::from_env();
    let mut color = ColorChoice::default();
    let mut verbose = 0u8;
    let mut path: Option<PathBuf> = None;

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
            other => return Err(other.unexpected().into()),
        }
    }

    let path: PathBuf = osarg::required(path, "<PATH>")?;

    println!("color={color:?} verbose={verbose} path={}", path.display());

    Ok(())
}
