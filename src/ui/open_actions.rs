use adw::prelude::*;
use gtk::gio;

use crate::archive;

use crate::ui::{archive_view, helpers::show_error, state::SharedState};

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
                            match archive::load(&path) {
                                Ok(entries) => archive_view::show_archive(
                                    &path,
                                    entries,
                                    &state,
                                    &list,
                                    &stack,
                                    &title,
                                    &archive_label,
                                    &extract_button,
                                    &up_button,
                                ),
                                Err(error) => show_error(&overlay, error),
                            }
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
                let overlay = overlay.clone();
                let state = state.clone();
                let list = list.clone();
                move |result| match result {
                    Ok(folder) => {
                        if let Some(destination) = folder.path() {
                            let state = state.borrow();
                            if let Some(archive_path) = &state.archive_path {
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
                                match archive::extract(archive_path, &destination, &selected) {
                                    Ok(count) => overlay.add_toast(adw::Toast::new(&format!(
                                        "Extracted {count} {}",
                                        if count == 1 { "item" } else { "items" }
                                    ))),
                                    Err(error) => show_error(&overlay, error),
                                }
                            }
                        }
                    }
                    Err(error) if error.matches(gtk::DialogError::Dismissed) => {}
                    Err(error) => show_error(&overlay, error),
                }
            });
        }
    });
}
