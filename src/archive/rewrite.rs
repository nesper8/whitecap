use std::{fs, io, path::Path};

use tempfile::NamedTempFile;

/// Creates a sibling temporary file suitable for atomically rebuilding an archive.
///
/// The source archive must not be a symbolic link or read-only. The temporary
/// file inherits the source archive's permissions and is left for the caller to
/// persist only after the replacement archive has been fully written.
pub(super) fn replacement_file(archive_path: &Path) -> io::Result<NamedTempFile> {
    if fs::symlink_metadata(archive_path)?.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Editing an archive through a symbolic link is not supported",
        ));
    }
    let metadata = fs::metadata(archive_path)?;
    if metadata.permissions().readonly() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "The archive is read-only",
        ));
    }
    let parent = archive_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let temporary = NamedTempFile::new_in(parent)?;
    fs::set_permissions(temporary.path(), metadata.permissions())?;
    Ok(temporary)
}
