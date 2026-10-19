//! Converts UnRAR password failures into shared archive errors.

use std::error::Error;

/// Distinguishes missing RAR passwords from rejected supplied passwords.
pub(crate) fn map_password_error(
    error: unrar_rs::RarError,
    password_was_provided: bool,
) -> Box<dyn Error> {
    match error {
        unrar_rs::RarError::EncryptedArchive | unrar_rs::RarError::EncryptedMember { .. }
            if !password_was_provided =>
        {
            crate::archive::PasswordError::Required.into()
        }
        unrar_rs::RarError::EncryptedArchive | unrar_rs::RarError::EncryptedMember { .. } => {
            crate::archive::PasswordError::Invalid.into()
        }
        unrar_rs::RarError::InvalidPassword | unrar_rs::RarError::WrongPassword { .. } => {
            crate::archive::PasswordError::Invalid.into()
        }
        error => error.into(),
    }
}
