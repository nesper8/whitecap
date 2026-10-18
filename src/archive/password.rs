//! Error types for archive password handling.

use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Password state reported by an archive operation.
pub enum PasswordError {
    /// The archive requires a password and none was supplied.
    Required,
    /// A supplied password was rejected by the archive.
    Invalid,
}

impl fmt::Display for PasswordError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Required => formatter.write_str("This archive requires a password"),
            Self::Invalid => formatter.write_str("The archive password is incorrect"),
        }
    }
}

impl Error for PasswordError {}
