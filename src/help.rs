//! Extremely small helpers for caller-owned help text.
//!
//! This module intentionally stays narrow: it writes a usage line plus a list
//! of pre-authored sections. It does not infer flags, format tables, colorize
//! output, or own application control flow.
//!
//! Typical usage pairs this module with [`crate::standard`] so the parser
//! recognizes `--help`, while the application still owns the rendered text.
//!
//! ```rust
//! use osarg::help;
//!
//! const SECTIONS: &[help::Section<'static>] = &[
//!     help::Section::new(
//!         "options:",
//!         "  -h, --help       show help\n  -V, --version    show version",
//!     ),
//!     help::Section::new("examples:", "  demo --help"),
//! ];
//!
//! let doc = help::Help::new("demo [OPTIONS]", SECTIONS);
//! let mut output = Vec::new();
//! doc.write(&mut output)?;
//!
//! assert_eq!(
//!     String::from_utf8(output)?,
//!     "usage: demo [OPTIONS]\n\noptions:\n  -h, --help       show help\n  -V, --version    show version\n\nexamples:\n  demo --help\n"
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::io;
use std::io::Write;

/// A caller-authored help section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Section<'a> {
    heading: &'a str,
    body: &'a str,
}

/// A thin borrowed help document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Help<'a> {
    usage: &'a str,
    sections: &'a [Section<'a>],
}

impl<'a> Section<'a> {
    /// Builds a section from a heading and body.
    #[must_use]
    pub const fn new(heading: &'a str, body: &'a str) -> Self {
        Self { heading, body }
    }

    /// Returns the section heading.
    #[must_use]
    pub const fn heading(self) -> &'a str {
        self.heading
    }

    /// Returns the section body.
    #[must_use]
    pub const fn body(self) -> &'a str {
        self.body
    }
}

impl<'a> Help<'a> {
    /// Builds a borrowed help document from a usage line and sections.
    #[must_use]
    pub const fn new(usage: &'a str, sections: &'a [Section<'a>]) -> Self {
        Self { usage, sections }
    }

    /// Returns the usage line without the `usage:` prefix.
    #[must_use]
    pub const fn usage(self) -> &'a str {
        self.usage
    }

    /// Returns the caller-authored sections.
    #[must_use]
    pub const fn sections(self) -> &'a [Section<'a>] {
        self.sections
    }

    /// Writes the help document to the provided writer.
    ///
    /// This writes a `usage: ...` line, a blank line when sections exist, and
    /// then each section separated by a single blank line.
    pub fn write(self, writer: &mut dyn Write) -> io::Result<()> {
        write_usage(writer, self.usage)?;

        if self.sections.is_empty() {
            return Ok(());
        }

        writer.write_all(b"\n")?;

        for (index, section) in self.sections.iter().copied().enumerate() {
            write_section(writer, section)?;

            if index + 1 != self.sections.len() {
                writer.write_all(b"\n")?;
            }
        }

        Ok(())
    }
}

/// Writes a single `usage: ...` line.
///
/// A trailing newline is always written.
pub fn write_usage(writer: &mut dyn Write, usage: &str) -> io::Result<()> {
    writer.write_all(b"usage: ")?;
    writer.write_all(usage.as_bytes())?;
    writer.write_all(b"\n")
}

/// Writes a caller-authored section.
///
/// The section heading, when non-empty, is written on its own line.
/// A trailing newline is always ensured for the body.
pub fn write_section(writer: &mut dyn Write, section: Section<'_>) -> io::Result<()> {
    if !section.heading.is_empty() {
        writer.write_all(section.heading.as_bytes())?;
        writer.write_all(b"\n")?;
    }

    writer.write_all(section.body.as_bytes())?;

    if !section.body.ends_with('\n') {
        writer.write_all(b"\n")?;
    }

    Ok(())
}
