//! Archive inspection and mutation services used by the GTK interface.
//!
//! Format-specific implementations are kept private to this module. Callers
//! use the small API re-exported below, which keeps UI code independent of the
//! archive library used for a particular format.
//!
//! Archive paths are normalized before they are shown in the UI or compared
//! with a selection. Extraction implementations additionally validate paths
//! before writing files so archive entries cannot escape the chosen
//! destination directory.

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
