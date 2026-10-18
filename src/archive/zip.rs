use std::{
    error::Error,
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use super::{ArchiveEntry, path};

pub(super) fn load(path: &Path) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    let mut archive = ::zip::ZipArchive::new(File::open(path)?)?;
    let mut entries = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        entries.push(ArchiveEntry {
            path: path::normalize(entry.name()),
            is_dir: entry.is_dir(),
            size: entry.size(),
        });
    }
    Ok(entries)
}

pub(super) fn extract(
    archive_path: &Path,
    destination: &Path,
    selected: &[(String, bool)],
) -> Result<usize, Box<dyn Error>> {
    let mut archive = ::zip::ZipArchive::new(File::open(archive_path)?)?;
    let mut count = 0;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let Some(relative_path) = entry.enclosed_name() else {
            continue;
        };
        let relative = path::normalize(&relative_path.to_string_lossy());
        if !path::is_selected(&relative, selected) {
            continue;
        }
        let output = destination.join(relative_path);
        if entry.is_dir() {
            fs::create_dir_all(output)?;
        } else {
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut file = File::create(output)?;
            io::copy(&mut entry, &mut file)?;
            file.flush()?;
        }
        count += 1;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::temporary_directory;

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
        extract(&archive_path, &destination, &selected).unwrap();

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
        extract(&archive_path, &destination, &[]).unwrap();

        assert!(!root.join("escaped.txt").exists());
        assert!(!destination.join("escaped.txt").exists());
        fs::remove_dir_all(root).unwrap();
    }
}
