//! ZIP archive support for listing, extraction, and mutation.
//!
//! ZIP-specific operations are isolated in submodules while this module
//! exposes the functions required by the archive dispatcher.

mod add;
mod delete;
mod extract;
mod load;
mod map_password_error;

pub(super) use add::add;
pub(super) use delete::delete;
pub(super) use extract::extract;
pub(super) use load::load;

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::Write,
    };

    use super::*;
    use crate::archive::{add_files, delete_entries, temporary_directory};

    #[test]
    fn extracts_only_selected_zip_directory() {
        let root = temporary_directory();
        let archive_path = root.join("sample.zip");
        let file = File::create(&archive_path).unwrap();
        let mut archive = ::zip::ZipWriter::new(file);
        let options = ::zip::write::SimpleFileOptions::default();
        archive.add_directory("docs/", options).unwrap();
        archive.start_file("docs/readme.txt", options).unwrap();
        archive.write_all(b"hello").unwrap();
        archive.start_file("other.txt", options).unwrap();
        archive.write_all(b"skip").unwrap();
        archive.finish().unwrap();

        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        let selected = vec![("docs".to_owned(), true)];
        extract(&archive_path, &destination, &selected, None).unwrap();

        assert_eq!(
            fs::read(destination.join("docs/readme.txt")).unwrap(),
            b"hello"
        );
        assert!(!destination.join("other.txt").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_parent_paths_during_zip_extraction() {
        let root = temporary_directory();
        let archive_path = root.join("unsafe.zip");
        let file = File::create(&archive_path).unwrap();
        let mut archive = ::zip::ZipWriter::new(file);
        archive
            .start_file(
                "../escaped.txt",
                ::zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
        archive.write_all(b"unsafe").unwrap();
        archive.finish().unwrap();

        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        extract(&archive_path, &destination, &[], None).unwrap();

        assert!(!root.join("escaped.txt").exists());
        assert!(!destination.join("escaped.txt").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn adds_dropped_folder_to_existing_zip() {
        let root = temporary_directory();
        let archive_path = root.join("sample.zip");
        let file = File::create(&archive_path).unwrap();
        let mut archive = ::zip::ZipWriter::new(file);
        archive
            .start_file("existing.txt", ::zip::write::SimpleFileOptions::default())
            .unwrap();
        archive.write_all(b"original").unwrap();
        archive.finish().unwrap();

        let source = root.join("docs");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("nested/readme.txt"), b"added").unwrap();
        assert_eq!(
            add_files(&archive_path, std::slice::from_ref(&source)).unwrap(),
            3
        );

        let entries = load(&archive_path).unwrap();
        assert!(entries.iter().any(|entry| entry.path == "existing.txt"));
        assert!(entries.iter().any(|entry| entry.path == "docs/nested/readme.txt"));
        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        extract(&archive_path, &destination, &[], None).unwrap();
        assert_eq!(fs::read(destination.join("existing.txt")).unwrap(), b"original");
        assert_eq!(
            fs::read(destination.join("docs/nested/readme.txt")).unwrap(),
            b"added"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn deletes_selected_zip_folder_and_preserves_other_entries() {
        let root = temporary_directory();
        let archive_path = root.join("sample.zip");
        let file = File::create(&archive_path).unwrap();
        let mut archive = ::zip::ZipWriter::new(file);
        let options = ::zip::write::SimpleFileOptions::default();
        archive.add_directory("docs/", options).unwrap();
        archive.start_file("docs/readme.txt", options).unwrap();
        archive.write_all(b"remove").unwrap();
        archive.start_file("keep.txt", options).unwrap();
        archive.write_all(b"keep").unwrap();
        archive.finish().unwrap();

        let selected = vec![("docs".to_owned(), true)];
        assert_eq!(delete_entries(&archive_path, &selected).unwrap(), 2);
        let entries = load(&archive_path).unwrap();
        assert!(!entries.iter().any(|entry| entry.path.starts_with("docs")));
        assert!(entries.iter().any(|entry| entry.path == "keep.txt"));
        fs::remove_dir_all(root).unwrap();
    }
}
