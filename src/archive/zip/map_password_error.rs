use std::error::Error;

pub(crate) fn map_password_error(error: ::zip::result::ZipError) -> Box<dyn Error> {
    match error {
        ::zip::result::ZipError::UnsupportedArchive(::zip::result::ZipError::PASSWORD_REQUIRED) => {
            crate::archive::PasswordError::Required.into()
        }
        ::zip::result::ZipError::InvalidPassword => crate::archive::PasswordError::Invalid.into(),
        error => error.into(),
    }
}
