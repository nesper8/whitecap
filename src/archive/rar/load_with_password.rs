//! Loads entry metadata from RAR archives.

use std::{error::Error, fs::File, path::Path};

use crate::archive::{ArchiveEntry, path};

use super::map_password_error::map_password_error;

/// Loads RAR metadata with an optional password.
pub(crate) fn load_with_password(
    path: &Path,
    password: Option<&str>,
) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    let password_was_provided = password.is_some();
    let archive = match password {
        Some(password) => unrar_rs::RarArchive::open_with_password(File::open(path)?, password),
        None => unrar_rs::RarArchive::open(File::open(path)?),
    }
    .map_err(|error| map_password_error(error, password_was_provided))?;

    Ok(archive
        .entries()
        .enumerate()
        .map(|(index, entry)| ArchiveEntry {
            path: path::normalize(&entry.name),
            is_dir: entry.is_directory,
            size: entry.unpacked_size.unwrap_or_default(),
            encrypted: archive.member_is_encrypted(index),
        })
        .collect())
}
