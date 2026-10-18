//! Connects directory-row activation to navigation.

use adw::prelude::*;

use crate::ui::state::SharedState;

/// Opens directory rows when the user activates them.
pub(crate) fn connect_row_activation(
    list: &gtk::ListBox,
    state: &SharedState,
    archive_list: &gtk::ListBox,
    location_label: &gtk::Label,
    up_button: &gtk::Button,
) {
    let state = state.clone();
    let archive_list = archive_list.clone();
    let location_label = location_label.clone();
    let up_button = up_button.clone();
    list.connect_row_activated(move |_, row| {
        let Some(index) = row
            .widget_name()
            .strip_prefix("archive-entry-")
            .and_then(|index| index.parse::<usize>().ok())
        else {
            return;
        };
        let mut state_ref = state.borrow_mut();
        let Some(entry) = state_ref.visible_entries.get(index) else {
            return;
        };
        if !entry.is_dir {
            return;
        }
        let directory = entry.path.clone();
        state_ref.current_directory = directory;
        let archive_name = state_ref
            .archive_path
            .as_deref()
            .and_then(std::path::Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or("Archive")
            .to_owned();
        drop(state_ref);
        super::render_directory::render_directory(
            &state,
            &archive_list,
            &location_label,
            &up_button,
            &archive_name,
        );
    });
}
