//! Assembles the main Whitecap window from focused UI components.

use std::path::PathBuf;

use adw::prelude::*;

use super::{
    build_archive_controls::build_archive_controls,
    build_content_stack::build_content_stack,
    build_header::build_header,
    connect_archive_actions::connect_archive_actions,
    connect_search_and_shortcuts::connect_search_and_shortcuts,
};
use crate::ui::{drag_drop, state::SharedState};

/// Assembles and presents the main application window.
///
/// `initial_archive` is used when the desktop environment opens a file
/// directly with Whitecap; `None` creates the empty welcome state.
pub(crate) fn build_window(app: &adw::Application, initial_archive: Option<PathBuf>) {
    let state: SharedState = Default::default();
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Whitecap")
        .default_width(980)
        .default_height(680)
        .build();

    let overlay = adw::ToastOverlay::new();
    state.borrow_mut().toast_overlay = Some(overlay.clone());
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let header = build_header();
    content.append(&header.header);

    let controls = build_archive_controls();
    content.append(&controls.toolbar);
    content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    let (stack, list) = build_content_stack(&controls.search);
    content.append(&stack);
    overlay.set_child(Some(&content));
    window.set_content(Some(&overlay));

    connect_search_and_shortcuts(&window, &controls.search, &stack, &list);
    connect_archive_actions(&window, &overlay, &state, &header, &controls, &stack, &list);
    window.present();

    if let Some(path) = initial_archive {
        drag_drop::load_dropped_archive(
            &path,
            &window,
            &overlay,
            &state,
            &list,
            &stack,
            &header.title,
            &controls.archive_label,
            &header.extract_button,
            &controls.up_button,
        );
    }
}
