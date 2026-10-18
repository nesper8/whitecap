mod ace;
mod add;
mod delete;
mod entry;
mod format;
mod freearc;
mod operations;
mod path;
mod rar;
mod sevenz;
mod tar;
mod zip;
#[cfg(test)]
mod test_support;

use format::{ArchiveKind, archive_kind};

pub use entry::ArchiveEntry;
pub use operations::{add_files, delete_entries, extract, load, supports_modification};
#[cfg(test)]
pub(super) use test_support::temporary_directory;
