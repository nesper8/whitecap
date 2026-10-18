//! Converts paths to the absolute form expected by `unarc`.

use std::{io, path::{Path, PathBuf}};

/// Resolves a relative path against the current working directory.
pub(crate) fn absolute_path(path: &Path) -> Result<PathBuf, io::Error> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}
