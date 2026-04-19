use core::fmt;
use std::ffi::{OsStr, OsString};

/// A structured parse error.
///
/// `osarg` keeps error representation small and structured:
///
/// - [`ErrorKind`] identifies the stable category
/// - [`Error::argument`] exposes the associated argument when one exists,
///   including caller-owned placeholders such as `<PATH>`
///
/// Callers can use the public constructors to build their own structured
/// errors after additional validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    kind: ErrorKind,
    argument: Option<OsString>,
}

/// The stable error categories returned by `osarg`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// The caller rejected an option or token.
    UnexpectedArgument,
    /// The caller rejected a positional value.
    UnexpectedPositional,
    /// A required argument or positional was not provided.
    MissingArgument,
    /// An option expected a value but none was available.
    MissingValue,
    /// A value failed typed parsing.
    InvalidValue,
    /// A value could not be converted to UTF-8 on request.
    InvalidUtf8,
    /// An option name was not valid UTF-8 or was otherwise invalid.
    InvalidOptionName,
    /// The current argument does not have an accessible value.
    ValueUnavailable,
}

impl Error {
    /// Returns the coarse error category.
    #[must_use = "the error category is returned from this method"]
    pub const fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Returns the argument associated with the error, when available.
    ///
    /// Parser-produced errors usually attach the offending argument or value.
    /// Caller-authored errors can also attach placeholders such as `<PATH>` or
    /// `<CMD>` for missing required arguments. Some synthesized errors, such as
    /// "no current value is available", may have no associated argument.
    #[must_use = "the associated argument is returned from this method"]
    pub fn argument(&self) -> Option<&OsStr> {
        self.argument.as_deref()
    }

    /// Builds an [`ErrorKind::UnexpectedArgument`] error.
    pub fn unexpected_argument(argument: OsString) -> Self {
        Self {
            kind: ErrorKind::UnexpectedArgument,
            argument: Some(argument),
        }
    }

    /// Builds an [`ErrorKind::UnexpectedPositional`] error.
    pub fn unexpected_positional(argument: OsString) -> Self {
        Self {
            kind: ErrorKind::UnexpectedPositional,
            argument: Some(argument),
        }
    }

    /// Builds an [`ErrorKind::MissingArgument`] error.
    pub fn missing_argument_for(argument: OsString) -> Self {
        Self {
            kind: ErrorKind::MissingArgument,
            argument: Some(argument),
        }
    }

    /// Builds an [`ErrorKind::MissingValue`] error.
    pub fn missing_value_for(argument: OsString) -> Self {
        Self {
            kind: ErrorKind::MissingValue,
            argument: Some(argument),
        }
    }

    /// Builds an [`ErrorKind::InvalidValue`] error.
    pub fn invalid_value_for(argument: OsString) -> Self {
        Self {
            kind: ErrorKind::InvalidValue,
            argument: Some(argument),
        }
    }

    /// Builds an [`ErrorKind::InvalidUtf8`] error.
    pub fn invalid_utf8(argument: OsString) -> Self {
        Self {
            kind: ErrorKind::InvalidUtf8,
            argument: Some(argument),
        }
    }

    /// Builds an [`ErrorKind::InvalidOptionName`] error.
    pub fn invalid_option_name(argument: OsString) -> Self {
        Self {
            kind: ErrorKind::InvalidOptionName,
            argument: Some(argument),
        }
    }

    /// Builds an [`ErrorKind::ValueUnavailable`] error.
    pub fn value_unavailable_for(argument: OsString) -> Self {
        Self {
            kind: ErrorKind::ValueUnavailable,
            argument: Some(argument),
        }
    }

    /// Builds an error without an associated argument.
    pub(crate) fn without_argument(kind: ErrorKind) -> Self {
        Self {
            kind,
            argument: None,
        }
    }

    pub(crate) fn missing_value(argument: &OsStr) -> Self {
        Self::missing_value_for(argument.to_os_string())
    }

    pub(crate) fn invalid_value(argument: &OsStr) -> Self {
        Self::invalid_value_for(argument.to_os_string())
    }

    pub(crate) fn value_unavailable(argument: Option<&OsStr>) -> Self {
        argument.map_or_else(
            || Self::without_argument(ErrorKind::ValueUnavailable),
            |argument| Self::value_unavailable_for(argument.to_os_string()),
        )
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let argument = self.argument.as_deref().map(OsStr::to_string_lossy);

        match (self.kind, argument) {
            (ErrorKind::UnexpectedArgument, Some(argument)) => {
                write!(f, "unexpected argument: {argument}")
            }
            (ErrorKind::UnexpectedPositional, Some(argument)) => {
                write!(f, "unexpected positional argument: {argument}")
            }
            (ErrorKind::MissingArgument, Some(argument)) => {
                write!(f, "missing required argument: {argument}")
            }
            (ErrorKind::MissingValue, Some(argument)) => {
                write!(f, "missing value for argument: {argument}")
            }
            (ErrorKind::InvalidValue, Some(argument)) => {
                write!(f, "invalid value: {argument}")
            }
            (ErrorKind::InvalidUtf8, Some(argument)) => {
                write!(f, "argument is not valid UTF-8: {argument}")
            }
            (ErrorKind::InvalidOptionName, Some(argument)) => {
                write!(f, "option name is invalid or not valid UTF-8: {argument}")
            }
            (ErrorKind::ValueUnavailable, Some(argument)) => {
                write!(f, "value is not available for argument: {argument}")
            }
            (ErrorKind::UnexpectedArgument, None) => write!(f, "unexpected argument"),
            (ErrorKind::UnexpectedPositional, None) => write!(f, "unexpected positional argument"),
            (ErrorKind::MissingArgument, None) => write!(f, "missing required argument"),
            (ErrorKind::MissingValue, None) => write!(f, "missing value"),
            (ErrorKind::InvalidValue, None) => write!(f, "invalid value"),
            (ErrorKind::InvalidUtf8, None) => write!(f, "argument is not valid UTF-8"),
            (ErrorKind::InvalidOptionName, None) => {
                write!(f, "option name is invalid or not valid UTF-8")
            }
            (ErrorKind::ValueUnavailable, None) => {
                write!(f, "value is not available for the current argument")
            }
        }
    }
}

impl std::error::Error for Error {}
