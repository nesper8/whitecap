use std::{
    error::Error,
    io,
    path::{Path, PathBuf},
    rc::Rc,
};

use adw::prelude::*;
use gtk::{gdk, gio, glib};

use crate::archive;

use super::{
    archive_view,
    helpers::{show_error, show_warning},
    state::SharedState,
};

type DropHandler = Rc<dyn Fn(Vec<PathBuf>)>;

pub(super) fn connect_archive_drop(
    address_bar: &impl gtk::prelude::IsA<gtk::Widget>,
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
        if let Some(path) = paths.first() {
            load_dropped_archive(
                path,
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

pub(super) fn connect_add_files_drop(
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
            Ok(count) => match archive::load(&archive_path) {
                Ok(entries) => {
                    archive_view::show_archive(
                        &archive_path,
                        entries,
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
            },
            Err(error) => show_error(&overlay, error),
        }
    });
    connect_file_drop(contents, drop_overlay, handler);
}

fn connect_file_drop(
    widget: &impl gtk::prelude::IsA<gtk::Widget>,
    overlay: adw::ToastOverlay,
    handler: DropHandler,
) {
    let file_list_formats = gdk::ContentFormats::for_type(gdk::FileList::static_type());
    let uri_formats = gdk::ContentFormats::new(&["text/uri-list"]);
    let drop_target = gtk::DropTargetAsync::new(
        Some(file_list_formats.union(&uri_formats)),
        gdk::DragAction::COPY,
    );
    drop_target.set_propagation_phase(gtk::PropagationPhase::Capture);
    drop_target.connect_drop(move |_, drop, _, _| {
        let drop_for_callback = drop.clone();
        let overlay = overlay.clone();
        let handler = handler.clone();
        drop.read_value_async(
            gdk::FileList::static_type(),
            glib::Priority::DEFAULT,
            None::<&gio::Cancellable>,
            move |result| match result {
                Ok(value) => match value.get::<gdk::FileList>() {
                    Ok(files) => {
                        let paths = files.files().iter().map(gio::File::path).collect::<Option<Vec<_>>>();
                        if let Some(paths) = paths.filter(|paths| !paths.is_empty()) {
                            handler(paths);
                        } else if files.files().is_empty() {
                            show_error(&overlay, "The dropped items are not local files");
                        } else {
                            show_error(&overlay, "Every dropped item must be a local file or folder");
                        }
                        drop_for_callback.finish(gdk::DragAction::COPY);
                    }
                    Err(error) => {
                        show_error(&overlay, error);
                        drop_for_callback.finish(gdk::DragAction::empty());
                    }
                },
                Err(_) => read_dropped_uri_list(&drop_for_callback, &overlay, &handler),
            },
        );
        true
    });
    widget.add_controller(drop_target);
}

fn read_dropped_uri_list(drop: &gdk::Drop, overlay: &adw::ToastOverlay, handler: &DropHandler) {
    let drop_to_finish = drop.clone();
    let overlay = overlay.clone();
    let handler = handler.clone();
    drop.read_async(
        &["text/uri-list"],
        glib::Priority::DEFAULT,
        None::<&gio::Cancellable>,
        move |result| match result {
            Ok((stream, _)) => {
                stream.read_bytes_async(
                    1024 * 1024,
                    glib::Priority::DEFAULT,
                    None::<&gio::Cancellable>,
                    move |result| match result {
                        Ok(bytes) => {
                            let text = String::from_utf8_lossy(bytes.as_ref());
                            let uris = text
                                .lines()
                                .map(str::trim)
                                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                                .collect::<Vec<_>>();
                            let paths = uris
                                .iter()
                                .map(|uri| gio::File::for_uri(uri).path())
                                .collect::<Option<Vec<_>>>();
                            if let Some(paths) = paths.filter(|paths| !paths.is_empty()) {
                                handler(paths);
                            } else if uris.is_empty() {
                                show_error(&overlay, "No local files were found in the dropped data");
                            } else {
                                show_error(&overlay, "Every dropped item must be a local file or folder");
                            }
                            drop_to_finish.finish(gdk::DragAction::COPY);
                        }
                        Err(error) => {
                            show_error(&overlay, error);
                            drop_to_finish.finish(gdk::DragAction::empty());
                        }
                    },
                );
            }
            Err(error) => {
                show_error(&overlay, error);
                drop_to_finish.finish(gdk::DragAction::empty());
            }
        },
    );
}

pub(super) fn load_dropped_archive(
    path: &Path,
    overlay: &adw::ToastOverlay,
    state: &SharedState,
    list: &gtk::ListBox,
    stack: &gtk::Stack,
    title: &gtk::Label,
    archive_label: &gtk::Label,
    extract_button: &gtk::Button,
    up_button: &gtk::Button,
) {
    match archive::load(path) {
        Ok(entries) => archive_view::show_archive(
            path,
            entries,
            state,
            list,
            stack,
            title,
            archive_label,
            extract_button,
            up_button,
        ),
        Err(error) => show_error(overlay, error),
    }
}

pub(super) fn create_drag_export(
    state: &SharedState,
    list: &gtk::ListBox,
    dragged_index: usize,
) -> Result<gdk::ContentProvider, Box<dyn Error>> {
    let (archive_path, selected) = {
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
        (archive_path, selected)
    };

    if selected.is_empty() {
        return Err(io::Error::other("No archive entries are selected for dragging").into());
    }

    let export_directory = tempfile::Builder::new()
        .prefix("whitecap-drag-")
        .tempdir()?;
    archive::extract(&archive_path, export_directory.path(), &selected)?;

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
