//! Converts ACE library password failures into archive-layer errors.

use std::error::Error;

/// Distinguishes missing ACE passwords from rejected supplied passwords.
pub(crate) fn map_password_error(
    error: unarc_rs::error::ArchiveError,
    password_was_provided: bool,
) -> Box<dyn Error> {
    match error {
        unarc_rs::error::ArchiveError::EncryptionRequired { .. } => {
            crate::archive::PasswordError::Required.into()
        }
        unarc_rs::error::ArchiveError::InvalidPassword { .. }
        | unarc_rs::error::ArchiveError::CrcMismatch { .. }
            if password_was_provided =>
        {
            crate::archive::PasswordError::Invalid.into()
        }
        error => error.into(),
    }
}
