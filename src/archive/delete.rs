use std::{error::Error, io, path::Path};

use super::{archive_kind, path};

pub(super) fn delete_entries(
    archive_path: &Path,
    selected: &[(String, bool)],
) -> Result<usize, Box<dyn Error>> {
    if selected.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Select one or more archive entries to delete",
        )
        .into());
    }

    let kind = archive_kind(archive_path)?;
    if !kind.supports_modification() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Deleting files is disabled for RAR, ACE, and FreeArc archives because Whitecap does not modify these formats.",
        )
        .into());
    }

    match kind {
        super::ArchiveKind::Zip => super::zip::delete(archive_path, selected),
        super::ArchiveKind::SevenZip => super::sevenz::delete(archive_path, selected),
        super::ArchiveKind::Tar(compression) => {
            super::tar::delete(archive_path, selected, compression)
        }
        super::ArchiveKind::Rar | super::ArchiveKind::Ace | super::ArchiveKind::FreeArc => unreachable!(
            "unsupported archive kinds are rejected before dispatch"
        ),
    }
}

pub(super) fn is_selected(entry_path: &str, selected: &[(String, bool)]) -> bool {
    path::is_selected(entry_path, selected)
}
