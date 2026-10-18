//! Exports selected archive entries for an outgoing GTK drag.

use std::{error::Error, io};

use adw::prelude::*;
use gtk::{gdk, gio, glib};

use crate::archive;
use crate::ui::state::SharedState;

/// Extracts selected entries into a temporary directory for an external drag.
pub(crate) fn create_drag_export(
    state: &SharedState,
    list: &gtk::ListBox,
    dragged_index: usize,
) -> Result<gdk::ContentProvider, Box<dyn Error>> {
    let (archive_path, selected, password) = {
        let state = state.borrow();
        let archive_path = state
            .archive_path
            .clone()
            .ok_or_else(|| io::Error::other("No archive is open"))?;
        let selected_rows = list.selected_rows();
        let selected_indices = if selected_rows.iter().any(|row| {
            row.widget_name()
                .strip_prefix("archive-entry-")
                .and_then(|index| index.parse::<usize>().ok())
                == Some(dragged_index)
        }) {
            selected_rows
                .iter()
                .filter_map(|row| {
                    row.widget_name()
                        .strip_prefix("archive-entry-")?
                        .parse::<usize>()
                        .ok()
                })
                .collect::<Vec<_>>()
        } else {
            vec![dragged_index]
        };
        let selected = selected_indices
            .into_iter()
            .filter_map(|index| state.visible_entries.get(index))
            .map(|entry| (entry.path.clone(), entry.is_dir))
            .collect::<Vec<_>>();
        (archive_path, selected, state.archive_password.clone())
    };

    if selected.is_empty() {
        return Err(io::Error::other("No archive entries are selected for dragging").into());
    }

    let export_directory = tempfile::Builder::new()
        .prefix("whitecap-drag-")
        .tempdir()?;
    archive::extract_with_password(
        &archive_path,
        export_directory.path(),
        &selected,
        password.as_deref(),
    )?;

    let files = selected
        .iter()
        .map(|(path, _)| gio::File::for_path(export_directory.path().join(path)))
        .collect::<Vec<_>>();
    let file_list = gdk::FileList::from_array(&files);
    let file_provider = gdk::ContentProvider::for_value(&file_list.to_value());
    let uri_list = files
        .iter()
        .map(|file| file.uri().to_string())
        .collect::<Vec<_>>()
        .join("\r\n")
        + "\r\n";
    let uri_bytes = glib::Bytes::from_owned(uri_list.into_bytes());
    let uri_provider = gdk::ContentProvider::for_bytes("text/uri-list", &uri_bytes);
    let provider = gdk::ContentProvider::new_union(&[file_provider, uri_provider]);

    state.borrow_mut().drag_exports.push(export_directory);
    Ok(provider)
}
