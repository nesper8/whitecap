//! State fields owned by the active application window.

use std::path::PathBuf;

use crate::archive::ArchiveEntry;

/// Mutable state shared by the window's archive callbacks.
#[derive(Default)]
pub(in crate::ui) struct AppState {
    /// Path of the currently open archive, if any.
    pub(in crate::ui) archive_path: Option<PathBuf>,
    /// Password most recently accepted for the current archive.
    pub(in crate::ui) archive_password: Option<String>,
    /// All entries loaded from the current archive.
    pub(in crate::ui) entries: Vec<ArchiveEntry>,
    /// Entries currently visible in the selected directory.
    pub(in crate::ui) visible_entries: Vec<ArchiveEntry>,
    /// Normalized path of the directory currently displayed.
    pub(in crate::ui) current_directory: String,
    /// Temporary directories kept alive for outgoing drag operations.
    pub(in crate::ui) drag_exports: Vec<tempfile::TempDir>,
    /// Overlay used for recoverable operation messages.
    pub(in crate::ui) toast_overlay: Option<adw::ToastOverlay>,
}
