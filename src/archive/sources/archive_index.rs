use std::{collections::HashMap, io, path::Path};

use crate::archive::ArchiveEntry;

use super::source_entry::SourceEntry;

pub(crate) struct ArchiveIndex {
    entries: HashMap<String, bool>,
}

impl ArchiveIndex {
    pub(crate) fn from_entries(entries: &[ArchiveEntry]) -> Self {
        Self {
            entries: entries
                .iter()
                .map(|entry| (entry.path.trim_end_matches('/').to_owned(), entry.is_dir))
                .collect(),
        }
    }

    pub(crate) fn add(&mut self, entry: &SourceEntry) -> io::Result<()> {
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
