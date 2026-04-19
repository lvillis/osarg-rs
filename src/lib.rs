#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]

//! `osarg` is a small, zero-dependency CLI parser built around `OsStr` / `OsString`.
//!
//! The parser stays imperative by design: iterate one argument at a time, match explicitly,
//! and pull values only when the current option expects one.
//!
//! `osarg` supports the common forms needed by small system and container-oriented CLIs:
//!
//! - short options such as `-v`
//! - grouped short options such as `-abc`
//! - long options such as `--verbose`
//! - attached long values such as `--port=8080`
//! - separated values such as `--port 8080` and `-p 8080`
//! - attached short tails such as `-p8080`
//! - `--` end-of-options handling
//! - positional arguments
//! - repeated options
//! - wrapper-style passthrough via [`Parser::into_remaining`] and [`Parser::remaining_vec`]
//!
//! It intentionally does **not** provide derive macros, schema ownership, generated help text,
//! or a command-tree framework.
//!
//! ## Core Rule
//!
//! The parser is stateful but explicit:
//!
//! - call [`Parser::next`] to get one [`Arg`] at a time
//! - inspect that argument in user code
//! - call [`Parser::value`] or [`Parser::value_opt`] immediately if that option takes a value
//!
//! Value access only applies to the most recently returned option.
//!
//! ## Basic Example
//!
//! ```rust
//! use osarg::{Arg, Parser};
//!
//! let mut parser = Parser::from_args(["-p", "8080", "--help"]);
//!
//! let mut port = None;
//! let mut help = false;
//!
//! while let Some(arg) = parser.next()? {
//!     match arg {
//!         Arg::Short('p') | Arg::Long("port") => {
//!             port = Some(parser.value()?.parse::<u16>()?);
//!         }
//!         Arg::Short('h') | Arg::Long("help") => {
//!             osarg::set_flag(&mut help);
//!         }
//!         other => return Err(other.unexpected()),
//!     }
//! }
//!
//! assert_eq!(port, Some(8080));
//! assert!(help);
//! # Ok::<(), osarg::Error>(())
//! ```
//!
//! ## Optional Values
//!
//! Optional values stay schema-free. When the next token already looks like another option,
//! [`Parser::value_opt`] and [`Parser::parse_opt`] leave it untouched and return `None`.
//!
//! ```rust
//! use osarg::{Arg, Parser};
//!
//! let mut parser = Parser::from_args(["--color", "--help"]);
//!
//! assert_eq!(parser.next()?, Some(Arg::Long("color")));
//! assert_eq!(parser.value_opt()?.map(|value| value.to_str()), None);
//! assert_eq!(parser.next()?, Some(Arg::Long("help")));
//! # Ok::<(), osarg::Error>(())
//! ```
//!
//! ## Help And Version
//!
//! The [`standard`] module recognizes conventional `help` / `version` flags, while [`help`]
//! writes caller-owned text.
//!
//! ```rust
//! use osarg::{Arg, Parser, help, standard};
//! use osarg::standard::Flag;
//!
//! const SECTIONS: &[help::Section<'static>] = &[help::Section::new(
//!     "options:",
//!     "  -h, --help       show help\n  -V, --version    show version",
//! )];
//! const HELP: help::Help<'static> = help::Help::new("demo [OPTIONS]", SECTIONS);
//! const VERSION: &str = "1.2.3";
//!
//! let mut parser = Parser::from_args(["--help"]);
//! let mut output = Vec::new();
//!
//! while let Some(arg) = parser.next()? {
//!     standard::try_write(&mut output, arg, HELP, VERSION)?;
//! }
//!
//! assert!(String::from_utf8(output)?.starts_with("usage: demo [OPTIONS]\n"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Passthrough
//!
//! Wrapper-style tools can stop parsing and forward the remaining raw command line:
//!
//! ```rust
//! use osarg::{Arg, Parser};
//!
//! let mut parser = Parser::from_args([
//!     "--env",
//!     "RUST_LOG=debug",
//!     "cargo",
//!     "test",
//!     "--",
//!     "--nocapture",
//! ]);
//!
//! let mut env_value = None;
//! let mut command = None;
//! let mut forwarded = Vec::new();
//!
//! while let Some(arg) = parser.next()? {
//!     match arg {
//!         Arg::Long("env") => {
//!             env_value = Some(parser.os_string()?);
//!         }
//!         Arg::Value(_) => {
//!             let (cmd, args) = parser.current_value_and_remaining()?;
//!             command = Some(cmd);
//!             forwarded = args;
//!             break;
//!         }
//!         other => return Err(other.unexpected()),
//!     }
//! }
//!
//! assert_eq!(env_value, Some(std::ffi::OsString::from("RUST_LOG=debug")));
//! assert_eq!(command, Some(std::ffi::OsString::from("cargo")));
//! assert_eq!(
//!     forwarded,
//!     vec![
//!         std::ffi::OsString::from("test"),
//!         std::ffi::OsString::from("--"),
//!         std::ffi::OsString::from("--nocapture"),
//!     ]
//! );
//! # Ok::<(), osarg::Error>(())
//! ```

mod arg;
mod error;
pub mod help;
mod parser;
pub mod standard;
#[cfg(test)]
mod tests;
mod value;

pub use arg::Arg;
pub use error::{Error, ErrorKind};
pub use parser::{Parser, Remaining};
pub use value::Value;

/// Extracts a required caller-owned value from an [`Option`].
///
/// This is a thin convenience for required positionals or subcommands that are
/// collected during parsing and finalized afterwards.
///
/// ```rust
/// let path = osarg::required(Some(String::from("./data")), "<PATH>")?;
///
/// assert_eq!(path, "./data");
/// # Ok::<(), osarg::Error>(())
/// ```
pub fn required<T, S>(value: Option<T>, argument: S) -> Result<T, Error>
where
    S: Into<std::ffi::OsString>,
{
    let argument = argument.into();
    value.ok_or_else(move || Error::missing_argument_for(argument))
}

/// Increments a repeated flag counter with saturating semantics.
///
/// This is a thin convenience for flags such as `-v -v` or `-vv`.
///
/// ```rust
/// let mut verbose = 0u8;
///
/// osarg::count_flag(&mut verbose);
/// osarg::count_flag(&mut verbose);
///
/// assert_eq!(verbose, 2);
/// ```
pub fn count_flag(counter: &mut u8) {
    *counter = counter.saturating_add(1);
}

/// Sets a boolean flag to `true`.
///
/// This is a thin convenience for single flags such as `--dry-run`.
///
/// ```rust
/// let mut dry_run = false;
///
/// osarg::set_flag(&mut dry_run);
///
/// assert!(dry_run);
/// ```
pub fn set_flag(flag: &mut bool) {
    *flag = true;
}
