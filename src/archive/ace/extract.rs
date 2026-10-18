//! Extracts selected entries from ACE archives.

use std::{error::Error, fs, io, path::Path};

use unarc_rs::ace::AceArchive;

use crate::archive::path;

use super::{
    MAX_ENTRY_SIZE,
    ensure_supported_archive::ensure_supported_archive,
    ensure_supported_entry::ensure_supported_entry,
    map_password_error::map_password_error,
    normalized_entry_path::normalized_entry_path,
    safe_relative_path::safe_relative_path,
};

/// Extracts selected ACE entries after validating their paths and features.
pub(crate) fn extract(
    archive_path: &Path,
    destination: &Path,
    selected: &[(String, bool)],
    password: Option<&str>,
) -> Result<usize, Box<dyn Error>> {
    let mut archive = AceArchive::new(std::fs::File::open(archive_path)?)?;
    ensure_supported_archive(&archive)?;
    if let Some(password) = password {
        archive.set_password(password);
    }
    let solid = archive.is_solid();
    let mut count = 0;

    while let Some(header) = archive.get_next_entry()? {
        ensure_supported_entry(&header)?;
        let relative = normalized_entry_path(&header.filename);
        let selected_entry = path::is_selected(&relative, selected);

        if !selected_entry && !(solid && !header.is_directory()) {
            archive.skip(&header)?;
            continue;
        }
        if header.is_directory() {
            let output = destination.join(safe_relative_path(&header.filename)?);
            fs::create_dir_all(output)?;
            if selected_entry {
                count += 1;
            }
            archive.skip(&header)?;
            continue;
        }
        if selected_entry && header.is_encrypted() && password.is_none() {
            return Err(crate::archive::PasswordError::Required.into());
        }
        if header.original_size > MAX_ENTRY_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "ACE entry '{}' exceeds the {}-byte extraction limit",
                    header.filename, MAX_ENTRY_SIZE
                ),
            )
            .into());
        }

        let data = archive
            .read(&header)
            .map_err(|error| map_password_error(error, password.is_some()))?;
        if selected_entry {
            let output = destination.join(safe_relative_path(&header.filename)?);
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(output, data)?;
            count += 1;
        }
    }

    Ok(count)
}
