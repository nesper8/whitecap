use std::{error::Error, path::Path};

use crate::archive::{ArchiveEntry, path};

use super::map_password_error::map_password_error;

pub(crate) fn load_with_password(
    path: &Path,
    password: Option<&str>,
) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    let archive = sevenz_rust2::ArchiveReader::open(
        path,
        sevenz_rust2::Password::new(password.unwrap_or_default()),
    )
    .map_err(|error| map_password_error(error, password.is_some()))?;
    let encrypted = archive.archive().blocks.iter().any(|block| {
        block.coders.iter().any(|coder| {
            coder.encoder_method_id() == sevenz_rust2::EncoderMethod::ID_AES256_SHA256
        })
    });
    Ok(archive
        .archive()
        .files
        .iter()
        .map(|entry| ArchiveEntry {
            path: path::normalize(entry.name()),
            is_dir: entry.is_directory(),
            size: entry.size(),
            encrypted: !entry.is_directory() && encrypted,
        })
        .collect())
}
