//! Loads FreeArc entry metadata without modifying the archive.

use std::{error::Error, path::Path};

use super::list_archive::list_archive;

/// Returns the normalized entries reported by a FreeArc archive.
pub(crate) fn load(archive_path: &Path) -> Result<Vec<crate::archive::ArchiveEntry>, Box<dyn Error>> {
    Ok(list_archive(archive_path)?
        .into_iter()
        .map(|entry| entry.entry)
        .collect())
}
