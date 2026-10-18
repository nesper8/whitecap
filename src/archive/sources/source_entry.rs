use std::path::PathBuf;

/// A validated filesystem item prepared for an archive writer.
///
/// Directory paths include a trailing slash so format-specific writers can
/// preserve explicit directory entries.
pub(crate) struct SourceEntry {
    pub(crate) source: PathBuf,
    pub(crate) archive_path: String,
    pub(crate) is_dir: bool,
}
