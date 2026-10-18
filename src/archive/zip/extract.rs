//! Extracts selected ZIP entries and maps encryption errors.

use std::{
    error::Error,
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use crate::archive::path;

use super::map_password_error::map_password_error;

/// Extracts selected ZIP entries while validating enclosed paths.
pub(crate) fn extract(
    archive_path: &Path,
    destination: &Path,
    selected: &[(String, bool)],
    password: Option<&str>,
) -> Result<usize, Box<dyn Error>> {
    let mut archive = ::zip::ZipArchive::new(File::open(archive_path)?)?;
    let mut count = 0;
    for index in 0..archive.len() {
        let relative_path = {
            let entry = archive.by_index_raw(index)?;
            let Some(relative_path) = entry.enclosed_name() else {
                continue;
            };
            let relative = path::normalize(&relative_path.to_string_lossy());
            if !path::is_selected(&relative, selected) {
                continue;
            }
            relative_path
        };
        let mut entry = match password {
            Some(password) => archive
                .by_index_decrypt(index, password.as_bytes())
                .map_err(map_password_error)?,
            None => match archive.by_index(index) {
                Ok(entry) => entry,
                Err(::zip::result::ZipError::UnsupportedArchive(
                    ::zip::result::ZipError::PASSWORD_REQUIRED,
                )) => return Err(crate::archive::PasswordError::Required.into()),
                Err(error) => return Err(error.into()),
            },
        };
        let output = destination.join(relative_path);
        if entry.is_dir() {
            fs::create_dir_all(output)?;
        } else {
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut file = File::create(output)?;
            io::copy(&mut entry, &mut file)?;
            file.flush()?;
        }
        count += 1;
    }
    Ok(count)
}
