use std::{
    error::Error,
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use super::{ArchiveEntry, path};

pub(super) fn load(path: &Path) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    let archive =
        sevenz_rust2::ArchiveReader::open(path, sevenz_rust2::Password::empty())?;
    Ok(archive
        .archive()
        .files
        .iter()
        .map(|entry| ArchiveEntry {
            path: path::normalize(entry.name()),
            is_dir: entry.is_directory(),
            size: entry.size(),
        })
        .collect())
}

pub(super) fn extract(
    archive_path: &Path,
    destination: &Path,
    selected: &[(String, bool)],
) -> Result<usize, Box<dyn Error>> {
    let mut count = 0;
    sevenz_rust2::decompress_file_with_extract_fn(
        archive_path,
        destination,
        |entry, reader, output| {
            let relative = path::normalize(entry.name());
            if !path::is_selected(&relative, selected) {
                io::copy(reader, &mut io::sink())?;
                return Ok(true);
            }

            if entry.is_directory() {
                fs::create_dir_all(output)?;
            } else {
                if let Some(parent) = output.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut file = File::create(output)?;
                io::copy(reader, &mut file)?;
                file.flush()?;
            }
            count += 1;
            Ok(true)
        },
    )?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::temporary_directory;

    #[test]
    fn extracts_only_selected_7z_entry() {
        let root = temporary_directory();
        let source = root.join("source");
        fs::create_dir_all(source.join("docs")).unwrap();
        fs::write(source.join("docs/readme.txt"), b"hello").unwrap();
        fs::write(source.join("other.txt"), b"skip").unwrap();

        let archive_path = root.join("sample.7z");
        sevenz_rust2::compress_to_path(&source, &archive_path).unwrap();
        let entries = load(&archive_path).unwrap();
        assert!(entries.iter().any(|entry| entry.path == "docs/readme.txt"));

        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        let selected = vec![("docs/readme.txt".to_owned(), false)];
        extract(&archive_path, &destination, &selected).unwrap();

        assert_eq!(
            fs::read(destination.join("docs/readme.txt")).unwrap(),
            b"hello"
        );
        assert!(!destination.join("other.txt").exists());
        fs::remove_dir_all(root).unwrap();
    }
}
