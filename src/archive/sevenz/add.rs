use std::{error::Error, fs::File, io::Write, path::Path};

use crate::archive::{rewrite, sources::SourceEntry};

pub(crate) fn add(archive_path: &Path, additions: &[SourceEntry]) -> Result<(), Box<dyn Error>> {
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
