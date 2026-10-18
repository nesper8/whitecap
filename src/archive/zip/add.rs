//! Appends validated source entries to a ZIP archive.

use std::{error::Error, fs::File, io, io::Write, path::Path};

use crate::archive::{rewrite, sources::SourceEntry};

/// Appends collected files and directories to a ZIP archive.
pub(crate) fn add(archive_path: &Path, additions: &[SourceEntry]) -> Result<(), Box<dyn Error>> {
    let mut temporary = rewrite::replacement_file(archive_path)?;
    io::copy(&mut File::open(archive_path)?, temporary.as_file_mut())?;

    let mut writer = ::zip::ZipWriter::new_append(temporary.reopen()?)?;
    let options = ::zip::write::SimpleFileOptions::default();
    for entry in additions {
        if entry.is_dir {
            writer.add_directory(entry.archive_path.trim_end_matches('/'), options)?;
        } else {
            writer.start_file(&entry.archive_path, options)?;
            io::copy(&mut File::open(&entry.source)?, &mut writer)?;
        }
    }
    writer.finish()?.flush()?;
    temporary.persist(archive_path)?;
    Ok(())
}
