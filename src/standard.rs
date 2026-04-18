//! Thin helpers for the conventional `--help` / `--version` flags.
//!
//! This module intentionally only matches the standard flag names. It does not
//! own help text, version text, exit behavior, or output formatting.
//!
//! It is designed to pair with [`crate::Parser`] and [`crate::help`] without
//! pulling command metadata into the library.
//!
//! ```rust
//! use osarg::{Arg, Parser, help, standard};
//! use standard::Flag;
//!
//! const SECTIONS: &[help::Section<'static>] = &[help::Section::new(
//!     "options:",
//!     "  -h, --help       show help\n  -V, --version    show version",
//! )];
//! const HELP: help::Help<'static> = help::Help::new("demo [OPTIONS]", SECTIONS);
//! const VERSION: &str = "1.2.3";
//!
//! let mut parser = Parser::new(["-V"].into_iter().map(std::ffi::OsString::from));
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
//! assert_eq!(String::from_utf8(output)?, "1.2.3");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::Arg;
use std::io;
use std::io::Write;

/// The conventional standard flags recognized by this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    /// `-h` / `--help`
    Help,
    /// `-V` / `--version`
    Version,
}

/// The conventional short help flag.
pub const HELP_SHORT: char = 'h';
/// The conventional long help flag.
pub const HELP_LONG: &str = "help";
/// The conventional short version flag.
pub const VERSION_SHORT: char = 'V';
/// The conventional long version flag.
pub const VERSION_LONG: &str = "version";

/// Returns `true` when the argument matches `-h` or `--help`.
#[must_use = "this query does not mutate the parsed argument"]
pub fn is_help(arg: Arg<'_>) -> bool {
    arg.matches(HELP_SHORT, HELP_LONG)
}

/// Returns `true` when the argument matches `-V` or `--version`.
#[must_use = "this query does not mutate the parsed argument"]
pub fn is_version(arg: Arg<'_>) -> bool {
    arg.matches(VERSION_SHORT, VERSION_LONG)
}

/// Classifies the argument as one of the conventional standard flags.
#[must_use = "this classification is returned from this function"]
pub fn classify(arg: Arg<'_>) -> Option<Flag> {
    if is_help(arg) {
        Some(Flag::Help)
    } else if is_version(arg) {
        Some(Flag::Version)
    } else {
        None
    }
}

impl Flag {
    /// Returns the conventional short form for this flag.
    #[must_use = "the conventional short flag is returned from this method"]
    pub const fn short(self) -> char {
        match self {
            Self::Help => HELP_SHORT,
            Self::Version => VERSION_SHORT,
        }
    }

    /// Returns the conventional long form for this flag.
    #[must_use = "the conventional long flag is returned from this method"]
    pub const fn long(self) -> &'static str {
        match self {
            Self::Help => HELP_LONG,
            Self::Version => VERSION_LONG,
        }
    }

    /// Returns `true` when this flag matches the parsed argument.
    #[must_use = "this query does not mutate the parsed argument"]
    pub fn matches(self, arg: Arg<'_>) -> bool {
        arg.matches(self.short(), self.long())
    }
}

/// Returns the caller-owned text associated with the standard flag.
#[must_use = "the selected text is returned from this function"]
pub const fn text<'a>(flag: Flag, help: &'a str, version: &'a str) -> &'a str {
    match flag {
        Flag::Help => help,
        Flag::Version => version,
    }
}

/// Writes the caller-owned help or version text to the provided writer.
///
/// No newline is added automatically. If you want a trailing newline, include
/// it in the supplied `help` or `version` text.
pub fn write(writer: &mut dyn Write, flag: Flag, help: &str, version: &str) -> io::Result<()> {
    writer.write_all(text(flag, help, version).as_bytes())
}
