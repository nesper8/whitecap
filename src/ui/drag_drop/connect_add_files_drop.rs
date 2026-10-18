//! Handles files dropped into an open archive's contents view.

use std::rc::Rc;

use crate::archive;
use crate::ui::{
    archive_view,
    drop_transport::{DropHandler, connect_file_drop},
    helpers::{show_error, show_warning},
    state::SharedState,
};

/// Accepts files dropped on the contents view and adds them to the archive.
pub(crate) fn connect_add_files_drop(
    contents: &impl gtk::prelude::IsA<gtk::Widget>,
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
    let drop_overlay = overlay.clone();
    let state = state.clone();
    let list = list.clone();
    let stack = stack.clone();
    let title = title.clone();
    let archive_label = archive_label.clone();
    let extract_button = extract_button.clone();
    let up_button = up_button.clone();
    let handler: DropHandler = Rc::new(move |paths| {
        let Some(archive_path) = state.borrow().archive_path.clone() else {
            show_error(&overlay, "Open an archive before dropping files to add");
            return;
        };
        match archive::supports_modification(&archive_path) {
            Ok(true) => {}
            Ok(false) => {
                show_warning(
                    &overlay,
                    "This archive format cannot be modified in Whitecap (RAR, ACE, and FreeArc).",
                );
                return;
            }
            Err(error) => {
                show_error(&overlay, error);
                return;
            }
        }
        match archive::add_files(&archive_path, &paths) {
            Ok(count) => {
                let password = state.borrow().archive_password.clone();
                match archive::load_with_password(&archive_path, password.as_deref()) {
                    Ok(entries) => {
                        archive_view::show_archive(
                            &archive_path,
                            entries,
                            password,
                            &state,
                            &list,
                            &stack,
                            &title,
                            &archive_label,
                            &extract_button,
                            &up_button,
                        );
                        overlay.add_toast(adw::Toast::new(&format!(
                            "Added {count} archive {}",
                            if count == 1 { "entry" } else { "entries" }
                        )));
                    }
                    Err(error) => show_error(&overlay, error),
                }
            }
            Err(error) => show_error(&overlay, error),
        }
    });
    connect_file_drop(contents, drop_overlay, handler);
}
