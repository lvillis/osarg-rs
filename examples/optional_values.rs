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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::from_env();
    let mut color = ColorChoice::default();
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
                    return Err(value.unexpected().into());
                }
                path = Some(value.to_os_string());
            }
            other => return Err(other.unexpected().into()),
        }
    }

    let path: OsString = path.ok_or_else(|| osarg::Error::unexpected_argument("<PATH>".into()))?;

    println!(
        "color={color:?} verbose={verbose} path={}",
        path.to_string_lossy()
    );

    Ok(())
}
