//! Copies existing TAR entries into a replacement archive.

use std::{error::Error, io::Write, path::Path};

use super::{Compression, reader::reader};

/// Appends every entry from the original TAR stream to a builder.
pub(crate) fn append_existing_entries<W: Write>(
    archive_path: &Path,
    compression: Compression,
    builder: &mut ::tar::Builder<W>,
) -> Result<(), Box<dyn Error>> {
    let mut archive = ::tar::Archive::new(reader(archive_path, compression)?);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let header = entry.header().clone();
        builder.append(&header, &mut entry)?;
    }
    Ok(())
}
