//! Normalizes ACE entry names for display and selection.

use crate::archive::path;

/// Converts an ACE filename into the normalized archive path used by the UI.
pub(crate) fn normalized_entry_path(name: &str) -> String {
    path::normalize(&name.replace('\\', "/"))
}
