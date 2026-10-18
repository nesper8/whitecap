use std::io;

use unarc_rs::ace::AceArchive;

pub(crate) fn ensure_supported_archive<R: io::Read + io::Seek>(
    archive: &AceArchive<R>,
) -> io::Result<()> {
    let main_header = archive.main_header();
    if main_header.is_multivolume() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Multi-volume ACE archives are not supported",
        ));
    }
    Ok(())
}
