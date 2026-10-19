//! Extracts selected entries from RAR archives.

use std::{error::Error, fs::File, path::Path};

use crate::archive::{PasswordError, path};

use super::map_password_error::map_password_error;

/// Extracts selected RAR entries without modifying the archive.
pub(crate) fn extract(
    archive_path: &Path,
    destination: &Path,
    selected: &[(String, bool)],
    password: Option<&str>,
) -> Result<usize, Box<dyn Error>> {
    let password_was_provided = password.is_some();
    let mut archive = match password {
        Some(password) => {
            unrar_rs::RarArchive::open_with_password(File::open(archive_path)?, password)
        }
        None => unrar_rs::RarArchive::open(File::open(archive_path)?),
    }
    .map_err(|error| map_password_error(error, password_was_provided))?;
    let mut count = 0;

    for index in 0..archive.len() {
        let encrypted = archive.member_is_encrypted(index);
        let entry = archive
            .by_index(index)
            .map_err(|error| map_password_error(error, password_was_provided))?;
        let relative = path::normalize(entry.name());
        if path::is_selected(&relative, selected) {
            if encrypted && !password_was_provided {
                return Err(PasswordError::Required.into());
            }
            entry
                .unpack_in(destination)
                .map_err(|error| map_password_error(error, password_was_provided))?;
            count += 1;
        } else {
            entry
                .skip()
                .map_err(|error| map_password_error(error, password_was_provided))?;
        }
    }

    Ok(count)
}
