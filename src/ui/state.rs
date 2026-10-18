use std::{cell::RefCell, path::PathBuf, rc::Rc};

use crate::archive::ArchiveEntry;

#[derive(Default)]
pub(super) struct AppState {
    pub(super) archive_path: Option<PathBuf>,
    pub(super) entries: Vec<ArchiveEntry>,
    pub(super) visible_entries: Vec<ArchiveEntry>,
    pub(super) current_directory: String,
    pub(super) drag_exports: Vec<tempfile::TempDir>,
    pub(super) toast_overlay: Option<adw::ToastOverlay>,
}

pub(super) type SharedState = Rc<RefCell<AppState>>;
