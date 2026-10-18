// RAR library notice required by unrar-rs:
// UnRAR source code may be used in any software to handle RAR archives without limitations free of charge, but cannot be used to develop RAR (WinRAR) compatible archiver and to re-create RAR compression algorithm, which is proprietary. Distribution of modified UnRAR source code in separate form or as a part of other software is permitted, provided that full text of this paragraph, starting from "UnRAR source code" words, is included in license, or in documentation if license is not available, and in source code comments of resulting package.
use std::{error::Error, fs::File, path::Path};

use super::{ArchiveEntry, path};

pub(super) fn load(path: &Path) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    let archive = unrar_rs::RarArchive::open(File::open(path)?)?;
    Ok(archive
        .entries()
        .map(|entry| ArchiveEntry {
            path: path::normalize(&entry.name),
            is_dir: entry.is_directory,
            size: entry.unpacked_size.unwrap_or_default(),
        })
        .collect())
}

pub(super) fn extract(
    archive_path: &Path,
    destination: &Path,
    selected: &[(String, bool)],
) -> Result<usize, Box<dyn Error>> {
    let mut archive = unrar_rs::RarArchive::open(File::open(archive_path)?)?;
    let mut count = 0;
    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        let relative = path::normalize(entry.name());
        if path::is_selected(&relative, selected) {
            entry.unpack_in(destination)?;
            count += 1;
        } else {
            entry.skip()?;
        }
    }
    Ok(count)
}
