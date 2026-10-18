//! Internal representation of a parsed FreeArc listing row.

use crate::archive::ArchiveEntry;

/// Keeps both the normalized display path and the original `unarc` path.
pub(crate) struct ListedEntry {
    pub(crate) entry: ArchiveEntry,
    pub(crate) archive_path: String,
}
