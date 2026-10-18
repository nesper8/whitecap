//! Shared mutable state for the active archive and its GTK presentation.

use std::{cell::RefCell, path::PathBuf, rc::Rc};

use crate::archive::ArchiveEntry;

#[derive(Default)]
/// Mutable state shared by the window's archive callbacks.
pub(super) struct AppState {
    /// Path of the currently open archive, if any.
    pub(super) archive_path: Option<PathBuf>,
    /// Password most recently accepted for the current archive.
    pub(super) archive_password: Option<String>,
    /// All entries loaded from the current archive.
    pub(super) entries: Vec<ArchiveEntry>,
    /// Entries currently visible in the selected directory.
    pub(super) visible_entries: Vec<ArchiveEntry>,
    /// Normalized path of the directory currently displayed.
    pub(super) current_directory: String,
    /// Temporary directories kept alive for outgoing drag operations.
    pub(super) drag_exports: Vec<tempfile::TempDir>,
    /// Overlay used for recoverable operation messages.
    pub(super) toast_overlay: Option<adw::ToastOverlay>,
}

/// Shared ownership wrapper used by GTK callbacks.
pub(super) type SharedState = Rc<RefCell<AppState>>;
