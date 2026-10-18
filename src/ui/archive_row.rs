use std::path::Path;

use adw::prelude::*;

use crate::archive::ArchiveEntry;

use super::{
    drag_drop,
    helpers::{format_size, show_error},
    state::SharedState,
};

pub(super) fn archive_entry_row(
    index: usize,
    entry: &ArchiveEntry,
    state: &SharedState,
    list: &gtk::ListBox,
) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.set_widget_name(&format!("archive-entry-{index}"));
    let line = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    line.set_margin_start(12);
    line.set_margin_end(12);
    line.set_margin_top(8);
    line.set_margin_bottom(8);

    let icon = gtk::Image::from_icon_name(if entry.is_dir {
        "folder-symbolic"
    } else {
        "text-x-generic-symbolic"
    });
    let name = gtk::Label::new(Some(
        Path::new(&entry.path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&entry.path),
    ));
    name.set_xalign(0.0);
    name.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    name.set_hexpand(true);
    let size_text = if entry.is_dir {
        String::new()
    } else {
        format_size(entry.size)
    };
    let size = gtk::Label::new(Some(&size_text));
    size.add_css_class("dim-label");
    line.append(&icon);
    line.append(&name);
    line.append(&size);
    row.set_child(Some(&line));

    let drag_source = gtk::DragSource::new();
    drag_source.set_actions(gtk::gdk::DragAction::COPY);
    drag_source.connect_prepare({
        let state = state.clone();
        let list = list.clone();
        move |_, _, _| {
            let export = match drag_drop::create_drag_export(&state, &list, index) {
                Ok(export) => export,
                Err(error) => {
                    if let Some(overlay) = &state.borrow().toast_overlay {
                        if error.downcast_ref::<crate::archive::PasswordError>().is_some() {
                            super::helpers::show_warning(
                                overlay,
                                "Use Extract to enter the archive password before dragging these entries.",
                            );
                        } else {
                            show_error(overlay, error);
                        }
                    }
                    return None;
                }
            };
            Some(export)
        }
    });
    row.add_controller(drag_source);
    row
}
