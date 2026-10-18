//! Read-only FreeArc archive support.
//!
//! FreeArc relies on the external `unarc` executable. Listing and extraction
//! are isolated from command execution, listing parsing, and path validation
//! so unsafe archive names are rejected before extraction.

mod absolute_path;
mod entry;
mod extract;
mod has_timestamp_prefix;
mod list_archive;
mod load;
mod parse_listing;
mod run_unarc;
mod validate_relative_path;

pub(super) use extract::extract;
pub(super) use load::load;

#[cfg(test)]
mod tests {
    use super::{
        parse_listing::parse_listing,
        validate_relative_path::validate_relative_path,
    };

    #[test]
    fn parses_file_and_directory_rows_and_keeps_spaces_in_names() {
        let output = concat!(
            "UnArc 0.60RC unpacker\n",
            ". Listing archive: sample.arc\n",
            "Date/time                  Size Filename\n",
            "----------------------------------------\n",
            "2024-01-02 03:04:05         123 folder/my file.txt\n",
            "2024-01-02 03:04:05       -dir- folder\n",
            "----------------------------------------\n",
            "1 files, 123 bytes, 100 compressed\n",
        );

        let entries = parse_listing(output).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].entry.path, "folder/my file.txt");
        assert_eq!(entries[0].entry.size, 123);
        assert!(!entries[0].entry.is_dir);
        assert_eq!(entries[1].entry.path, "folder");
        assert!(entries[1].entry.is_dir);
    }

    #[test]
    fn rejects_unsafe_extraction_paths() {
        assert!(validate_relative_path("../outside.txt").is_err());
        assert!(validate_relative_path("folder/../../outside.txt").is_err());
        assert!(validate_relative_path(r"folder\..\outside.txt").is_err());
        assert!(validate_relative_path("/outside.txt").is_err());
        assert!(validate_relative_path("folder/readme.txt").is_ok());
    }
}
