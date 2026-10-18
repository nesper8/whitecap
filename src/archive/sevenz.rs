use std::{
    error::Error,
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use super::{ArchiveEntry, add::{self, SourceEntry}, delete, path};

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

pub(super) fn add(archive_path: &Path, additions: &[SourceEntry]) -> Result<(), Box<dyn Error>> {
    let temporary = add::replacement_file(archive_path)?;
    let mut writer = sevenz_rust2::ArchiveWriter::create(temporary.path())?;
    let mut reader = sevenz_rust2::ArchiveReader::open(
        archive_path,
        sevenz_rust2::Password::empty(),
    )?;
    reader.for_each_entries(|entry, data| {
        writer.push_archive_entry(entry.clone(), Some(data))?;
        Ok(true)
    })?;
    for entry in additions {
        let name = entry.archive_path.trim_end_matches('/').to_owned();
        let archive_entry = sevenz_rust2::ArchiveEntry::from_path(&entry.source, name);
        if entry.is_dir {
            writer.push_archive_entry(archive_entry, None::<File>)?;
        } else {
            writer.push_archive_entry(archive_entry, Some(File::open(&entry.source)?))?;
        }
    }
    writer.finish()?.flush()?;
    temporary.persist(archive_path)?;
    Ok(())
}

pub(super) fn delete(
    archive_path: &Path,
    selected: &[(String, bool)],
) -> Result<usize, Box<dyn Error>> {
    let temporary = add::replacement_file(archive_path)?;
    let mut writer = sevenz_rust2::ArchiveWriter::create(temporary.path())?;
    let mut reader = sevenz_rust2::ArchiveReader::open(
        archive_path,
        sevenz_rust2::Password::empty(),
    )?;
    let mut count = 0;
    reader.for_each_entries(|entry, data| {
        let entry_path = path::normalize(entry.name());
        if delete::is_selected(&entry_path, selected) {
            count += 1;
            io::copy(data, &mut io::sink())?;
        } else {
            writer.push_archive_entry(entry.clone(), Some(data))?;
        }
        Ok(true)
    })?;
    if count == 0 {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "None of the selected entries were found in the archive",
        )
        .into());
    }
    writer.finish()?.flush()?;
    temporary.persist(archive_path)?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::{add_files, delete_entries, temporary_directory};

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

    #[test]
    fn adds_files_and_preserves_existing_7z_entries() {
        let root = temporary_directory();
        let original = root.join("original");
        fs::create_dir_all(&original).unwrap();
        fs::write(original.join("existing.txt"), b"old").unwrap();
        let archive_path = root.join("sample.7z");
        sevenz_rust2::compress_to_path(&original, &archive_path).unwrap();

        let added = root.join("added.txt");
        fs::write(&added, b"new").unwrap();
        assert_eq!(add_files(&archive_path, std::slice::from_ref(&added)).unwrap(), 1);

        let archive_entries = load(&archive_path).unwrap();
        assert!(archive_entries.iter().any(|entry| entry.path == "existing.txt"));
        assert!(archive_entries.iter().any(|entry| entry.path == "added.txt"));
        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        extract(&archive_path, &destination, &[]).unwrap();
        assert_eq!(fs::read(destination.join("existing.txt")).unwrap(), b"old");
        assert_eq!(fs::read(destination.join("added.txt")).unwrap(), b"new");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn deletes_selected_7z_folder_and_preserves_other_entries() {
        let root = temporary_directory();
        let source = root.join("source");
        fs::create_dir_all(source.join("docs")).unwrap();
        fs::write(source.join("docs/readme.txt"), b"remove").unwrap();
        fs::write(source.join("keep.txt"), b"keep").unwrap();
        let archive_path = root.join("sample.7z");
        sevenz_rust2::compress_to_path(&source, &archive_path).unwrap();

        let selected = vec![("docs".to_owned(), true)];
        assert_eq!(delete_entries(&archive_path, &selected).unwrap(), 2);
        let entries = load(&archive_path).unwrap();
        assert!(!entries.iter().any(|entry| entry.path.starts_with("docs")));
        assert!(entries.iter().any(|entry| entry.path == "keep.txt"));
        fs::remove_dir_all(root).unwrap();
    }
}
