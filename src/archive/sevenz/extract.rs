//! Extracts selected 7z entries with optional password handling.

use std::{error::Error, fs::{self, File}, io::{self, Write}, path::Path};

use crate::archive::path;

use super::map_password_error::map_password_error;

/// Extracts selected 7z entries and preserves password error distinctions.
pub(crate) fn extract(
    archive_path: &Path,
    destination: &Path,
    selected: &[(String, bool)],
    password: Option<&str>,
) -> Result<usize, Box<dyn Error>> {
    let mut count = 0;
    sevenz_rust2::decompress_with_extract_fn_and_password(
        File::open(archive_path)?,
        destination,
        sevenz_rust2::Password::new(password.unwrap_or_default()),
        |entry, reader, output| {
            let relative = path::normalize(entry.name());
            if !path::is_selected(&relative, selected) {
                io::copy(reader, &mut io::sink())?;
                return Ok(true);
            }

            if entry.is_directory() {
                fs::create_dir_all(output)?;
            } else {
                if let Some(parent) = output.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut file = File::create(output)?;
                io::copy(reader, &mut file)?;
                file.flush()?;
            }
            count += 1;
            Ok(true)
        },
    )
    .map_err(|error| map_password_error(error, password.is_some()))?;
    Ok(count)
}
