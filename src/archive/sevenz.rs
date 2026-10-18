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

pub(super) fn load_with_password(
    path: &Path,
    password: Option<&str>,
) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    let archive = sevenz_rust2::ArchiveReader::open(
        path,
        sevenz_rust2::Password::new(password.unwrap_or_default()),
    )
    .map_err(|error| map_password_error(error, password.is_some()))?;
    let encrypted = archive.archive().blocks.iter().any(|block| {
        block.coders.iter().any(|coder| {
            coder.encoder_method_id() == sevenz_rust2::EncoderMethod::ID_AES256_SHA256
        })
    });
    Ok(archive
        .archive()
        .files
        .iter()
        .map(|entry| ArchiveEntry {
            path: path::normalize(entry.name()),
            is_dir: entry.is_directory(),
            size: entry.size(),
            encrypted: !entry.is_directory() && encrypted,
        })
        .collect())
}

pub(super) fn extract(
    archive_path: &Path,
    destination: &Path,
    selected: &[(String, bool)],
    password: Option<&str>,
) -> Result<usize, Box<dyn Error>> {
    let mut count = 0;
    sevenz_rust2::decompress_with_extract_fn_and_password(
        File::open(archive_path)?,
        destination,
        sevenz_rust2::Password::new(password.unwrap_or_default()),
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
    )
    .map_err(|error| map_password_error(error, password.is_some()))?;
    Ok(count)
}

fn map_password_error(
    error: sevenz_rust2::Error,
    password_was_provided: bool,
) -> Box<dyn Error> {
    match error {
        sevenz_rust2::Error::PasswordRequired if password_was_provided => {
            super::PasswordError::Invalid.into()
        }
        sevenz_rust2::Error::PasswordRequired => super::PasswordError::Required.into(),
        sevenz_rust2::Error::MaybeBadPassword(_) if password_was_provided => {
            super::PasswordError::Invalid.into()
        }
        sevenz_rust2::Error::MaybeBadPassword(_) => super::PasswordError::Required.into(),
        error => error.into(),
    }
}

pub(super) fn add(archive_path: &Path, additions: &[SourceEntry]) -> Result<(), Box<dyn Error>> {
    let temporary = rewrite::replacement_file(archive_path)?;
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
    let temporary = rewrite::replacement_file(archive_path)?;
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
        let entries = load_with_password(&archive_path, None).unwrap();
        assert!(entries.iter().any(|entry| entry.path == "docs/readme.txt"));

        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        let selected = vec![("docs/readme.txt".to_owned(), false)];
        extract(&archive_path, &destination, &selected, None).unwrap();

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

        let archive_entries = load_with_password(&archive_path, None).unwrap();
        assert!(archive_entries.iter().any(|entry| entry.path == "existing.txt"));
        assert!(archive_entries.iter().any(|entry| entry.path == "added.txt"));
        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        extract(&archive_path, &destination, &[], None).unwrap();
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
        let entries = load_with_password(&archive_path, None).unwrap();
        assert!(!entries.iter().any(|entry| entry.path.starts_with("docs")));
        assert!(entries.iter().any(|entry| entry.path == "keep.txt"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn prompts_for_password_and_extracts_encrypted_7z_entries() {
        let root = temporary_directory();
        let source = root.join("source");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("secret.txt"), b"private data").unwrap();
        let archive_path = root.join("encrypted.7z");
        sevenz_rust2::compress_to_path_encrypted(
            &source,
            &archive_path,
            sevenz_rust2::Password::new("correct horse"),
        )
        .unwrap();

        let load_error = load_with_password(&archive_path, None).unwrap_err();
        assert!(matches!(
            load_error.downcast_ref::<super::super::PasswordError>(),
            Some(super::super::PasswordError::Required)
        ));
        let entries =
            load_with_password(&archive_path, Some("correct horse")).unwrap();
        assert!(entries.iter().any(|entry| entry.encrypted));
        let selected = vec![("secret.txt".to_owned(), false)];
        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();

        let missing_password = extract(&archive_path, &destination, &selected, None).unwrap_err();
        assert!(matches!(
            missing_password.downcast_ref::<super::super::PasswordError>(),
            Some(super::super::PasswordError::Required)
        ));

        let wrong_password =
            extract(&archive_path, &destination, &selected, Some("wrong")).unwrap_err();
        assert!(matches!(
            wrong_password.downcast_ref::<super::super::PasswordError>(),
            Some(super::super::PasswordError::Invalid)
        ));

        assert_eq!(
            extract(&archive_path, &destination, &selected, Some("correct horse")).unwrap(),
            1
        );
        assert_eq!(
            fs::read(destination.join("secret.txt")).unwrap(),
            b"private data"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
