//! Loads normalized metadata from a ZIP archive.

use std::{error::Error, fs::File, path::Path};

use crate::archive::{ArchiveEntry, path};

/// Loads normalized metadata for every ZIP entry.
pub(crate) fn load(path: &Path) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    let mut archive = ::zip::ZipArchive::new(File::open(path)?)?;
    let mut entries = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let entry = archive.by_index_raw(index)?;
        entries.push(ArchiveEntry {
            path: path::normalize(entry.name()),
            is_dir: entry.is_dir(),
            size: entry.size(),
            encrypted: entry.encrypted(),
        });
    }
    Ok(entries)
}
