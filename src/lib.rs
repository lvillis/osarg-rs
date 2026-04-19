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
//! let mut parser = Parser::new(
//!     ["-p", "8080", "--help"]
//!         .into_iter()
//!         .map(std::ffi::OsString::from),
//! );
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
//!             help = true;
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
//! let mut parser = Parser::new(
//!     ["--color", "--help"]
//!         .into_iter()
//!         .map(std::ffi::OsString::from),
//! );
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
//! let mut parser = Parser::new(["--help"].into_iter().map(std::ffi::OsString::from));
//! let mut output = Vec::new();
//!
//! while let Some(arg) = parser.next()? {
//!     if let Some(flag) = standard::classify(arg) {
//!         match flag {
//!             Flag::Help => HELP.write(&mut output)?,
//!             Flag::Version => standard::write(&mut output, flag, "", VERSION)?,
//!         }
//!     }
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
//! let mut parser = Parser::new(
//!     ["--env", "RUST_LOG=debug", "cargo", "test", "--", "--nocapture"]
//!         .into_iter()
//!         .map(std::ffi::OsString::from),
//! );
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
//!         Arg::Value(value) => {
//!             command = Some(value.to_os_string());
//!             forwarded = parser.remaining_vec();
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
