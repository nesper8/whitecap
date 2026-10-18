//! Validated source data passed to format-specific archive writers.

use std::path::PathBuf;

/// A validated filesystem item prepared for an archive writer.
///
/// Directory paths include a trailing slash so format-specific writers can
/// preserve explicit directory entries.
pub(crate) struct SourceEntry {
    /// Filesystem path read by the format-specific writer.
    pub(crate) source: PathBuf,
    /// Archive-relative path, including a trailing slash for directories.
    pub(crate) archive_path: String,
    /// Whether the source should be written as an explicit directory entry.
    pub(crate) is_dir: bool,
}
