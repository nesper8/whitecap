use std::{error::Error, path::Path};

use crate::archive::{rewrite, sources::SourceEntry};

use super::{
    Compression,
    append_existing_entries::append_existing_entries,
    append_source_entries::append_source_entries,
    compress_tar::compress_tar,
};

pub(crate) fn add(
    archive_path: &Path,
    additions: &[SourceEntry],
    compression: Compression,
) -> Result<(), Box<dyn Error>> {
    let output = rewrite::replacement_file(archive_path)?;

    if matches!(compression, Compression::None) {
        let mut builder = ::tar::Builder::new(output.reopen()?);
        append_existing_entries(archive_path, compression, &mut builder)?;
        append_source_entries(additions, &mut builder)?;
        builder.into_inner()?.sync_all()?;
    } else {
        let uncompressed = tempfile::NamedTempFile::new()?;
        let mut builder = ::tar::Builder::new(uncompressed.reopen()?);
        append_existing_entries(archive_path, compression, &mut builder)?;
        append_source_entries(additions, &mut builder)?;
        builder.into_inner()?.sync_all()?;

        compress_tar(uncompressed.path(), output.reopen()?, compression)?;
    }
    output.persist(archive_path)?;
    Ok(())
}
