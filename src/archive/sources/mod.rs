//! Shared source collection and collision validation for archive writers.
//!
//! Source paths are converted to archive-relative names, traversed in stable
//! order, and rejected when they are symbolic links, unsupported filesystem
//! items, or conflicts with entries already present in an archive.

mod archive_index;
mod collect_entry;
mod collect_sources;
mod ensure_no_collisions;
mod source_entry;

pub(super) use collect_sources::collect_sources;
pub(super) use ensure_no_collisions::ensure_no_collisions;
pub(super) use source_entry::SourceEntry;

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::archive::ArchiveEntry;

    #[test]
    fn collects_dropped_folder_with_relative_paths() {
        let root = super::super::temporary_directory();
        let folder = root.join("documents");
        fs::create_dir_all(folder.join("nested")).unwrap();
        fs::write(folder.join("nested/readme.txt"), b"hello").unwrap();

        let entries = collect_sources(std::slice::from_ref(&folder)).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].archive_path, "documents/");
        assert_eq!(entries[1].archive_path, "documents/nested/");
        assert_eq!(entries[2].archive_path, "documents/nested/readme.txt");
        assert!(entries[0].is_dir);
        assert!(!entries[2].is_dir);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_collisions_with_existing_files() {
        let existing = vec![ArchiveEntry {
            path: "readme.txt".to_owned(),
            is_dir: false,
            size: 10,
            encrypted: false,
        }];
        let additions = vec![SourceEntry {
            source: std::path::PathBuf::new(),
            archive_path: "readme.txt".to_owned(),
            is_dir: false,
        }];
        assert_eq!(
            ensure_no_collisions(&existing, &additions)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::AlreadyExists
        );
    }
}
