//! Copies only non-selected TAR entries during deletion.

use std::{error::Error, io, io::Write, path::Path};

use crate::archive::{delete, path};

use super::{Compression, reader::reader};

/// Copies entries not matched by the deletion selection into a builder.
pub(crate) fn append_selected_entries<W: Write>(
    archive_path: &Path,
    compression: Compression,
    selected: &[(String, bool)],
    count: &mut usize,
    builder: &mut ::tar::Builder<W>,
) -> Result<(), Box<dyn Error>> {
    let mut archive = ::tar::Archive::new(reader(archive_path, compression)?);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_path = path::normalize(&entry.path()?.to_string_lossy());
        if delete::is_selected(&entry_path, selected) {
            *count += 1;
            io::copy(&mut entry, &mut io::sink())?;
        } else {
            let header = entry.header().clone();
            builder.append(&header, &mut entry)?;
        }
    }
    Ok(())
}
