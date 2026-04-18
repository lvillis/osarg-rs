use crate::{Error, Value};
use core::fmt;
use std::ffi::OsString;

/// A single parsed command-line token.
///
/// `osarg` keeps parsing explicit: the parser yields one `Arg` at a time, and
/// user code decides whether that token is accepted, rejected, or followed by a
/// value lookup on the parser itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arg<'a> {
    /// A short option such as `-h`.
    Short(char),
    /// A long option such as `--help`.
    Long(&'a str),
    /// A positional value or a token after `--`.
    Value(Value<'a>),
}

impl<'a> Arg<'a> {
    /// Returns the short option when this argument is [`Arg::Short`].
    #[must_use = "this returns the queried short option without consuming any parser state"]
    pub const fn as_short(self) -> Option<char> {
        match self {
            Self::Short(short) => Some(short),
            Self::Long(_) | Self::Value(_) => None,
        }
    }

    /// Returns the long option when this argument is [`Arg::Long`].
    #[must_use = "this returns the queried long option without consuming any parser state"]
    pub const fn as_long(self) -> Option<&'a str> {
        match self {
            Self::Long(long) => Some(long),
            Self::Short(_) | Self::Value(_) => None,
        }
    }

    /// Returns `true` when this argument is the given short option.
    #[must_use = "this query does not mutate the parsed argument"]
    pub const fn is_short(self, short: char) -> bool {
        matches!(self, Self::Short(actual) if actual == short)
    }

    /// Returns `true` when this argument is the given long option.
    #[must_use = "this query does not mutate the parsed argument"]
    pub fn is_long(self, long: &str) -> bool {
        matches!(self, Self::Long(actual) if actual == long)
    }

    /// Returns `true` when this argument is a positional value.
    #[must_use = "this query does not mutate the parsed argument"]
    pub const fn is_value(self) -> bool {
        matches!(self, Self::Value(_))
    }

    /// Returns the positional value when this argument is [`Arg::Value`].
    #[must_use = "this returns the borrowed positional value without consuming any parser state"]
    pub const fn as_value(self) -> Option<Value<'a>> {
        match self {
            Self::Value(value) => Some(value),
            Self::Short(_) | Self::Long(_) => None,
        }
    }

    /// Returns `true` when this argument matches either the short or long form.
    #[must_use = "this query does not mutate the parsed argument"]
    pub fn matches(self, short: char, long: &str) -> bool {
        self.is_short(short) || self.is_long(long)
    }

    /// Converts the parsed argument into an "unexpected" error.
    ///
    /// Short and long options become [`crate::ErrorKind::UnexpectedArgument`].
    /// Positional values become [`crate::ErrorKind::UnexpectedPositional`].
    pub fn unexpected(self) -> Error {
        match self {
            Self::Value(value) => value.unexpected(),
            Self::Short(short) => {
                let mut argument = String::with_capacity(2);
                argument.push('-');
                argument.push(short);
                Error::unexpected_argument(OsString::from(argument))
            }
            Self::Long(long) => {
                let mut argument = String::with_capacity(2 + long.len());
                argument.push_str("--");
                argument.push_str(long);
                Error::unexpected_argument(OsString::from(argument))
            }
        }
    }
}

impl fmt::Display for Arg<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Short(short) => write!(f, "-{short}"),
            Self::Long(long) => write!(f, "--{long}"),
            Self::Value(value) => fmt::Display::fmt(value, f),
        }
    }
}
