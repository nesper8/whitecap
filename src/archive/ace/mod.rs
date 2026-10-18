//! ACE archive support.
//!
//! ACE archives are available for listing and extraction only. The submodules
//! keep format validation, metadata loading, extraction, and error mapping
//! separate while this module exposes the dispatch functions used by the
//! archive layer.

mod ensure_supported_archive;
mod ensure_supported_entry;
mod extract;
mod load;
mod map_password_error;
mod normalized_entry_path;
mod safe_relative_path;

/// Maximum uncompressed ACE entry size accepted during extraction.
const MAX_ENTRY_SIZE: u64 = unarc_rs::DEFAULT_MAX_ENTRY_SIZE;

pub(super) use extract::extract;
pub(super) use load::load;

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        load::load,
        safe_relative_path::safe_relative_path,
    };
    use crate::archive::temporary_directory;

    #[test]
    fn rejects_unsafe_paths() {
        for name in ["../outside.txt", "/absolute.txt", r"..\outside.txt", "C:\\outside.txt"] {
            assert!(safe_relative_path(name).is_err(), "accepted {name}");
        }
    }

    #[test]
    fn lists_and_extracts_stored_ace_entries() {
        let root = temporary_directory();
        let archive_path = root.join("sample.ace");
        fs::write(&archive_path, stored_archive("hello.txt", b"hello")).unwrap();

        let entries = load(&archive_path).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "hello.txt");
        assert_eq!(entries[0].size, 5);

        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        let selected = vec![("hello.txt".to_owned(), false)];
        assert_eq!(
            super::extract::extract(&archive_path, &destination, &selected, None).unwrap(),
            1
        );
        assert_eq!(fs::read(destination.join("hello.txt")).unwrap(), b"hello");
        fs::remove_dir_all(root).unwrap();
    }

    fn stored_archive(name: &str, data: &[u8]) -> Vec<u8> {
        let mut archive = Vec::new();

        let mut main_header = vec![0];
        main_header.extend_from_slice(&0u16.to_le_bytes());
        main_header.extend_from_slice(b"**ACE**");
        main_header.extend_from_slice(&[10, 10, 0, 0]);
        main_header.extend_from_slice(&[0; 12]);
        append_header(&mut archive, &main_header);

        let mut file_header = vec![1];
        file_header.extend_from_slice(&0u16.to_le_bytes());
        file_header.extend_from_slice(&(data.len() as u32).to_le_bytes());
        file_header.extend_from_slice(&(data.len() as u32).to_le_bytes());
        file_header.extend_from_slice(&[0; 8]);
        file_header.extend_from_slice(&ace_crc32(data).to_le_bytes());
        file_header.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
        file_header.extend_from_slice(&(name.len() as u16).to_le_bytes());
        file_header.extend_from_slice(name.as_bytes());
        append_header(&mut archive, &file_header);
        archive.extend_from_slice(data);

        archive
    }

    fn append_header(archive: &mut Vec<u8>, header: &[u8]) {
        archive.extend_from_slice(&(ace_crc32(header) as u16).to_le_bytes());
        archive.extend_from_slice(&(header.len() as u16).to_le_bytes());
        archive.extend_from_slice(header);
    }

    fn ace_crc32(data: &[u8]) -> u32 {
        let mut crc = 0xffff_ffff;
        for byte in data {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = if crc & 1 != 0 {
                    (crc >> 1) ^ 0xedb8_8320
                } else {
                    crc >> 1
                };
            }
        }
        crc
    }
}
