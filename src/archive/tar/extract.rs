//! Extracts selected entries from TAR-family archives.

use std::{error::Error, path::Path};

use crate::archive::path;

use super::{Compression, reader::reader};

/// Extracts selected entries from a TAR stream.
pub(crate) fn extract(
    archive_path: &Path,
    destination: &Path,
    selected: &[(String, bool)],
    compression: Compression,
) -> Result<usize, Box<dyn Error>> {
    let reader = reader(archive_path, compression)?;
    let mut archive = ::tar::Archive::new(reader);
    let mut count = 0;
    for entry in archive.entries()? {
        let mut entry = entry?;
        let relative = path::normalize(&entry.path()?.to_string_lossy());
        if path::is_selected(&relative, selected) && entry.unpack_in(destination)? {
            count += 1;
        }
    }
    Ok(count)
}
