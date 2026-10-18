//! Starts the asynchronous archive-loading flow.

use std::path::Path;

use crate::ui::state::SharedState;

/// Starts loading an archive and handles a possible password retry.
pub(crate) fn open_archive(
    path: &Path,
    window: &adw::ApplicationWindow,
    overlay: &adw::ToastOverlay,
    state: &SharedState,
    list: &gtk::ListBox,
    stack: &gtk::Stack,
    title: &gtk::Label,
    archive_label: &gtk::Label,
    extract_button: &gtk::Button,
    up_button: &gtk::Button,
) {
    super::load_archive::load_archive(
        path.to_path_buf(),
        None,
        window.clone(),
        overlay.clone(),
        state.clone(),
        list.clone(),
        stack.clone(),
        title.clone(),
        archive_label.clone(),
        extract_button.clone(),
        up_button.clone(),
    );
}
