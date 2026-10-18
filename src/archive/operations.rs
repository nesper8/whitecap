use std::{
    error::Error,
    io,
    path::{Path, PathBuf},
};

use super::{ArchiveEntry, ArchiveKind, archive_kind};

pub fn load(path: &Path) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    match archive_kind(path)? {
        ArchiveKind::Zip => super::zip::load(path),
        ArchiveKind::Tar(compression) => super::tar::load(path, compression),
        ArchiveKind::SevenZip => super::sevenz::load(path),
        ArchiveKind::Rar => super::rar::load(path),
        ArchiveKind::FreeArc => super::freearc::load(path),
        ArchiveKind::Ace => super::ace::load(path),
    }
}

pub fn extract(
    archive_path: &Path,
    destination: &Path,
    selected: &[(String, bool)],
) -> Result<usize, Box<dyn Error>> {
    match archive_kind(archive_path)? {
        ArchiveKind::Zip => super::zip::extract(archive_path, destination, selected),
        ArchiveKind::Tar(compression) => {
            super::tar::extract(archive_path, destination, selected, compression)
        }
        ArchiveKind::SevenZip => super::sevenz::extract(archive_path, destination, selected),
        ArchiveKind::Rar => super::rar::extract(archive_path, destination, selected),
        ArchiveKind::FreeArc => super::freearc::extract(archive_path, destination, selected),
        ArchiveKind::Ace => super::ace::extract(archive_path, destination, selected),
    }
}

pub fn add_files(archive_path: &Path, sources: &[PathBuf]) -> Result<usize, Box<dyn Error>> {
    super::add::add_files(archive_path, sources)
}

pub fn supports_modification(path: &Path) -> Result<bool, io::Error> {
    Ok(matches!(
        archive_kind(path)?,
        ArchiveKind::Zip | ArchiveKind::SevenZip | ArchiveKind::Tar(_)
    ))
}

pub fn delete_entries(
    archive_path: &Path,
    selected: &[(String, bool)],
) -> Result<usize, Box<dyn Error>> {
    super::delete::delete_entries(archive_path, selected)
}
