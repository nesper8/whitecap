//! Compresses an intermediate TAR stream with the selected codec.

use std::{error::Error, fs::File, io, path::Path};

use super::Compression;

/// Compresses an uncompressed TAR file with the selected codec.
pub(crate) fn compress_tar(
    source: &Path,
    output: File,
    compression: Compression,
) -> Result<(), Box<dyn Error>> {
    let mut input = File::open(source)?;
    match compression {
        Compression::Gzip => {
            let mut encoder =
                flate2::write::GzEncoder::new(output, flate2::Compression::default());
            io::copy(&mut input, &mut encoder)?;
            encoder.finish()?.sync_all()?;
        }
        Compression::Bzip2 => {
            let mut encoder =
                bzip2::write::BzEncoder::new(output, bzip2::Compression::default());
            io::copy(&mut input, &mut encoder)?;
            encoder.finish()?.sync_all()?;
        }
        Compression::Xz => {
            let mut encoder = xz2::write::XzEncoder::new(output, 6);
            io::copy(&mut input, &mut encoder)?;
            encoder.finish()?.sync_all()?;
        }
        Compression::Zstd => {
            let mut encoder = zstd::stream::write::Encoder::new(output, 0)?;
            io::copy(&mut input, &mut encoder)?;
            encoder.finish()?.sync_all()?;
        }
        Compression::None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot compress an uncompressed TAR archive",
            )
            .into());
        }
    }
    Ok(())
}
