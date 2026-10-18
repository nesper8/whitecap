//! Renders the current directory and updates navigation state.

use adw::prelude::*;

use crate::ui::{
    archive_contents::entries_in_directory,
    archive_row::archive_entry_row,
    state::SharedState,
};

/// Rebuilds list rows and updates the location and parent-folder controls.
pub(crate) fn render_directory(
    state: &SharedState,
    list: &gtk::ListBox,
    location_label: &gtk::Label,
    up_button: &gtk::Button,
    archive_name: &str,
) {
    let (current_directory, visible_entries) = {
        let state = state.borrow();
        let current_directory = state.current_directory.clone();
        let visible_entries = entries_in_directory(&state.entries, &current_directory);
        (current_directory, visible_entries)
    };

    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    for (index, entry) in visible_entries.iter().enumerate() {
        list.append(&archive_entry_row(index, entry, state, list));
    }

    let location = if current_directory.is_empty() {
        format!("{archive_name} /")
    } else {
        format!("{archive_name} / {current_directory}")
    };
    location_label.set_text(&location);
    up_button.set_sensitive(!current_directory.is_empty());
    state.borrow_mut().visible_entries = visible_entries;
}
