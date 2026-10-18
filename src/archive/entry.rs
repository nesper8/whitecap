/// A single item reported by an archive reader.
///
/// The path is normalized to use `/` separators and is relative to the root
/// of the archive. Directory entries are retained because the UI uses them to
/// render folders and because selections can include a directory recursively.
#[derive(Clone, Debug)]
pub struct ArchiveEntry {
    /// Normalized path of the entry inside the archive.
    pub path: String,
    /// Whether this entry represents a directory.
    pub is_dir: bool,
    /// Uncompressed size in bytes, when provided by the archive format.
    pub size: u64,
    /// Whether the entry is encrypted or belongs to an encrypted archive.
    pub encrypted: bool,
}
