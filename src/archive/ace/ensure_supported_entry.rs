use std::io;

pub(crate) fn ensure_supported_entry(
    header: &unarc_rs::ace::FileHeader,
) -> io::Result<()> {
    if header.is_continued_from_prev() || header.is_continued_to_next() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Multi-volume ACE entries are not supported",
        ));
    }
    Ok(())
}
