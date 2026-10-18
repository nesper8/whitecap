use std::{
    error::Error,
    io,
    path::{Path, PathBuf},
};

use super::{ArchiveEntry, ArchiveKind, archive_kind};

pub fn load(path: &Path) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    load_with_password(path, None)
}

pub fn load_with_password(
    path: &Path,
    password: Option<&str>,
) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    match archive_kind(path)? {
        ArchiveKind::Zip => super::zip::load(path),
        ArchiveKind::Tar(compression) => super::tar::load(path, compression),
        ArchiveKind::SevenZip => super::sevenz::load_with_password(path, password),
        ArchiveKind::Rar => super::rar::load_with_password(path, password),
        ArchiveKind::FreeArc => super::freearc::load(path),
        ArchiveKind::Ace => super::ace::load(path),
    }
}

pub fn extract_with_password(
    archive_path: &Path,
    destination: &Path,
    selected: &[(String, bool)],
    password: Option<&str>,
) -> Result<usize, Box<dyn Error>> {
    match archive_kind(archive_path)? {
        ArchiveKind::Zip => super::zip::extract(archive_path, destination, selected, password),
        ArchiveKind::Tar(compression) => {
            super::tar::extract(archive_path, destination, selected, compression)
        }
        ArchiveKind::SevenZip => {
            super::sevenz::extract(archive_path, destination, selected, password)
        }
        ArchiveKind::Rar => super::rar::extract(archive_path, destination, selected, password),
        ArchiveKind::FreeArc => super::freearc::extract(archive_path, destination, selected),
        ArchiveKind::Ace => super::ace::extract(archive_path, destination, selected, password),
    }
}

pub fn add_files(archive_path: &Path, sources: &[PathBuf]) -> Result<usize, Box<dyn Error>> {
    super::add::add_files(archive_path, sources)
}

pub fn supports_modification(path: &Path) -> Result<bool, io::Error> {
    Ok(archive_kind(path)?.supports_modification())
}

pub fn delete_entries(
    archive_path: &Path,
    selected: &[(String, bool)],
) -> Result<usize, Box<dyn Error>> {
    super::delete::delete_entries(archive_path, selected)
}
