//! Validates ACE entry paths before filesystem extraction.

use std::{io, path::{Component, Path, PathBuf}};

/// Converts an ACE name to a safe relative filesystem path.
///
/// Absolute paths, drive-prefixed names, and parent traversal are rejected.
pub(crate) fn safe_relative_path(name: &str) -> io::Result<PathBuf> {
    let normalized = name.replace('\\', "/");
    let mut relative = PathBuf::new();
    for component in Path::new(&normalized).components() {
        match component {
            Component::Normal(part) => {
                if relative.as_os_str().is_empty()
                    && part
                        .to_str()
                        .is_some_and(|part| part.len() >= 2 && part.as_bytes()[1] == b':')
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Unsafe ACE entry path: {name}"),
                    ));
                }
                relative.push(part);
            }
            Component::CurDir => {}
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unsafe ACE entry path: {name}"),
                ));
            }
        }
    }
    if relative.as_os_str().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unsafe ACE entry path: {name}"),
        ));
    }
    Ok(relative)
}
