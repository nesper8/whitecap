//! Applies collision checks to collected archive sources.

use std::io;

use crate::archive::ArchiveEntry;

use super::{archive_index::ArchiveIndex, source_entry::SourceEntry};

/// Ensures additions cannot overwrite or conflict with existing archive paths.
///
/// This checks both direct collisions and structural conflicts, such as adding
/// a file where an existing directory or parent path is required.
pub(crate) fn ensure_no_collisions(
    existing: &[ArchiveEntry],
    additions: &[SourceEntry],
) -> io::Result<()> {
    let mut index = ArchiveIndex::from_entries(existing);
    additions.iter().try_for_each(|entry| index.add(entry))
}
