//! Rebuilds a 7z archive without selected entries.

use std::{error::Error, io, io::Write, path::Path};

use crate::archive::{delete, path, rewrite};

/// Rewrites a 7z archive while omitting selected entries.
pub(crate) fn delete(
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
