//! Rebuilds a ZIP archive without selected entries.

use std::{error::Error, io, io::Write, path::Path};

use crate::archive::{delete, path, rewrite};

/// Rebuilds a ZIP archive without selected entries.
pub(crate) fn delete(
    archive_path: &Path,
    selected: &[(String, bool)],
) -> Result<usize, Box<dyn Error>> {
    let temporary = rewrite::replacement_file(archive_path)?;
    let mut archive = ::zip::ZipArchive::new(std::fs::File::open(archive_path)?)?;
    let mut writer = ::zip::ZipWriter::new(temporary.reopen()?);
    let mut count = 0;

    for index in 0..archive.len() {
        let entry = archive.by_index_raw(index)?;
        let entry_path = path::normalize(entry.name());
        if delete::is_selected(&entry_path, selected) {
            count += 1;
        } else {
            writer.raw_copy_file(entry)?;
        }
    }
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
