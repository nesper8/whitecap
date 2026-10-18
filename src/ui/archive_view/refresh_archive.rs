//! Refreshes the visible archive after a mutation.

use std::path::Path;

use crate::archive::ArchiveEntry;
use crate::ui::state::SharedState;

/// Replaces the current archive entries and redraws the visible directory.
pub(crate) fn refresh_archive(
    entries: Vec<ArchiveEntry>,
    state: &SharedState,
    list: &gtk::ListBox,
    archive_label: &gtk::Label,
    up_button: &gtk::Button,
) {
    let archive_name = {
        let mut state = state.borrow_mut();
        state.entries = entries;
        state
            .archive_path
            .as_deref()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or("Archive")
            .to_owned()
    };
    super::render_directory::render_directory(
        state,
        list,
        archive_label,
        up_button,
        &archive_name,
    );
}
