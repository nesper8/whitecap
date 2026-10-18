mod ace;
mod add;
mod delete;
mod entry;
mod format;
mod freearc;
mod operations;
mod path;
mod password;
mod rar;
mod rewrite;
mod sevenz;
mod sources;
mod tar;
mod zip;
#[cfg(test)]
mod test_support;

use format::{ArchiveKind, archive_kind};

pub use entry::ArchiveEntry;
pub use operations::{
    add_files, delete_entries, extract_with_password, load, load_with_password,
    supports_modification,
};
pub use password::PasswordError;
#[cfg(test)]
pub(super) use test_support::temporary_directory;
