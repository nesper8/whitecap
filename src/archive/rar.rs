// RAR library notice required by unrar-rs:
// UnRAR source code may be used in any software to handle RAR archives without limitations free of charge, but cannot be used to develop RAR (WinRAR) compatible archiver and to re-create RAR compression algorithm, which is proprietary. Distribution of modified UnRAR source code in separate form or as a part of other software is permitted, provided that full text of this paragraph, starting from "UnRAR source code" words, is included in license, or in documentation if license is not available, and in source code comments of resulting package.
//! Read-only RAR archive support.
//!
//! RAR archives can be listed and extracted, but Whitecap intentionally does
//! not create or rewrite them.

use std::{error::Error, fs::File, path::Path};

use super::{ArchiveEntry, path};

/// Loads RAR metadata with an optional password.
pub(super) fn load_with_password(
    path: &Path,
    password: Option<&str>,
) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    let archive = match password {
        Some(password) => unrar_rs::RarArchive::open_with_password(File::open(path)?, password),
        None => unrar_rs::RarArchive::open(File::open(path)?),
    }
    .map_err(|error| map_password_error(error, password.is_some()))?;
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

/// Extracts selected RAR entries without modifying the archive.
pub(super) fn extract(
    archive_path: &Path,
    destination: &Path,
    selected: &[(String, bool)],
    password: Option<&str>,
) -> Result<usize, Box<dyn Error>> {
    let mut archive = match password {
        Some(password) => {
            unrar_rs::RarArchive::open_with_password(File::open(archive_path)?, password)
        }
        None => unrar_rs::RarArchive::open(File::open(archive_path)?),
    }
    .map_err(|error| map_password_error(error, password.is_some()))?;
    let mut count = 0;
    for index in 0..archive.len() {
        let encrypted = archive.member_is_encrypted(index);
        let entry = archive
            .by_index(index)
            .map_err(|error| map_password_error(error, password.is_some()))?;
        let relative = path::normalize(entry.name());
        if path::is_selected(&relative, selected) {
            if encrypted && password.is_none() {
                return Err(super::PasswordError::Required.into());
            }
            entry
                .unpack_in(destination)
                .map_err(|error| map_password_error(error, password.is_some()))?;
            count += 1;
        } else {
            entry
                .skip()
                .map_err(|error| map_password_error(error, password.is_some()))?;
        }
    }
    Ok(count)
}

/// Maps UnRAR encryption failures to the shared password error states.
fn map_password_error(
    error: unrar_rs::RarError,
    password_was_provided: bool,
) -> Box<dyn Error> {
    match error {
        unrar_rs::RarError::EncryptedArchive | unrar_rs::RarError::EncryptedMember { .. }
            if !password_was_provided =>
        {
            super::PasswordError::Required.into()
        }
        unrar_rs::RarError::EncryptedArchive | unrar_rs::RarError::EncryptedMember { .. } => {
            super::PasswordError::Invalid.into()
        }
        unrar_rs::RarError::InvalidPassword | unrar_rs::RarError::WrongPassword { .. } => {
            super::PasswordError::Invalid.into()
        }
        error => error.into(),
    }
}
