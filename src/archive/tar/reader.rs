use std::{error::Error, fs::File, io::Read, path::Path};

use super::Compression;

pub(crate) fn reader(path: &Path, compression: Compression) -> Result<Box<dyn Read>, Box<dyn Error>> {
    let file = File::open(path)?;
    Ok(match compression {
        Compression::None => Box::new(file),
        Compression::Gzip => Box::new(flate2::read::GzDecoder::new(file)),
        Compression::Bzip2 => Box::new(bzip2::read::BzDecoder::new(file)),
        Compression::Xz => Box::new(xz2::read::XzDecoder::new(file)),
        Compression::Zstd => Box::new(zstd::stream::read::Decoder::new(file)?),
    })
}
