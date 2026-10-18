//! Rejects absolute and traversal paths reported by FreeArc.

use std::{io, path::{Component, Path}};

/// Rejects FreeArc paths that could escape the extraction directory.
pub(crate) fn validate_relative_path(path: &str) -> Result<(), io::Error> {
    let normalized = path.replace('\\', "/");
    let path = Path::new(&normalized);
    let has_drive_prefix = normalized
        .as_bytes()
        .get(1)
        .is_some_and(|byte| *byte == b':');
    if has_drive_prefix
        || path.components().any(|component| {
            !matches!(component, Component::Normal(_) | Component::CurDir)
        })
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Refusing to extract a FreeArc entry with an unsafe path",
        ));
    }
    Ok(())
}
