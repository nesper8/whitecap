use std::{
    error::Error,
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

use super::{ArchiveEntry, add::{self, SourceEntry}, delete, path};

#[derive(Clone, Copy)]
pub(super) enum Compression {
    None,
    Gzip,
    Bzip2,
    Xz,
    Zstd,
}

pub(super) fn load(
    path: &Path,
    compression: Compression,
) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    let reader = reader(path, compression)?;
    let mut archive = ::tar::Archive::new(reader);
    let mut entries = Vec::new();
    for entry in archive.entries()? {
        let entry = entry?;
        entries.push(ArchiveEntry {
            path: path::normalize(&entry.path()?.to_string_lossy()),
            is_dir: entry.header().entry_type().is_dir(),
            size: entry.size(),
        });
    }
    Ok(entries)
}

pub(super) fn extract(
    archive_path: &Path,
    destination: &Path,
    selected: &[(String, bool)],
    compression: Compression,
) -> Result<usize, Box<dyn Error>> {
    let reader = reader(archive_path, compression)?;
    let mut archive = ::tar::Archive::new(reader);
    let mut count = 0;
    for entry in archive.entries()? {
        let mut entry = entry?;
        let relative = path::normalize(&entry.path()?.to_string_lossy());
        if path::is_selected(&relative, selected) && entry.unpack_in(destination)? {
            count += 1;
        }
    }
    Ok(count)
}

fn reader(path: &Path, compression: Compression) -> Result<Box<dyn Read>, Box<dyn Error>> {
    let file = File::open(path)?;
    Ok(match compression {
        Compression::None => Box::new(file),
        Compression::Gzip => Box::new(flate2::read::GzDecoder::new(file)),
        Compression::Bzip2 => Box::new(bzip2::read::BzDecoder::new(file)),
        Compression::Xz => Box::new(xz2::read::XzDecoder::new(file)),
        Compression::Zstd => Box::new(zstd::stream::read::Decoder::new(file)?),
    })
}

pub(super) fn add(
    archive_path: &Path,
    additions: &[SourceEntry],
    compression: Compression,
) -> Result<(), Box<dyn Error>> {
    let output = add::replacement_file(archive_path)?;

    if matches!(compression, Compression::None) {
        let mut builder = ::tar::Builder::new(output.reopen()?);
        append_existing_entries(archive_path, compression, &mut builder)?;
        append_source_entries(additions, &mut builder)?;
        builder.into_inner()?.sync_all()?;
    } else {
        let uncompressed = tempfile::NamedTempFile::new()?;
        let mut builder = ::tar::Builder::new(uncompressed.reopen()?);
        append_existing_entries(archive_path, compression, &mut builder)?;
        append_source_entries(additions, &mut builder)?;
        builder.into_inner()?.sync_all()?;

        compress_tar(uncompressed.path(), output.reopen()?, compression)?;
    }
    output.persist(archive_path)?;
    Ok(())
}

pub(super) fn delete(
    archive_path: &Path,
    selected: &[(String, bool)],
    compression: Compression,
) -> Result<usize, Box<dyn Error>> {
    let output = add::replacement_file(archive_path)?;
    let mut count = 0;
    if matches!(compression, Compression::None) {
        let mut builder = ::tar::Builder::new(output.reopen()?);
        append_selected_entries(archive_path, compression, selected, &mut count, &mut builder)?;
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "None of the selected entries were found in the archive",
            )
            .into());
        }
        builder.into_inner()?.sync_all()?;
    } else {
        let uncompressed = tempfile::NamedTempFile::new()?;
        let mut builder = ::tar::Builder::new(uncompressed.reopen()?);
        append_selected_entries(archive_path, compression, selected, &mut count, &mut builder)?;
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "None of the selected entries were found in the archive",
            )
            .into());
        }
        builder.into_inner()?.sync_all()?;
        compress_tar(uncompressed.path(), output.reopen()?, compression)?;
    }
    output.persist(archive_path)?;
    Ok(count)
}

fn compress_tar(
    source: &Path,
    output: File,
    compression: Compression,
) -> Result<(), Box<dyn Error>> {
    let mut input = File::open(source)?;
    match compression {
        Compression::Gzip => {
            let mut encoder =
                flate2::write::GzEncoder::new(output, flate2::Compression::default());
            io::copy(&mut input, &mut encoder)?;
            encoder.finish()?.sync_all()?;
        }
        Compression::Bzip2 => {
            let mut encoder =
                bzip2::write::BzEncoder::new(output, bzip2::Compression::default());
            io::copy(&mut input, &mut encoder)?;
            encoder.finish()?.sync_all()?;
        }
        Compression::Xz => {
            let mut encoder = xz2::write::XzEncoder::new(output, 6);
            io::copy(&mut input, &mut encoder)?;
            encoder.finish()?.sync_all()?;
        }
        Compression::Zstd => {
            let mut encoder = zstd::stream::write::Encoder::new(output, 0)?;
            io::copy(&mut input, &mut encoder)?;
            encoder.finish()?.sync_all()?;
        }
        Compression::None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot compress an uncompressed TAR archive",
            )
            .into());
        }
    }
    Ok(())
}

fn append_existing_entries<W: Write>(
    archive_path: &Path,
    compression: Compression,
    builder: &mut ::tar::Builder<W>,
) -> Result<(), Box<dyn Error>> {
    let mut archive = ::tar::Archive::new(reader(archive_path, compression)?);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let header = entry.header().clone();
        builder.append(&header, &mut entry)?;
    }
    Ok(())
}

fn append_selected_entries<W: Write>(
    archive_path: &Path,
    compression: Compression,
    selected: &[(String, bool)],
    count: &mut usize,
    builder: &mut ::tar::Builder<W>,
) -> Result<(), Box<dyn Error>> {
    let mut archive = ::tar::Archive::new(reader(archive_path, compression)?);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_path = path::normalize(&entry.path()?.to_string_lossy());
        if delete::is_selected(&entry_path, selected) {
            *count += 1;
            io::copy(&mut entry, &mut io::sink())?;
        } else {
            let header = entry.header().clone();
            builder.append(&header, &mut entry)?;
        }
    }
    Ok(())
}

fn append_source_entries<W: Write>(
    additions: &[SourceEntry],
    builder: &mut ::tar::Builder<W>,
) -> Result<(), Box<dyn Error>> {
    for entry in additions {
        let archive_path = entry.archive_path.trim_end_matches('/');
        if entry.is_dir {
            builder.append_dir(archive_path, &entry.source)?;
        } else {
            builder.append_path_with_name(&entry.source, archive_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::{add_files, delete_entries, temporary_directory};
    use std::fs;

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
        std::fs::create_dir_all(&destination).unwrap();
        let selected = vec![("docs/readme.txt".to_owned(), false)];
        extract(
            &archive_path,
            &destination,
            &selected,
            Compression::None,
        )
        .unwrap();

        assert_eq!(
            std::fs::read(destination.join("docs/readme.txt")).unwrap(),
            b"hello"
        );
        std::fs::remove_dir_all(root).unwrap();
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
