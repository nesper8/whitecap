use adw::prelude::*;
use gtk::gio;

use crate::archive;

use crate::ui::{
    archive_view,
    helpers::show_error,
    password_dialog::request_password,
    state::SharedState,
};

pub fn connect_open_action(
    button: &gtk::Button,
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
    button.connect_clicked({
        let window = window.clone();
        let overlay = overlay.clone();
        let state = state.clone();
        let list = list.clone();
        let stack = stack.clone();
        let title = title.clone();
        let archive_label = archive_label.clone();
        let extract_button = extract_button.clone();
        let up_button = up_button.clone();
        move |_| {
            let dialog = gtk::FileDialog::new();
            dialog.set_title("Open Archive");
            let filter = gtk::FileFilter::new();
            filter.set_name(Some("Supported archives"));
            for pattern in [
                "*.zip", "*.tar", "*.tar.gz", "*.tgz", "*.tar.bz2", "*.tbz2", "*.tar.xz",
                "*.txz", "*.tar.zst", "*.tzst", "*.7z", "*.rar", "*.arc", "*.ace",
            ] {
                filter.add_pattern(pattern);
            }
            dialog.set_default_filter(Some(&filter));
            dialog.open(Some(&window), None::<&gio::Cancellable>, {
                let window = window.clone();
                let overlay = overlay.clone();
                let state = state.clone();
                let list = list.clone();
                let stack = stack.clone();
                let title = title.clone();
                let archive_label = archive_label.clone();
                let extract_button = extract_button.clone();
                let up_button = up_button.clone();
                move |result| match result {
                    Ok(file) => {
                        if let Some(path) = file.path() {
                            archive_view::open_archive(
                                &path,
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
                    }
                    Err(error) if error.matches(gtk::DialogError::Dismissed) => {}
                    Err(error) => show_error(&overlay, error),
                }
            });
        }
    });
}

pub fn connect_extract_action(
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
                            extract_selected(
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

fn extract_selected(
    window: adw::ApplicationWindow,
    overlay: adw::ToastOverlay,
    state: SharedState,
    archive_path: std::path::PathBuf,
    destination: std::path::PathBuf,
    selected: Vec<(String, bool)>,
    encrypted_selection: bool,
    password: Option<String>,
) {
    if encrypted_selection && password.is_none() {
        request_extract_password(
            window,
            overlay,
            state,
            archive_path,
            destination,
            selected,
            encrypted_selection,
            "Enter the password to extract the selected encrypted entries.",
        );
        return;
    }
    match archive::extract_with_password(
        &archive_path,
        &destination,
        &selected,
        password.as_deref(),
    ) {
        Ok(count) => {
            state.borrow_mut().archive_password = password;
            overlay.add_toast(adw::Toast::new(&format!(
                "Extracted {count} {}",
                if count == 1 { "item" } else { "items" }
            )));
        }
        Err(error) => match error.downcast_ref::<archive::PasswordError>() {
            Some(password_error) => {
                let message = match password_error {
                    archive::PasswordError::Required => {
                        "Enter the password to extract the selected encrypted entries."
                    }
                    archive::PasswordError::Invalid => "The password was incorrect. Try again.",
                };
                request_extract_password(
                    window,
                    overlay,
                    state,
                    archive_path,
                    destination,
                    selected,
                    encrypted_selection,
                    message,
                );
            }
            None => show_error(&overlay, error),
        },
    }
}

fn request_extract_password(
    window: adw::ApplicationWindow,
    overlay: adw::ToastOverlay,
    state: SharedState,
    archive_path: std::path::PathBuf,
    destination: std::path::PathBuf,
    selected: Vec<(String, bool)>,
    encrypted_selection: bool,
    message: &str,
) {
    let retry_window = window.clone();
    request_password(&window, message, move |password| {
        if let Some(password) = password {
            extract_selected(
                retry_window,
                overlay,
                state,
                archive_path,
                destination,
                selected,
                encrypted_selection,
                Some(password),
            );
        }
    });
}
