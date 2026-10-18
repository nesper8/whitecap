use crate::archive::ArchiveEntry;

pub(crate) struct ListedEntry {
    pub(crate) entry: ArchiveEntry,
    pub(crate) archive_path: String,
}
