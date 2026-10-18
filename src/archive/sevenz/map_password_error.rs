//! Converts 7z password failures into the shared archive error type.

use std::error::Error;

/// Maps 7z password-required and bad-password errors to shared states.
pub(crate) fn map_password_error(
    error: sevenz_rust2::Error,
    password_was_provided: bool,
) -> Box<dyn Error> {
    match error {
        sevenz_rust2::Error::PasswordRequired if password_was_provided => {
            crate::archive::PasswordError::Invalid.into()
        }
        sevenz_rust2::Error::PasswordRequired => crate::archive::PasswordError::Required.into(),
        sevenz_rust2::Error::MaybeBadPassword(_) if password_was_provided => {
            crate::archive::PasswordError::Invalid.into()
        }
        sevenz_rust2::Error::MaybeBadPassword(_) => {
            crate::archive::PasswordError::Required.into()
        }
        error => error.into(),
    }
}
