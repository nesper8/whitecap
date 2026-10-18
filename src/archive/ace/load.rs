use std::{error::Error, fs::File, path::Path};

use unarc_rs::ace::AceArchive;

use crate::archive::ArchiveEntry;

use super::{
    ensure_supported_archive::ensure_supported_archive,
    ensure_supported_entry::ensure_supported_entry,
    normalized_entry_path::normalized_entry_path,
};

pub(crate) fn load(path: &Path) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    let mut archive = AceArchive::new(File::open(path)?)?;
    ensure_supported_archive(&archive)?;

    let mut entries = Vec::new();
    while let Some(header) = archive.get_next_entry()? {
        ensure_supported_entry(&header)?;
        entries.push(ArchiveEntry {
            path: normalized_entry_path(&header.filename),
            is_dir: header.is_directory(),
            size: header.original_size,
            encrypted: header.is_encrypted(),
        });
        archive.skip(&header)?;
    }
    Ok(entries)
}
