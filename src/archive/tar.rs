use std::{
    error::Error,
    fs::File,
    io::Read,
    path::Path,
};

use super::{ArchiveEntry, path};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::temporary_directory;

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
}
