//! Public archive-operation dispatch and format-independent behavior.

use std::{
    error::Error,
    io,
    path::{Path, PathBuf},
};

use super::{ArchiveEntry, ArchiveKind, archive_kind};

/// Loads the entries in an archive that does not require a password.
///
/// This is equivalent to [`load_with_password`] with no password. Encrypted
/// formats return [`super::PasswordError::Required`] when the archive cannot
/// be inspected without credentials.
pub fn load(path: &Path) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    load_with_password(path, None)
}

/// Loads archive entries using an optional password.
///
/// The returned paths are suitable for display and selection. This operation
/// does not extract data and therefore does not create or modify files.
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

/// Extracts all selected entries into `destination`.
///
/// A selection item is a `(path, is_directory)` pair. An empty selection means
/// the complete archive; selecting a directory includes its descendants.
/// Implementations validate archive paths before writing them beneath the
/// destination. The return value is the number of extracted entries.
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

/// Adds filesystem files or directories to a modifiable archive.
///
/// Source directories retain their names and relative paths. Existing archive
/// paths are never overwritten. ZIP, 7z, and TAR archives (including
/// compressed TAR variants) support this operation; RAR, ACE, and FreeArc are
/// intentionally read-only.
pub fn add_files(archive_path: &Path, sources: &[PathBuf]) -> Result<usize, Box<dyn Error>> {
    super::add::add_files(archive_path, sources)
}

/// Reports whether the archive format supports Whitecap's mutation operations.
///
/// This checks the format inferred from the filename and does not inspect or
/// modify the archive contents.
pub fn supports_modification(path: &Path) -> Result<bool, io::Error> {
    Ok(archive_kind(path)?.supports_modification())
}

/// Removes selected entries from a modifiable archive.
///
/// Directory selections are recursive. The operation rebuilds the archive as
/// needed and returns the number of removed entries. It returns an error when
/// no selection is supplied or when the selected entries do not exist.
pub fn delete_entries(
    archive_path: &Path,
    selected: &[(String, bool)],
) -> Result<usize, Box<dyn Error>> {
    super::delete::delete_entries(archive_path, selected)
}
