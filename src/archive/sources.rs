use std::{
    collections::HashMap,
    fs,
    io,
    path::{Path, PathBuf},
};

use super::ArchiveEntry;

pub(super) struct SourceEntry {
    pub(super) source: PathBuf,
    pub(super) archive_path: String,
    pub(super) is_dir: bool,
}

pub(super) fn collect_sources(sources: &[PathBuf]) -> Result<Vec<SourceEntry>, io::Error> {
    let mut entries = Vec::new();
    for source in sources {
        let name = source.file_name().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Cannot add a filesystem root: {}", source.display()),
            )
        })?;
        let archive_path = name.to_string_lossy().replace('\\', "/");
        collect_entry(source, archive_path, &mut entries)?;
    }
    Ok(entries)
}

fn collect_entry(
    source: &Path,
    archive_path: String,
    entries: &mut Vec<SourceEntry>,
) -> Result<(), io::Error> {
    let metadata = fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("Symbolic links cannot be added: {}", source.display()),
        ));
    }

    if metadata.is_file() {
        entries.push(SourceEntry {
            source: source.to_path_buf(),
            archive_path,
            is_dir: false,
        });
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("Unsupported filesystem item: {}", source.display()),
        ));
    }

    entries.push(SourceEntry {
        source: source.to_path_buf(),
        archive_path: format!("{archive_path}/"),
        is_dir: true,
    });
    let mut children = fs::read_dir(source)?.collect::<Result<Vec<_>, _>>()?;
    children.sort_by_key(|child| child.file_name());
    for child in children {
        let child_name = child.file_name().to_string_lossy().replace('\\', "/");
        collect_entry(
            &child.path(),
            format!("{archive_path}/{child_name}"),
            entries,
        )?;
    }
    Ok(())
}

pub(super) fn ensure_no_collisions(
    existing: &[ArchiveEntry],
    additions: &[SourceEntry],
) -> io::Result<()> {
    let mut index = ArchiveIndex::from_entries(existing);
    additions.iter().try_for_each(|entry| index.add(entry))
}

struct ArchiveIndex {
    entries: HashMap<String, bool>,
}

impl ArchiveIndex {
    fn from_entries(entries: &[ArchiveEntry]) -> Self {
        Self {
            entries: entries
                .iter()
                .map(|entry| (entry.path.trim_end_matches('/').to_owned(), entry.is_dir))
                .collect(),
        }
    }

    fn add(&mut self, entry: &SourceEntry) -> io::Result<()> {
        let archive_path = entry.archive_path.trim_end_matches('/');
        self.ensure_parent_directories_exist(archive_path)?;
        match self.entries.get(archive_path) {
            Some(true) if entry.is_dir => Ok(()),
            Some(_) => Err(Self::existing_entry_error(archive_path)),
            None => {
                self.ensure_file_does_not_replace_directory(archive_path, entry.is_dir)?;
                self.entries.insert(archive_path.to_owned(), entry.is_dir);
                Ok(())
            }
        }
    }

    fn ensure_parent_directories_exist(&self, archive_path: &str) -> io::Result<()> {
        let mut parent = Path::new(archive_path).parent();
        while let Some(parent_path) = parent {
            let parent_key = parent_path.to_string_lossy();
            if self.entries.get(parent_key.as_ref()) == Some(&false) {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("The archive contains a file where '{parent_key}' must be a folder"),
                ));
            }
            parent = parent_path.parent();
        }
        Ok(())
    }

    fn ensure_file_does_not_replace_directory(
        &self,
        archive_path: &str,
        is_dir: bool,
    ) -> io::Result<()> {
        if !is_dir
            && self.entries.keys().any(|path| {
                path.strip_prefix(archive_path)
                    .is_some_and(|suffix| suffix.starts_with('/'))
            })
        {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "The archive contains entries beneath '{archive_path}', which cannot be replaced by a file"
                ),
            ));
        }
        Ok(())
    }

    fn existing_entry_error(archive_path: &str) -> io::Error {
        io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "The archive already contains '{archive_path}'; rename or remove the existing entry before adding it"
            ),
        )
    }
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
