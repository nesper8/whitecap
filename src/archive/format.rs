use std::{io, path::Path};

#[derive(Clone, Copy)]
pub(super) enum ArchiveKind {
    Zip,
    Tar(super::tar::Compression),
    SevenZip,
    Rar,
    FreeArc,
    Ace,
}

pub(super) fn archive_kind(path: &Path) -> Result<ArchiveKind, io::Error> {
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
        Ok(ArchiveKind::Tar(super::tar::Compression::Gzip))
    } else if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") {
        Ok(ArchiveKind::Tar(super::tar::Compression::Bzip2))
    } else if name.ends_with(".tar.xz") || name.ends_with(".txz") {
        Ok(ArchiveKind::Tar(super::tar::Compression::Xz))
    } else if name.ends_with(".tar.zst") || name.ends_with(".tzst") {
        Ok(ArchiveKind::Tar(super::tar::Compression::Zstd))
    } else if name.ends_with(".tar") {
        Ok(ArchiveKind::Tar(super::tar::Compression::None))
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
    fn detects_archive_extensions_case_insensitively() {
        assert!(matches!(
            archive_kind(Path::new("archive.ZIP")),
            Ok(ArchiveKind::Zip)
        ));
        assert!(matches!(
            archive_kind(Path::new("archive.RAR")),
            Ok(ArchiveKind::Rar)
        ));
        assert!(matches!(
            archive_kind(Path::new("archive.7Z")),
            Ok(ArchiveKind::SevenZip)
        ));
        assert!(matches!(
            archive_kind(Path::new("archive.ARC")),
            Ok(ArchiveKind::FreeArc)
        ));
        assert!(matches!(
            archive_kind(Path::new("archive.ACE")),
            Ok(ArchiveKind::Ace)
        ));
    }

    #[test]
    fn recognizes_supported_tar_compression_extensions() {
        for name in [
            "archive.tar",
            "archive.tar.gz",
            "archive.tgz",
            "archive.tar.bz2",
            "archive.tbz2",
            "archive.tar.xz",
            "archive.txz",
            "archive.tar.zst",
            "archive.tzst",
        ] {
            assert!(
                matches!(
                    archive_kind(Path::new(name)),
                    Ok(ArchiveKind::Tar(_))
                ),
                "{name}"
            );
        }
    }

    #[test]
    fn allows_modification_only_for_supported_writer_formats() {
        for name in ["archive.zip", "archive.7z", "archive.tar", "archive.tar.gz"] {
            assert!(
                super::super::operations::supports_modification(Path::new(name)).unwrap(),
                "{name}"
            );
        }
        for name in ["archive.rar", "archive.ace", "archive.arc"] {
            assert!(
                !super::super::operations::supports_modification(Path::new(name)).unwrap(),
                "{name}"
            );
        }
    }
}
