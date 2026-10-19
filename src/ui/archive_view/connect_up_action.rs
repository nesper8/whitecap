//! Connects the parent-directory navigation button.

use adw::prelude::*;

use crate::ui::state::SharedState;

use super::archive_name;

/// Connects the parent-folder button to directory navigation.
pub(crate) fn connect_up_action(
    button: &gtk::Button,
    state: &SharedState,
    list: &gtk::ListBox,
    location_label: &gtk::Label,
) {
    let state = state.clone();
    let list = list.clone();
    let location_label = location_label.clone();
    let button_ref = button.clone();
    button.connect_clicked(move |_| {
        let archive_name = state
            .borrow()
            .archive_path
            .as_deref()
            .map(archive_name::location_name)
            .unwrap_or_else(|| "Archive".to_owned());
        {
            let mut state = state.borrow_mut();
            if let Some((parent, _)) = state.current_directory.rsplit_once('/') {
                state.current_directory = parent.to_owned();
            } else {
                state.current_directory.clear();
            }
        }
        super::render_directory::render_directory(
            &state,
            &list,
            &location_label,
            &button_ref,
            &archive_name,
        );
    });
}
