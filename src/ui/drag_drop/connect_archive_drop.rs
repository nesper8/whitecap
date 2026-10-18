//! Handles archives dropped on the location bar.

use std::rc::Rc;

use crate::ui::{
    drop_transport::{DropHandler, connect_file_drop},
    state::SharedState,
};

/// Accepts an archive dropped on the location bar and opens it.
pub(crate) fn connect_archive_drop(
    address_bar: &impl gtk::prelude::IsA<gtk::Widget>,
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
    let overlay = overlay.clone();
    let window = window.clone();
    let drop_overlay = overlay.clone();
    let state = state.clone();
    let list = list.clone();
    let stack = stack.clone();
    let title = title.clone();
    let archive_label = archive_label.clone();
    let extract_button = extract_button.clone();
    let up_button = up_button.clone();
    let handler: DropHandler = Rc::new(move |paths| {
        if let Some(path) = paths.first() {
            super::load_dropped_archive::load_dropped_archive(
                path,
                &window,
                &overlay,
                &state,
                &list,
                &stack,
                &title,
                &archive_label,
                &extract_button,
                &up_button,
            );
        }
    });
    connect_file_drop(address_bar, drop_overlay, handler);
}
