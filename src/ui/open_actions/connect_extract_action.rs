//! Connects the extraction button to the folder chooser.

use adw::prelude::*;
use gtk::gio;

use crate::ui::{helpers::show_error, state::SharedState};

/// Connects the extract button to the destination-folder chooser.
pub(crate) fn connect_extract_action(
    button: &gtk::Button,
    window: &adw::ApplicationWindow,
    overlay: &adw::ToastOverlay,
    state: &SharedState,
    list: &gtk::ListBox,
) {
    button.connect_clicked({
        let window = window.clone();
        let overlay = overlay.clone();
        let state = state.clone();
        let list = list.clone();
        move |_| {
            let dialog = gtk::FileDialog::new();
            dialog.set_title("Choose Extraction Folder");
            dialog.select_folder(Some(&window), None::<&gio::Cancellable>, {
                let window = window.clone();
                let overlay = overlay.clone();
                let state = state.clone();
                let list = list.clone();
                move |result| match result {
                    Ok(folder) => {
                        if let Some(destination) = folder.path() {
                            let (archive_path, selected, encrypted_selection, password) = {
                                let state = state.borrow();
                                let Some(archive_path) = state.archive_path.clone() else {
                                    return;
                                };
                                let selected_entries = list
                                    .selected_rows()
                                    .iter()
                                    .filter_map(|row| {
                                        row.widget_name()
                                            .strip_prefix("archive-entry-")?
                                            .parse::<usize>()
                                            .ok()
                                            .and_then(|index| state.visible_entries.get(index))
                                    })
                                    .collect::<Vec<_>>();
                                let encrypted_selection = if selected_entries.is_empty() {
                                    state.entries.iter().any(|entry| entry.encrypted)
                                } else {
                                    selected_entries.iter().any(|entry| entry.encrypted)
                                };
                                let selected = selected_entries
                                    .iter()
                                    .map(|entry| (entry.path.clone(), entry.is_dir))
                                    .collect::<Vec<_>>();
                                (
                                    archive_path,
                                    selected,
                                    encrypted_selection,
                                    state.archive_password.clone(),
                                )
                            };
                            super::extract_selected::extract_selected(
                                window.clone(),
                                overlay.clone(),
                                state.clone(),
                                archive_path,
                                destination,
                                selected,
                                encrypted_selection,
                                password,
                            );
                        }
                    }
                    Err(error) if error.matches(gtk::DialogError::Dismissed) => {}
                    Err(error) => show_error(&overlay, error),
                }
            });
        }
    });
}
