use std::{
    error::Error,
    io,
    path::{Path, PathBuf},
};

use super::{archive_kind, sources};

pub(super) fn add_files(
    archive_path: &Path,
    sources: &[PathBuf],
) -> Result<usize, Box<dyn Error>> {
    if sources.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "No files were dropped").into());
    }

    let kind = archive_kind(archive_path)?;
    if !kind.supports_modification() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Adding files is disabled for RAR, ACE, and FreeArc archives because Whitecap does not modify these formats.",
        )
        .into());
    }

    let mut entries = sources::collect_sources(sources)?;
    let existing = super::load(archive_path)?;
    sources::ensure_no_collisions(&existing, &entries)?;
    entries.retain(|entry| {
        !entry.is_dir
            || !existing.iter().any(|existing| {
                existing.is_dir
                    && existing.path.trim_end_matches('/')
                        == entry.archive_path.trim_end_matches('/')
            })
    });
    let count = entries.len();
    if count == 0 {
        return Ok(0);
    }

    match kind {
        super::ArchiveKind::Zip => super::zip::add(archive_path, &entries)?,
        super::ArchiveKind::Tar(compression) => {
            super::tar::add(archive_path, &entries, compression)?
        }
        super::ArchiveKind::SevenZip => super::sevenz::add(archive_path, &entries)?,
        super::ArchiveKind::Rar | super::ArchiveKind::Ace | super::ArchiveKind::FreeArc => unreachable!(
            "unsupported archive kinds are rejected before dispatch"
        ),
    }

    Ok(count)
}
