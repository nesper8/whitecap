//! Formats archive names for the window title and location bar.

use std::path::Path;

use crate::archive;

const READ_ONLY_ICON: &str = "🔒";

/// Returns the archive filename used by the window title.
pub(crate) fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Archive")
        .to_owned()
}

/// Returns the archive filename used by the location bar.
///
/// Read-only archive formats are prefixed with a padlock so the location bar
/// communicates that their contents cannot be modified in Whitecap.
pub(crate) fn location_name(path: &Path) -> String {
    let file_name = file_name(path);
    let is_read_only = archive::supports_modification(path)
        .map(|supports_modification| !supports_modification)
        .unwrap_or(false);

    if is_read_only {
        format!("{READ_ONLY_ICON} {file_name}")
    } else {
        file_name
    }
}
