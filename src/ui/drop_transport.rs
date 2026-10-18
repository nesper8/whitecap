//! Converts GTK drop events into lists of local filesystem paths.

use std::{path::PathBuf, rc::Rc};

use adw::prelude::*;
use gtk::{gdk, gio, glib};

use super::helpers::show_error;

/// Callback invoked with local paths recovered from a GTK drop.
pub(super) type DropHandler = Rc<dyn Fn(Vec<PathBuf>)>;

/// Installs asynchronous file and URI-list drop handling on a widget.
pub(super) fn connect_file_drop(
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
                        let paths = files
                            .files()
                            .iter()
                            .map(gio::File::path)
                            .collect::<Option<Vec<_>>>();
                        if let Some(paths) = paths.filter(|paths| !paths.is_empty()) {
                            handler(paths);
                        } else if files.files().is_empty() {
                            show_error(&overlay, "The dropped items are not local files");
                        } else {
                            show_error(
                                &overlay,
                                "Every dropped item must be a local file or folder",
                            );
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

/// Reads a text URI list and forwards its local files to the drop handler.
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
                                show_error(
                                    &overlay,
                                    "No local files were found in the dropped data",
                                );
                            } else {
                                show_error(
                                    &overlay,
                                    "Every dropped item must be a local file or folder",
                                );
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
