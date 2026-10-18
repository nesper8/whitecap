//! Routes dropped archives into the archive-view loading flow.

use std::path::Path;

use crate::ui::{archive_view, state::SharedState};

/// Routes a dropped archive to the archive-view loading flow.
pub(crate) fn load_dropped_archive(
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
    archive_view::open_archive(
        path,
        window,
        overlay,
        state,
        list,
        stack,
        title,
        archive_label,
        extract_button,
        up_button,
    );
}
