//! Wires archive operations and drop targets to the constructed controls.

use super::{
    build_archive_controls::ArchiveControls,
    build_header::HeaderWidgets,
};
use crate::ui::{actions, archive_view, drag_drop, state::SharedState};

/// Wires archive operations and drop targets to the constructed controls.
pub(super) fn connect_archive_actions(
    window: &adw::ApplicationWindow,
    overlay: &adw::ToastOverlay,
    state: &SharedState,
    header: &HeaderWidgets,
    controls: &ArchiveControls,
    stack: &gtk::Stack,
    list: &gtk::ListBox,
) {
    actions::connect_open_action(
        &header.open_button,
        window,
        overlay,
        state,
        list,
        stack,
        &header.title,
        &controls.archive_label,
        &header.extract_button,
        &controls.up_button,
    );
    actions::connect_extract_action(
        &header.extract_button,
        window,
        overlay,
        state,
        list,
    );
    actions::connect_delete_action(
        &controls.delete_button,
        window,
        overlay,
        state,
        list,
        &controls.archive_label,
        &controls.up_button,
    );
    archive_view::connect_up_action(
        &controls.up_button,
        state,
        list,
        &controls.archive_label,
    );
    archive_view::connect_row_activation(
        list,
        state,
        list,
        &controls.archive_label,
        &controls.up_button,
    );
    drag_drop::connect_archive_drop(
        &controls.location_bar,
        window,
        overlay,
        state,
        list,
        stack,
        &header.title,
        &controls.archive_label,
        &header.extract_button,
        &controls.up_button,
    );
    drag_drop::connect_add_files_drop(
        list,
        overlay,
        state,
        list,
        stack,
        &header.title,
        &controls.archive_label,
        &header.extract_button,
        &controls.up_button,
    );
}
