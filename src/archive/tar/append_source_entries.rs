//! Writes collected files and directories into a TAR builder.

use std::{error::Error, io::Write};

use crate::archive::sources::SourceEntry;

/// Appends validated files and directories to a TAR builder.
pub(crate) fn append_source_entries<W: Write>(
    additions: &[SourceEntry],
    builder: &mut ::tar::Builder<W>,
) -> Result<(), Box<dyn Error>> {
    for entry in additions {
        let archive_path = entry.archive_path.trim_end_matches('/');
        if entry.is_dir {
            builder.append_dir(archive_path, &entry.source)?;
        } else {
            builder.append_path_with_name(&entry.source, archive_path)?;
        }
    }
    Ok(())
}
