use adw::prelude::*;
use gtk::{gio, glib};

use crate::archive;

use crate::ui::{archive_view, helpers::{show_error, show_warning}, state::SharedState};

pub fn connect_delete_action(
    button: &gtk::Button,
    window: &adw::ApplicationWindow,
    overlay: &adw::ToastOverlay,
    state: &SharedState,
    list: &gtk::ListBox,
    archive_label: &gtk::Label,
    up_button: &gtk::Button,
) {
    list.connect_selected_rows_changed({
        let button = button.clone();
        move |list| button.set_sensitive(!list.selected_rows().is_empty())
    });

    let request_delete = {
        let window = window.clone();
        let overlay = overlay.clone();
        let state = state.clone();
        let list = list.clone();
        let archive_label = archive_label.clone();
        let up_button = up_button.clone();
        move || {
            let (archive_path, selected) = {
                let state = state.borrow();
                let Some(archive_path) = state.archive_path.clone() else {
                    return;
                };
                let selected = list
                    .selected_rows()
                    .iter()
                    .filter_map(|row| {
                        row.widget_name()
                            .strip_prefix("archive-entry-")?
                            .parse::<usize>()
                            .ok()
                            .and_then(|index| state.visible_entries.get(index))
                    })
                    .map(|entry| (entry.path.clone(), entry.is_dir))
                    .collect::<Vec<_>>();
                (archive_path, selected)
            };
            if selected.is_empty() {
                return;
            }
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

            let dialog = adw::AlertDialog::new(
                Some(if selected.len() == 1 {
                    "Delete selected entry?"
                } else {
                    "Delete selected entries?"
                }),
                Some(
                    "This permanently removes the selected entries from the archive. This cannot be undone.",
                ),
            );
            dialog.add_response("cancel", "Cancel");
            dialog.add_response("delete", "Delete");
            dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
            dialog.set_close_response("cancel");
            dialog.set_default_response(Some("cancel"));
            let overlay = overlay.clone();
            let state = state.clone();
            let list = list.clone();
            let archive_label = archive_label.clone();
            let up_button = up_button.clone();
            dialog.choose(Some(&window), None::<&gio::Cancellable>, move |response| {
                if response != "delete" {
                    return;
                }
                match archive::delete_entries(&archive_path, &selected) {
                    Ok(count) => match archive::load(&archive_path) {
                        Ok(entries) => {
                            archive_view::refresh_archive(
                                entries,
                                &state,
                                &list,
                                &archive_label,
                                &up_button,
                            );
                            overlay.add_toast(adw::Toast::new(&format!(
                                "Deleted {count} {}",
                                if count == 1 { "entry" } else { "entries" }
                            )));
                        }
                        Err(error) => show_error(&overlay, error),
                    },
                    Err(error) => show_error(&overlay, error),
                }
            });
        }
    };
    button.connect_clicked({
        let request_delete = request_delete.clone();
        move |_| request_delete()
    });

    let delete_key = gtk::EventControllerKey::new();
    delete_key.set_propagation_phase(gtk::PropagationPhase::Capture);
    delete_key.connect_key_pressed(move |_, key, _, _| {
        if key
            .name()
            .is_some_and(|name| name.eq_ignore_ascii_case("Delete"))
        {
            request_delete();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    list.add_controller(delete_key);
}
