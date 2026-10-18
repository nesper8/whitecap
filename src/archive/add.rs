use std::{
    collections::HashMap,
    error::Error,
    fs,
    io,
    path::{Path, PathBuf},
};

use tempfile::NamedTempFile;

use super::{ArchiveEntry, archive_kind};

pub(super) struct SourceEntry {
    pub(super) source: PathBuf,
    pub(super) archive_path: String,
    pub(super) is_dir: bool,
}

pub(super) fn add_files(
    archive_path: &Path,
    sources: &[PathBuf],
) -> Result<usize, Box<dyn Error>> {
    if sources.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "No files were dropped").into());
    }

    let kind = archive_kind(archive_path)?;
    if !kind.supports_modification() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Adding files is disabled for RAR, ACE, and FreeArc archives because Whitecap does not modify these formats.",
        )
        .into());
    }

    let mut entries = collect_sources(sources)?;
    let existing = super::load(archive_path)?;
    ensure_no_collisions(&existing, &entries)?;
    entries.retain(|entry| {
        !entry.is_dir
            || !existing.iter().any(|existing| {
                existing.is_dir
                    && existing.path.trim_end_matches('/')
                        == entry.archive_path.trim_end_matches('/')
            })
    });
    let count = entries.len();
    if count == 0 {
        return Ok(0);
    }

    match kind {
        super::ArchiveKind::Zip => super::zip::add(archive_path, &entries)?,
        super::ArchiveKind::Tar(compression) => {
            super::tar::add(archive_path, &entries, compression)?
        }
        super::ArchiveKind::SevenZip => super::sevenz::add(archive_path, &entries)?,
        super::ArchiveKind::Rar | super::ArchiveKind::Ace | super::ArchiveKind::FreeArc => unreachable!(
            "unsupported archive kinds are rejected before dispatch"
        ),
    }

    Ok(count)
}

pub(super) fn replacement_file(archive_path: &Path) -> io::Result<NamedTempFile> {
    if fs::symlink_metadata(archive_path)?.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Editing an archive through a symbolic link is not supported",
        ));
    }
    let metadata = fs::metadata(archive_path)?;
    if metadata.permissions().readonly() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "The archive is read-only",
        ));
    }
    let parent = archive_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let temporary = NamedTempFile::new_in(parent)?;
    fs::set_permissions(temporary.path(), metadata.permissions())?;
    Ok(temporary)
}

pub(super) fn ensure_no_collisions(
    existing: &[ArchiveEntry],
    additions: &[SourceEntry],
) -> io::Result<()> {
    let mut paths = existing
        .iter()
        .map(|entry| (entry.path.trim_end_matches('/').to_owned(), entry.is_dir))
        .collect::<HashMap<_, _>>();

    for entry in additions {
        let archive_path = entry.archive_path.trim_end_matches('/');
        let mut parent = Path::new(archive_path).parent();
        while let Some(parent_path) = parent {
            let parent_key = parent_path.to_string_lossy();
            if paths.get(parent_key.as_ref()) == Some(&false) {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("The archive contains a file where '{parent_key}' must be a folder"),
                ));
            }
            parent = parent_path.parent();
        }
        match paths.get(archive_path) {
            Some(true) if entry.is_dir => continue,
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "The archive already contains '{}'; rename or remove the existing entry before adding it",
                        archive_path
                    ),
                ));
            }
            None => {
                if !entry.is_dir
                    && paths
                        .keys()
                        .any(|path| path.starts_with(&format!("{archive_path}/")))
                {
                    return Err(io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        format!(
                            "The archive contains entries beneath '{archive_path}', which cannot be replaced by a file"
                        ),
                    ));
                }
                paths.insert(archive_path.to_owned(), entry.is_dir);
            }
        }
    }
    Ok(())
}

fn collect_sources(sources: &[PathBuf]) -> Result<Vec<SourceEntry>, io::Error> {
    let mut entries = Vec::new();
    for source in sources {
        let metadata = fs::symlink_metadata(source)?;
        if metadata.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("Symbolic links cannot be added: {}", source.display()),
            ));
        }
        let name = source.file_name().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Cannot add a filesystem root: {}", source.display()),
            )
        })?;
        let archive_path = name.to_string_lossy().replace('\\', "/");
        if metadata.is_dir() {
            collect_directory(source, &archive_path, &mut entries)?;
        } else if metadata.is_file() {
            entries.push(SourceEntry {
                source: source.clone(),
                archive_path,
                is_dir: false,
            });
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("Unsupported filesystem item: {}", source.display()),
            ));
        }
    }
    Ok(entries)
}

fn collect_directory(
    source: &Path,
    archive_path: &str,
    entries: &mut Vec<SourceEntry>,
) -> Result<(), io::Error> {
    entries.push(SourceEntry {
        source: source.to_path_buf(),
        archive_path: format!("{archive_path}/"),
        is_dir: true,
    });
    let mut children = fs::read_dir(source)?.collect::<Result<Vec<_>, _>>()?;
    children.sort_by_key(|child| child.file_name());
    for child in children {
        let child_path = child.path();
        let metadata = fs::symlink_metadata(&child_path)?;
        let name = child.file_name().to_string_lossy().replace('\\', "/");
        let archive_child_path = format!("{archive_path}/{name}");
        if metadata.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("Symbolic links cannot be added: {}", child_path.display()),
            ));
        }
        if metadata.is_dir() {
            collect_directory(&child_path, &archive_child_path, entries)?;
        } else if metadata.is_file() {
            entries.push(SourceEntry {
                source: child_path,
                archive_path: archive_child_path,
                is_dir: false,
            });
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("Unsupported filesystem item: {}", child_path.display()),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_dropped_folder_with_relative_paths() {
        let root = super::super::temporary_directory();
        let folder = root.join("documents");
        fs::create_dir_all(folder.join("nested")).unwrap();
        fs::write(folder.join("nested/readme.txt"), b"hello").unwrap();

        let entries = collect_sources(std::slice::from_ref(&folder)).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].archive_path, "documents/");
        assert_eq!(entries[1].archive_path, "documents/nested/");
        assert_eq!(entries[2].archive_path, "documents/nested/readme.txt");
        assert!(entries[0].is_dir);
        assert!(!entries[2].is_dir);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_collisions_with_existing_files() {
        let existing = vec![ArchiveEntry {
            path: "readme.txt".to_owned(),
            is_dir: false,
            size: 10,
            encrypted: false,
        }];
        let additions = vec![SourceEntry {
            source: PathBuf::new(),
            archive_path: "readme.txt".to_owned(),
            is_dir: false,
        }];
        assert_eq!(
            ensure_no_collisions(&existing, &additions)
                .unwrap_err()
                .kind(),
            io::ErrorKind::AlreadyExists
        );
    }
}
