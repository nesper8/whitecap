use std::{
    error::Error,
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use super::{
    ArchiveEntry,
    delete,
    path,
    rewrite,
    sources::SourceEntry,
};

pub(super) fn load(path: &Path) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    let mut archive = ::zip::ZipArchive::new(File::open(path)?)?;
    let mut entries = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let entry = archive.by_index_raw(index)?;
        entries.push(ArchiveEntry {
            path: path::normalize(entry.name()),
            is_dir: entry.is_dir(),
            size: entry.size(),
            encrypted: entry.encrypted(),
        });
    }
    Ok(entries)
}

pub(super) fn extract(
    archive_path: &Path,
    destination: &Path,
    selected: &[(String, bool)],
    password: Option<&str>,
) -> Result<usize, Box<dyn Error>> {
    let mut archive = ::zip::ZipArchive::new(File::open(archive_path)?)?;
    let mut count = 0;
    for index in 0..archive.len() {
        let relative_path = {
            let entry = archive.by_index_raw(index)?;
            let Some(relative_path) = entry.enclosed_name() else {
                continue;
            };
            let relative = path::normalize(&relative_path.to_string_lossy());
            if !path::is_selected(&relative, selected) {
                continue;
            }
            relative_path
        };
        let mut entry = match password {
            Some(password) => archive
                .by_index_decrypt(index, password.as_bytes())
                .map_err(map_password_error)?,
            None => match archive.by_index(index) {
                Ok(entry) => entry,
                Err(::zip::result::ZipError::UnsupportedArchive(
                    ::zip::result::ZipError::PASSWORD_REQUIRED,
                )) => return Err(super::PasswordError::Required.into()),
                Err(error) => return Err(error.into()),
            },
        };
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

fn map_password_error(error: ::zip::result::ZipError) -> Box<dyn Error> {
    match error {
        ::zip::result::ZipError::UnsupportedArchive(::zip::result::ZipError::PASSWORD_REQUIRED) => {
            super::PasswordError::Required.into()
        }
        ::zip::result::ZipError::InvalidPassword => super::PasswordError::Invalid.into(),
        error => error.into(),
    }
}

pub(super) fn add(archive_path: &Path, additions: &[SourceEntry]) -> Result<(), Box<dyn Error>> {
    let mut temporary = rewrite::replacement_file(archive_path)?;
    io::copy(&mut File::open(archive_path)?, temporary.as_file_mut())?;

    let mut writer = ::zip::ZipWriter::new_append(temporary.reopen()?)?;
    let options = ::zip::write::SimpleFileOptions::default();
    for entry in additions {
        if entry.is_dir {
            writer.add_directory(
                entry.archive_path.trim_end_matches('/'),
                options,
            )?;
        } else {
            writer.start_file(&entry.archive_path, options)?;
            io::copy(&mut File::open(&entry.source)?, &mut writer)?;
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
    let temporary = rewrite::replacement_file(archive_path)?;
    let mut archive = ::zip::ZipArchive::new(File::open(archive_path)?)?;
    let mut writer = ::zip::ZipWriter::new(temporary.reopen()?);
    let mut count = 0;

    for index in 0..archive.len() {
        let entry = archive.by_index_raw(index)?;
        let entry_path = path::normalize(entry.name());
        if delete::is_selected(&entry_path, selected) {
            count += 1;
        } else {
            writer.raw_copy_file(entry)?;
        }
    }
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
