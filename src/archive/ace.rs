use std::{
    error::Error,
    fs::{self, File},
    io,
    path::{Component, Path, PathBuf},
};

use unarc_rs::ace::AceArchive;

use super::{ArchiveEntry, path};

const MAX_ENTRY_SIZE: u64 = unarc_rs::DEFAULT_MAX_ENTRY_SIZE;

pub(super) fn load(path: &Path) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    let mut archive = AceArchive::new(File::open(path)?)?;
    ensure_supported_archive(&archive)?;

    let mut entries = Vec::new();
    while let Some(header) = archive.get_next_entry()? {
        ensure_supported_entry(&header)?;
        entries.push(ArchiveEntry {
            path: normalized_entry_path(&header.filename),
            is_dir: header.is_directory(),
            size: header.original_size,
            encrypted: header.is_encrypted(),
        });
        archive.skip(&header)?;
    }
    Ok(entries)
}

pub(super) fn extract(
    archive_path: &Path,
    destination: &Path,
    selected: &[(String, bool)],
    password: Option<&str>,
) -> Result<usize, Box<dyn Error>> {
    let mut archive = AceArchive::new(File::open(archive_path)?)?;
    ensure_supported_archive(&archive)?;
    if let Some(password) = password {
        archive.set_password(password);
    }
    let solid = archive.is_solid();
    let mut count = 0;

    while let Some(header) = archive.get_next_entry()? {
        ensure_supported_entry(&header)?;
        let relative = normalized_entry_path(&header.filename);
        let selected_entry = path::is_selected(&relative, selected);

        if !selected_entry && !(solid && !header.is_directory()) {
            archive.skip(&header)?;
            continue;
        }
        if header.is_directory() {
            let output = destination.join(safe_relative_path(&header.filename)?);
            fs::create_dir_all(output)?;
            if selected_entry {
                count += 1;
            }
            archive.skip(&header)?;
            continue;
        }
        if selected_entry && header.is_encrypted() && password.is_none() {
            return Err(super::PasswordError::Required.into());
        }
        if header.original_size > MAX_ENTRY_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "ACE entry '{}' exceeds the {}-byte extraction limit",
                    header.filename, MAX_ENTRY_SIZE
                ),
            )
            .into());
        }

        let data = archive
            .read(&header)
            .map_err(|error| map_password_error(error, password.is_some()))?;
        if selected_entry {
            let output = destination.join(safe_relative_path(&header.filename)?);
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(output, data)?;
            count += 1;
        }
    }

    Ok(count)
}

fn ensure_supported_archive<R: io::Read + io::Seek>(
    archive: &AceArchive<R>,
) -> io::Result<()> {
    let main_header = archive.main_header();
    if main_header.is_multivolume() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Multi-volume ACE archives are not supported",
        ));
    }
    Ok(())
}

fn ensure_supported_entry(header: &unarc_rs::ace::FileHeader) -> io::Result<()> {
    if header.is_continued_from_prev() || header.is_continued_to_next() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Multi-volume ACE entries are not supported",
        ));
    }
    Ok(())
}

fn map_password_error(
    error: unarc_rs::error::ArchiveError,
    password_was_provided: bool,
) -> Box<dyn Error> {
    match error {
        unarc_rs::error::ArchiveError::EncryptionRequired { .. } => {
            super::PasswordError::Required.into()
        }
        unarc_rs::error::ArchiveError::InvalidPassword { .. }
        | unarc_rs::error::ArchiveError::CrcMismatch { .. }
            if password_was_provided =>
        {
            super::PasswordError::Invalid.into()
        }
        error => error.into(),
    }
}

fn safe_relative_path(name: &str) -> io::Result<PathBuf> {
    let normalized = name.replace('\\', "/");
    let mut relative = PathBuf::new();
    for component in Path::new(&normalized).components() {
        match component {
            Component::Normal(part) => {
                if relative.as_os_str().is_empty()
                    && part
                        .to_str()
                        .is_some_and(|part| part.len() >= 2 && part.as_bytes()[1] == b':')
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Unsafe ACE entry path: {name}"),
                    ));
                }
                relative.push(part);
            }
            Component::CurDir => {}
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unsafe ACE entry path: {name}"),
                ));
            }
        }
    }
    if relative.as_os_str().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unsafe ACE entry path: {name}"),
        ));
    }
    Ok(relative)
}

fn normalized_entry_path(name: &str) -> String {
    path::normalize(&name.replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(extract(&archive_path, &destination, &selected, None).unwrap(), 1);
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
