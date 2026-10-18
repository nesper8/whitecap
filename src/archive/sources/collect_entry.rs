//! Recursively validates and collects one filesystem item.

use std::{fs, io, path::Path};

use super::source_entry::SourceEntry;

/// Recursively collects one file or directory into archive-relative entries.
///
/// Symbolic links and unsupported filesystem item types are rejected.
pub(crate) fn collect_entry(
    source: &Path,
    archive_path: String,
    entries: &mut Vec<SourceEntry>,
) -> Result<(), io::Error> {
    let metadata = fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("Symbolic links cannot be added: {}", source.display()),
        ));
    }

    if metadata.is_file() {
        entries.push(SourceEntry {
            source: source.to_path_buf(),
            archive_path,
            is_dir: false,
        });
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("Unsupported filesystem item: {}", source.display()),
        ));
    }

    entries.push(SourceEntry {
        source: source.to_path_buf(),
        archive_path: format!("{archive_path}/"),
        is_dir: true,
    });
    let mut children = fs::read_dir(source)?.collect::<Result<Vec<_>, _>>()?;
    children.sort_by_key(|child| child.file_name());
    for child in children {
        let child_name = child.file_name().to_string_lossy().replace('\\', "/");
        collect_entry(
            &child.path(),
            format!("{archive_path}/{child_name}"),
            entries,
        )?;
    }
    Ok(())
}
