//! Commits loaded archive data to state and displays its root directory.

use std::path::Path;

use adw::prelude::*;

use crate::archive::ArchiveEntry;
use crate::ui::state::SharedState;

use super::render_directory::render_directory;

/// Stores a loaded archive in shared state and renders its root directory.
pub(crate) fn show_archive(
    path: &Path,
    entries: Vec<ArchiveEntry>,
    password: Option<String>,
    state: &SharedState,
    list: &gtk::ListBox,
    stack: &gtk::Stack,
    title: &gtk::Label,
    archive_label: &gtk::Label,
    extract_button: &gtk::Button,
    up_button: &gtk::Button,
) {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Archive")
        .to_owned();
    {
        let mut state = state.borrow_mut();
        state.archive_path = Some(path.to_path_buf());
        state.archive_password = password;
        state.entries = entries;
        state.visible_entries.clear();
        state.current_directory.clear();
    }
    render_directory(
        state,
        list,
        archive_label,
        up_button,
        &file_name,
    );

    title.set_text(&file_name);
    extract_button.set_sensitive(true);
    stack.set_visible_child_name("contents");
    list.grab_focus();
}
