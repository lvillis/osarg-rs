use crate::Error;
use core::fmt;
use core::str::FromStr;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

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

    /// Clones the value into an owned [`PathBuf`].
    ///
    /// This preserves the OS-native path contents without requiring UTF-8.
    #[must_use = "the owned path is returned from this method"]
    pub fn to_path_buf(self) -> PathBuf {
        PathBuf::from(self.raw)
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

    /// Converts the value to an owned UTF-8 [`String`].
    ///
    /// This returns [`crate::ErrorKind::InvalidUtf8`] when the underlying value
    /// is not valid UTF-8.
    #[must_use = "callers must use or propagate the owned UTF-8 string result"]
    pub fn to_owned_string(self) -> Result<String, Error> {
        Ok(self.to_str()?.to_owned())
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

    /// Splits a required UTF-8 value into a pair around the first delimiter.
    ///
    /// This is useful for common forms such as `KEY=VALUE`.
    /// Missing delimiters are reported as [`crate::ErrorKind::InvalidValue`].
    ///
    /// ```rust
    /// use osarg::{Arg, Parser};
    ///
    /// let mut parser = Parser::from_args(["KEY=VALUE"]);
    /// let arg = parser.next()?.expect("value present");
    /// let value = arg.as_value().expect("positional");
    ///
    /// assert_eq!(value.split_once_required('=')?, ("KEY", "VALUE"));
    /// # Ok::<(), osarg::Error>(())
    /// ```
    #[must_use = "callers must use or propagate the split result"]
    pub fn split_once_required(self, delimiter: char) -> Result<(&'a str, &'a str), Error> {
        self.to_str()?
            .split_once(delimiter)
            .ok_or_else(|| self.invalid())
    }

    /// Splits a required UTF-8 value around the first delimiter and requires a non-empty key.
    ///
    /// This is useful for common forms such as `KEY=VALUE`.
    /// Missing delimiters or empty keys are reported as
    /// [`crate::ErrorKind::InvalidValue`].
    ///
    /// ```rust
    /// use osarg::{Arg, Parser};
    ///
    /// let mut parser = Parser::from_args(["KEY=VALUE"]);
    /// let arg = parser.next()?.expect("value present");
    /// let value = arg.as_value().expect("positional");
    ///
    /// assert_eq!(value.split_once_nonempty_key('=')?, ("KEY", "VALUE"));
    /// # Ok::<(), osarg::Error>(())
    /// ```
    #[must_use = "callers must use or propagate the validated split result"]
    pub fn split_once_nonempty_key(self, delimiter: char) -> Result<(&'a str, &'a str), Error> {
        let (key, value) = self.split_once_required(delimiter)?;

        if key.is_empty() {
            return Err(self.invalid());
        }

        Ok((key, value))
    }

    /// Splits a required UTF-8 value around the first delimiter, requires a non-empty key,
    /// and returns owned UTF-8 strings.
    ///
    /// This is useful for forms such as `KEY=VALUE` that need to be stored
    /// beyond the parser's borrow lifetime.
    ///
    /// ```rust
    /// use osarg::Parser;
    ///
    /// let mut parser = Parser::from_args(["KEY=VALUE"]);
    /// let value = parser.next()?.expect("value present").as_value().expect("positional");
    ///
    /// assert_eq!(
    ///     value.split_once_nonempty_key_owned('=')?,
    ///     (String::from("KEY"), String::from("VALUE"))
    /// );
    /// # Ok::<(), osarg::Error>(())
    /// ```
    #[must_use = "callers must use or propagate the owned validated split result"]
    pub fn split_once_nonempty_key_owned(self, delimiter: char) -> Result<(String, String), Error> {
        let (key, value) = self.split_once_nonempty_key(delimiter)?;
        Ok((key.to_owned(), value.to_owned()))
    }

    /// Splits a required UTF-8 value around the first delimiter and parses the right-hand side.
    ///
    /// This is useful for forms such as `PORT=8080` or `LIMIT=32`.
    /// Missing delimiters and parse failures are reported as
    /// [`crate::ErrorKind::InvalidValue`].
    ///
    /// ```rust
    /// use osarg::Parser;
    ///
    /// let mut parser = Parser::from_args(["PORT=8080"]);
    /// let value = parser.next()?.expect("value present").as_value().expect("positional");
    ///
    /// assert_eq!(value.split_once_parse_value::<u16>('=')?, ("PORT", 8080));
    /// # Ok::<(), osarg::Error>(())
    /// ```
    #[must_use = "callers must use or propagate the split and parse result"]
    pub fn split_once_parse_value<T>(self, delimiter: char) -> Result<(&'a str, T), Error>
    where
        T: FromStr,
    {
        let (key, value) = self.split_once_required(delimiter)?;
        let parsed = value.parse::<T>().map_err(|_| self.invalid())?;
        Ok((key, parsed))
    }

    /// Splits a required UTF-8 value around the first delimiter, requires a non-empty key,
    /// and parses the right-hand side.
    ///
    /// This is useful for forms such as `PORT=8080` or `RETRY=3`.
    /// Missing delimiters, empty keys, and parse failures are reported as
    /// [`crate::ErrorKind::InvalidValue`].
    ///
    /// ```rust
    /// use osarg::Parser;
    ///
    /// let mut parser = Parser::from_args(["PORT=8080"]);
    /// let value = parser.next()?.expect("value present").as_value().expect("positional");
    ///
    /// assert_eq!(
    ///     value.split_once_nonempty_key_parse::<u16>('=')?,
    ///     ("PORT", 8080)
    /// );
    /// # Ok::<(), osarg::Error>(())
    /// ```
    #[must_use = "callers must use or propagate the validated split and parse result"]
    pub fn split_once_nonempty_key_parse<T>(self, delimiter: char) -> Result<(&'a str, T), Error>
    where
        T: FromStr,
    {
        let (key, value) = self.split_once_nonempty_key(delimiter)?;
        let parsed = value.parse::<T>().map_err(|_| self.invalid())?;
        Ok((key, parsed))
    }

    /// Converts the value into an "invalid value" error.
    ///
    /// This is useful when user code performs additional validation after
    /// calling [`Value::to_str`] or otherwise interpreting the raw value.
    ///
    /// ```rust
    /// use osarg::{Arg, Parser};
    ///
    /// let mut parser = Parser::from_args(["MODE"]);
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

    /// Stores the value into an [`OsString`] slot that must be set at most once.
    ///
    /// This is useful for positional arguments such as `<PATH>` or `<CMD>`.
    /// If the slot is already populated, this returns [`Value::unexpected`].
    ///
    /// ```rust
    /// use osarg::{Arg, Parser};
    /// use std::ffi::OsString;
    ///
    /// let mut parser = Parser::from_args(["./data"]);
    /// let arg = parser.next()?.expect("value present");
    /// let value = arg.as_value().expect("positional");
    /// let mut path = None;
    ///
    /// value.store_os_string(&mut path)?;
    ///
    /// assert_eq!(path, Some(OsString::from("./data")));
    /// # Ok::<(), osarg::Error>(())
    /// ```
    pub fn store_os_string(self, slot: &mut Option<OsString>) -> Result<(), Error> {
        if slot.is_some() {
            return Err(self.unexpected());
        }

        *slot = Some(self.to_os_string());
        Ok(())
    }

    /// Stores the value into a [`PathBuf`] slot that must be set at most once.
    ///
    /// This is useful for path-like positionals such as `<PATH>` or `<ROOT>`.
    /// If the slot is already populated, this returns [`Value::unexpected`].
    ///
    /// ```rust
    /// use osarg::{Arg, Parser};
    /// use std::path::PathBuf;
    ///
    /// let mut parser = Parser::from_args(["./data"]);
    /// let arg = parser.next()?.expect("value present");
    /// let value = arg.as_value().expect("positional");
    /// let mut path = None;
    ///
    /// value.store_path_buf(&mut path)?;
    ///
    /// assert_eq!(path, Some(PathBuf::from("./data")));
    /// # Ok::<(), osarg::Error>(())
    /// ```
    pub fn store_path_buf(self, slot: &mut Option<PathBuf>) -> Result<(), Error> {
        if slot.is_some() {
            return Err(self.unexpected());
        }

        *slot = Some(self.to_path_buf());
        Ok(())
    }

    /// Stores the value into a UTF-8 [`String`] slot that must be set at most once.
    ///
    /// This is useful for required UTF-8 positionals such as `<NAME>`.
    /// If the slot is already populated, this returns [`Value::unexpected`].
    ///
    /// ```rust
    /// use osarg::{Arg, Parser};
    ///
    /// let mut parser = Parser::from_args(["demo"]);
    /// let arg = parser.next()?.expect("value present");
    /// let value = arg.as_value().expect("positional");
    /// let mut name = None;
    ///
    /// value.store_string(&mut name)?;
    ///
    /// assert_eq!(name, Some(String::from("demo")));
    /// # Ok::<(), osarg::Error>(())
    /// ```
    pub fn store_string(self, slot: &mut Option<String>) -> Result<(), Error> {
        if slot.is_some() {
            return Err(self.unexpected());
        }

        *slot = Some(self.to_owned_string()?);
        Ok(())
    }

    /// Stores the parsed value into a typed slot that must be set at most once.
    ///
    /// This is useful for typed positionals such as `<PORT>`.
    /// If the slot is already populated, this returns [`Value::unexpected`].
    ///
    /// ```rust
    /// use osarg::{Arg, Parser};
    ///
    /// let mut parser = Parser::from_args(["8080"]);
    /// let arg = parser.next()?.expect("value present");
    /// let value = arg.as_value().expect("positional");
    /// let mut port = None;
    ///
    /// value.store_parse::<u16>(&mut port)?;
    ///
    /// assert_eq!(port, Some(8080));
    /// # Ok::<(), osarg::Error>(())
    /// ```
    pub fn store_parse<T>(self, slot: &mut Option<T>) -> Result<(), Error>
    where
        T: FromStr,
    {
        if slot.is_some() {
            return Err(self.unexpected());
        }

        *slot = Some(self.parse::<T>()?);
        Ok(())
    }

    /// Splits the value around the first delimiter, requires a non-empty key,
    /// and stores the owned pair into a slot that must be set at most once.
    ///
    /// This is useful for single positionals such as `<KEY=VALUE>`.
    pub fn store_split_once_nonempty_key_owned(
        self,
        delimiter: char,
        slot: &mut Option<(String, String)>,
    ) -> Result<(), Error> {
        if slot.is_some() {
            return Err(self.unexpected());
        }

        *slot = Some(self.split_once_nonempty_key_owned(delimiter)?);
        Ok(())
    }

    /// Splits the value around the first delimiter, requires a non-empty key,
    /// parses the right-hand side, and stores the pair into a slot that must be
    /// set at most once.
    ///
    /// This is useful for single positionals such as `<PORT=8080>`.
    pub fn store_split_once_nonempty_key_parse<T>(
        self,
        delimiter: char,
        slot: &mut Option<(String, T)>,
    ) -> Result<(), Error>
    where
        T: FromStr,
    {
        if slot.is_some() {
            return Err(self.unexpected());
        }

        let (key, value) = self.split_once_nonempty_key_parse(delimiter)?;
        *slot = Some((key.to_owned(), value));
        Ok(())
    }

    /// Pushes the value into a repeated OS-native collector.
    ///
    /// This is useful for repeated options such as `-I path`.
    ///
    /// ```rust
    /// use osarg::Parser;
    /// use std::ffi::OsString;
    ///
    /// let mut parser = Parser::from_args(["include", "generated"]);
    /// let mut values = Vec::new();
    ///
    /// parser
    ///     .next()?
    ///     .expect("first value")
    ///     .as_value()
    ///     .expect("positional")
    ///     .push_os_string(&mut values);
    /// parser
    ///     .next()?
    ///     .expect("second value")
    ///     .as_value()
    ///     .expect("positional")
    ///     .push_os_string(&mut values);
    ///
    /// assert_eq!(values, vec![OsString::from("include"), OsString::from("generated")]);
    /// # Ok::<(), osarg::Error>(())
    /// ```
    pub fn push_os_string(self, values: &mut Vec<OsString>) {
        values.push(self.to_os_string());
    }

    /// Pushes the value into a repeated path collector.
    ///
    /// This is useful for repeated options such as `-I include` or `--path root`.
    pub fn push_path_buf(self, values: &mut Vec<PathBuf>) {
        values.push(self.to_path_buf());
    }

    /// Pushes the value into a repeated UTF-8 collector.
    ///
    /// This is useful for repeated options such as `--feature demo`.
    pub fn push_string(self, values: &mut Vec<String>) -> Result<(), Error> {
        values.push(self.to_owned_string()?);
        Ok(())
    }

    /// Parses the value and pushes it into a repeated typed collector.
    ///
    /// This is useful for repeated options such as `--port 8080 --port 9090`.
    pub fn push_parse<T>(self, values: &mut Vec<T>) -> Result<(), Error>
    where
        T: FromStr,
    {
        values.push(self.parse::<T>()?);
        Ok(())
    }

    /// Splits the value around the first delimiter, requires a non-empty key,
    /// and pushes the owned pair into a repeated collector.
    ///
    /// This is useful for repeated options such as `--env KEY=VALUE`.
    pub fn push_split_once_nonempty_key_owned(
        self,
        delimiter: char,
        values: &mut Vec<(String, String)>,
    ) -> Result<(), Error> {
        values.push(self.split_once_nonempty_key_owned(delimiter)?);
        Ok(())
    }

    /// Splits the value around the first delimiter, requires a non-empty key,
    /// parses the right-hand side, and pushes the pair into a repeated collector.
    ///
    /// This is useful for repeated options such as `--port-map HTTP=8080`.
    pub fn push_split_once_nonempty_key_parse<T>(
        self,
        delimiter: char,
        values: &mut Vec<(String, T)>,
    ) -> Result<(), Error>
    where
        T: FromStr,
    {
        let (key, value) = self.split_once_nonempty_key_parse(delimiter)?;
        values.push((key.to_owned(), value));
        Ok(())
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
