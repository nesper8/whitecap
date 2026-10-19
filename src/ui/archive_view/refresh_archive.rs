//! Refreshes the visible archive after a mutation.

use crate::archive::ArchiveEntry;
use crate::ui::state::SharedState;

use super::archive_name;

/// Replaces the current archive entries and redraws the visible directory.
pub(crate) fn refresh_archive(
    entries: Vec<ArchiveEntry>,
    state: &SharedState,
    list: &gtk::ListBox,
    archive_label: &gtk::Label,
    up_button: &gtk::Button,
) {
    let archive_path = {
        let mut state = state.borrow_mut();
        state.entries = entries;
        state.archive_path.clone()
    };
    let archive_name = archive_path
        .as_deref()
        .map(archive_name::location_name)
        .unwrap_or_else(|| "Archive".to_owned());
    super::render_directory::render_directory(
        state,
        list,
        archive_label,
        up_button,
        &archive_name,
    );
}
