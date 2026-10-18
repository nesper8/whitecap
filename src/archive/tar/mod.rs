//! TAR and compressed-TAR archive support.
//!
//! TAR uses a shared reader and separate rewrite paths for each compression
//! variant. The public functions here are the format-dispatch boundary; the
//! implementation details live in focused submodules.

mod add;
mod append_existing_entries;
mod append_selected_entries;
mod append_source_entries;
mod compress_tar;
mod delete;
mod extract;
mod load;
mod reader;

#[derive(Clone, Copy)]
/// Compression codecs supported by TAR readers and writers.
pub(super) enum Compression {
    /// An uncompressed TAR stream.
    None,
    /// Gzip-compressed TAR data.
    Gzip,
    /// Bzip2-compressed TAR data.
    Bzip2,
    /// XZ-compressed TAR data.
    Xz,
    /// Zstandard-compressed TAR data.
    Zstd,
}

pub(super) use add::add;
pub(super) use delete::delete;
pub(super) use extract::extract;
pub(super) use load::load;

#[cfg(test)]
mod tests {
    use std::{fs, fs::File};

    use super::*;
    use crate::archive::{add_files, delete_entries, temporary_directory};

    #[test]
    fn extracts_selected_tar_entry() {
        let root = temporary_directory();
        let archive_path = root.join("sample.tar");
        let file = File::create(&archive_path).unwrap();
        let mut archive = ::tar::Builder::new(file);
        let mut header = ::tar::Header::new_gnu();
        header.set_size(5);
        header.set_mode(0o644);
        header.set_cksum();
        archive
            .append_data(&mut header, "docs/readme.txt", &b"hello"[..])
            .unwrap();
        archive.finish().unwrap();

        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        let selected = vec![("docs/readme.txt".to_owned(), false)];
        extract(
            &archive_path,
            &destination,
            &selected,
            Compression::None,
        )
        .unwrap();

        assert_eq!(
            fs::read(destination.join("docs/readme.txt")).unwrap(),
            b"hello"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn adds_dropped_file_to_compressed_tar() {
        let root = temporary_directory();
        let archive_path = root.join("sample.tar.gz");
        {
            let file = File::create(&archive_path).unwrap();
            let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
            let mut archive = ::tar::Builder::new(encoder);
            let mut header = ::tar::Header::new_gnu();
            header.set_size(8);
            header.set_mode(0o644);
            header.set_cksum();
            archive
                .append_data(&mut header, "existing.txt", &b"original"[..])
                .unwrap();
            archive.into_inner().unwrap().finish().unwrap();
        }

        let added = root.join("added.txt");
        fs::write(&added, b"added").unwrap();
        assert_eq!(
            add_files(&archive_path, std::slice::from_ref(&added)).unwrap(),
            1
        );

        let entries = load(&archive_path, Compression::Gzip).unwrap();
        assert!(entries.iter().any(|entry| entry.path == "existing.txt"));
        assert!(entries.iter().any(|entry| entry.path == "added.txt"));
        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        extract(&archive_path, &destination, &[], Compression::Gzip).unwrap();
        assert_eq!(fs::read(destination.join("existing.txt")).unwrap(), b"original");
        assert_eq!(fs::read(destination.join("added.txt")).unwrap(), b"added");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn deletes_selected_tar_entry_and_preserves_other_entries() {
        let root = temporary_directory();
        let archive_path = root.join("sample.tar");
        let file = File::create(&archive_path).unwrap();
        let mut archive = ::tar::Builder::new(file);
        for (name, data) in [("remove.txt", b"remove".as_slice()), ("keep.txt", b"keep")] {
            let mut header = ::tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            archive.append_data(&mut header, name, data).unwrap();
        }
        archive.finish().unwrap();

        let selected = vec![("remove.txt".to_owned(), false)];
        assert_eq!(delete_entries(&archive_path, &selected).unwrap(), 1);
        let entries = load(&archive_path, Compression::None).unwrap();
        assert!(!entries.iter().any(|entry| entry.path == "remove.txt"));
        assert!(entries.iter().any(|entry| entry.path == "keep.txt"));
        fs::remove_dir_all(root).unwrap();
    }
}
