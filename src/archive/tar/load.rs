use std::{error::Error, path::Path};

use crate::archive::{ArchiveEntry, path};

use super::{Compression, reader::reader};

pub(crate) fn load(
    path: &Path,
    compression: Compression,
) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    let reader = reader(path, compression)?;
    let mut archive = ::tar::Archive::new(reader);
    let mut entries = Vec::new();
    for entry in archive.entries()? {
        let entry = entry?;
        entries.push(ArchiveEntry {
            path: path::normalize(&entry.path()?.to_string_lossy()),
            is_dir: entry.header().entry_type().is_dir(),
            size: entry.size(),
            encrypted: false,
        });
    }
    Ok(entries)
}
