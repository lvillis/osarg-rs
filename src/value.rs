use crate::Error;
use core::fmt;
use core::str::FromStr;
use std::ffi::{OsStr, OsString};

/// A borrowed CLI value backed by the parser's current argument storage.
///
/// Values preserve OS-native semantics until you explicitly ask for UTF-8 with
/// [`Value::to_str`] or typed parsing with [`Value::parse`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Value<'a> {
    raw: &'a OsStr,
}

impl<'a> Value<'a> {
    /// Builds a value from a borrowed OS string.
    pub(crate) fn new(raw: &'a OsStr) -> Self {
        Self { raw }
    }

    /// Returns the original `OsStr` backing this value.
    #[must_use = "this returns the borrowed OS-native value"]
    pub const fn as_os_str(self) -> &'a OsStr {
        self.raw
    }

    /// Clones the value into an owned [`OsString`].
    #[must_use = "the owned OS string is returned from this method"]
    pub fn to_os_string(self) -> OsString {
        self.raw.to_os_string()
    }

    /// Returns a lossy UTF-8 rendering of the value.
    #[must_use = "the lossy UTF-8 rendering is returned from this method"]
    pub fn to_string_lossy(self) -> std::borrow::Cow<'a, str> {
        self.raw.to_string_lossy()
    }

    /// Converts the value to UTF-8.
    ///
    /// This returns [`crate::ErrorKind::InvalidUtf8`] when the underlying value
    /// is not valid UTF-8.
    #[must_use = "callers must use or propagate the UTF-8 conversion result"]
    pub fn to_str(self) -> Result<&'a str, Error> {
        self.raw
            .to_str()
            .ok_or_else(|| Error::invalid_utf8(self.raw.to_os_string()))
    }

    /// Parses the value with [`FromStr`].
    ///
    /// Parse failures are mapped to [`crate::ErrorKind::InvalidValue`].
    #[must_use = "callers must use or propagate the typed parse result"]
    pub fn parse<T>(self) -> Result<T, Error>
    where
        T: FromStr,
    {
        self.to_str()?
            .parse::<T>()
            .map_err(|_| Error::invalid_value(self.raw))
    }

    /// Converts the value into an "invalid value" error.
    ///
    /// This is useful when user code performs additional validation after
    /// calling [`Value::to_str`] or otherwise interpreting the raw value.
    ///
    /// ```rust
    /// use osarg::{Arg, Parser};
    ///
    /// let mut parser = Parser::new(["MODE"].into_iter().map(std::ffi::OsString::from));
    /// let arg = parser.next()?.expect("value present");
    /// let value = arg.as_value().expect("positional");
    ///
    /// assert_eq!(value.invalid().to_string(), "invalid value: MODE");
    /// # Ok::<(), osarg::Error>(())
    /// ```
    #[must_use = "the constructed error must be returned or handled by the caller"]
    pub fn invalid(self) -> Error {
        Error::invalid_value(self.raw)
    }

    /// Converts the value into an "unexpected positional" error.
    ///
    /// This is a convenience for rejecting positional arguments in user code.
    #[must_use = "the constructed error must be returned or handled by the caller"]
    pub fn unexpected(self) -> Error {
        Error::unexpected_positional(self.raw.to_os_string())
    }
}

impl AsRef<OsStr> for Value<'_> {
    fn as_ref(&self) -> &OsStr {
        self.raw
    }
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw.to_string_lossy())
    }
}

impl From<Value<'_>> for OsString {
    fn from(value: Value<'_>) -> Self {
        value.to_os_string()
    }
}

impl<'a> TryFrom<Value<'a>> for &'a str {
    type Error = Error;

    fn try_from(value: Value<'a>) -> Result<Self, Self::Error> {
        value.to_str()
    }
}
