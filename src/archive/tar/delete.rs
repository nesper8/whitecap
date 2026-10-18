use std::{error::Error, io, path::Path};

use crate::archive::rewrite;

use super::{
    Compression,
    append_selected_entries::append_selected_entries,
    compress_tar::compress_tar,
};

pub(crate) fn delete(
    archive_path: &Path,
    selected: &[(String, bool)],
    compression: Compression,
) -> Result<usize, Box<dyn Error>> {
    let output = rewrite::replacement_file(archive_path)?;
    let mut count = 0;
    if matches!(compression, Compression::None) {
        let mut builder = ::tar::Builder::new(output.reopen()?);
        append_selected_entries(archive_path, compression, selected, &mut count, &mut builder)?;
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "None of the selected entries were found in the archive",
            )
            .into());
        }
        builder.into_inner()?.sync_all()?;
    } else {
        let uncompressed = tempfile::NamedTempFile::new()?;
        let mut builder = ::tar::Builder::new(uncompressed.reopen()?);
        append_selected_entries(archive_path, compression, selected, &mut count, &mut builder)?;
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "None of the selected entries were found in the archive",
            )
            .into());
        }
        builder.into_inner()?.sync_all()?;
        compress_tar(uncompressed.path(), output.reopen()?, compression)?;
    }
    output.persist(archive_path)?;
    Ok(count)
}
