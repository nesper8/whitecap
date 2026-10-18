use std::{io, path::PathBuf};

use super::{collect_entry::collect_entry, source_entry::SourceEntry};

/// Collects files and directories in deterministic, archive-relative order.
///
/// Symbolic links and special filesystem items are rejected. Directory
/// children are sorted by filename so repeated additions produce stable
/// archive entry ordering.
pub(crate) fn collect_sources(sources: &[PathBuf]) -> Result<Vec<SourceEntry>, io::Error> {
    let mut entries = Vec::new();
    for source in sources {
        let name = source.file_name().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Cannot add a filesystem root: {}", source.display()),
            )
        })?;
        let archive_path = name.to_string_lossy().replace('\\', "/");
        collect_entry(source, archive_path, &mut entries)?;
    }
    Ok(entries)
}
