mod ace;
mod add;
mod delete;
mod path;
mod rar;
mod sevenz;
mod tar;
mod zip;
mod freearc;

use std::{error::Error, io, path::{Path, PathBuf}};

#[derive(Clone)]
pub struct ArchiveEntry {
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

#[cfg(test)]
pub(super) fn temporary_directory() -> std::path::PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("whitecap-test-{}-{unique}", std::process::id()));
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[derive(Clone, Copy)]
enum ArchiveKind {
    Zip,
    Tar(tar::Compression),
    SevenZip,
    Rar,
    FreeArc,
    Ace,
}

pub fn load(path: &Path) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    match archive_kind(path)? {
        ArchiveKind::Zip => zip::load(path),
        ArchiveKind::Tar(compression) => tar::load(path, compression),
        ArchiveKind::SevenZip => sevenz::load(path),
        ArchiveKind::Rar => rar::load(path),
        ArchiveKind::FreeArc => freearc::load(path),
        ArchiveKind::Ace => ace::load(path),
    }
}

pub fn extract(
    archive_path: &Path,
    destination: &Path,
    selected: &[(String, bool)],
) -> Result<usize, Box<dyn Error>> {
    match archive_kind(archive_path)? {
        ArchiveKind::Zip => zip::extract(archive_path, destination, selected),
        ArchiveKind::Tar(compression) => {
            tar::extract(archive_path, destination, selected, compression)
        }
        ArchiveKind::SevenZip => sevenz::extract(archive_path, destination, selected),
        ArchiveKind::Rar => rar::extract(archive_path, destination, selected),
        ArchiveKind::FreeArc => freearc::extract(archive_path, destination, selected),
        ArchiveKind::Ace => ace::extract(archive_path, destination, selected),
    }
}

pub fn add_files(archive_path: &Path, sources: &[PathBuf]) -> Result<usize, Box<dyn Error>> {
    add::add_files(archive_path, sources)
}

pub fn supports_modification(path: &Path) -> Result<bool, io::Error> {
    Ok(matches!(
        archive_kind(path)?,
        ArchiveKind::Zip | ArchiveKind::SevenZip | ArchiveKind::Tar(_)
    ))
}

pub fn delete_entries(
    archive_path: &Path,
    selected: &[(String, bool)],
) -> Result<usize, Box<dyn Error>> {
    delete::delete_entries(archive_path, selected)
}

fn archive_kind(path: &Path) -> Result<ArchiveKind, io::Error> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_lowercase();
    if name.ends_with(".zip") {
        Ok(ArchiveKind::Zip)
    } else if name.ends_with(".7z") {
        Ok(ArchiveKind::SevenZip)
    } else if name.ends_with(".rar") {
        Ok(ArchiveKind::Rar)
    } else if name.ends_with(".arc") {
        Ok(ArchiveKind::FreeArc)
    } else if name.ends_with(".ace") {
        Ok(ArchiveKind::Ace)
    } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        Ok(ArchiveKind::Tar(tar::Compression::Gzip))
    } else if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") {
        Ok(ArchiveKind::Tar(tar::Compression::Bzip2))
    } else if name.ends_with(".tar.xz") || name.ends_with(".txz") {
        Ok(ArchiveKind::Tar(tar::Compression::Xz))
    } else if name.ends_with(".tar.zst") || name.ends_with(".tzst") {
        Ok(ArchiveKind::Tar(tar::Compression::Zstd))
    } else if name.ends_with(".tar") {
        Ok(ArchiveKind::Tar(tar::Compression::None))
    } else {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Unsupported archive format. Supported formats are ZIP, RAR4/RAR5, 7z, ACE, FreeArc and TAR (including gzip, bzip2, xz and zstd).",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_rar_and_7z_extensions_case_insensitively() {
        assert!(matches!(
            archive_kind(Path::new("archive.RAR")).unwrap(),
            ArchiveKind::Rar
        ));
        assert!(matches!(
            archive_kind(Path::new("archive.7Z")).unwrap(),
            ArchiveKind::SevenZip
        ));
        assert!(matches!(
            archive_kind(Path::new("archive.ARC")).unwrap(),
            ArchiveKind::FreeArc
        ));
        assert!(matches!(
            archive_kind(Path::new("archive.ACE")).unwrap(),
            ArchiveKind::Ace
        ));
    }

    #[test]
    fn allows_modification_only_for_supported_writer_formats() {
        for name in ["archive.zip", "archive.7z", "archive.tar", "archive.tar.gz"] {
            assert!(supports_modification(Path::new(name)).unwrap(), "{name}");
        }
        for name in ["archive.rar", "archive.ace", "archive.arc"] {
            assert!(!supports_modification(Path::new(name)).unwrap(), "{name}");
        }
    }
}
