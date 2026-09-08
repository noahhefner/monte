//! Fatal error handling.
//!
//! Per FR-007, malformed docstrings and undocumented elements are handled
//! per-element (via `DocComment::Error` / `DocComment::None`) and MUST NOT
//! abort the run. This module only covers the fatal errors that may
//! legitimately stop the tool (e.g. invalid or unreadable input paths, per
//! contracts/cli.md and T019).

use std::fmt;
use std::io;

/// A fatal error that halts the tool, surfaced to the user on exit.
#[derive(Debug)]
pub enum Error {
    /// The provided input directory does not exist or is not a directory.
    InvalidInputPath(String),
    /// A source file could not be read from disk.
    ReadFile(String, io::Error),
    /// A user-provided theme stylesheet could not be read (contracts/cli.md).
    ReadTheme(String, io::Error),
    /// A Python source file failed to parse.
    ParseSource(String, String),
    /// The output directory could not be written.
    WriteOutput(String, io::Error),
    /// A generic message (e.g. no input/output arguments supplied).
    Message(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidInputPath(p) => {
                write!(
                    f,
                    "input path '{}' does not exist or is not a directory",
                    p
                )
            }
            Error::ReadFile(path, e) => {
                write!(f, "could not read '{}': {}", path, e)
            }
            Error::ReadTheme(path, e) => {
                write!(f, "could not read theme '{}': {}", path, e)
            }
            Error::ParseSource(path, e) => {
                write!(f, "could not parse '{}': {}", path, e)
            }
            Error::WriteOutput(path, e) => {
                write!(f, "could not write output '{}': {}", path, e)
            }
            Error::Message(m) => write!(f, "{}", m),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Message(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
