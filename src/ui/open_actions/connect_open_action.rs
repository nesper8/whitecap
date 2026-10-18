//! Connects the archive-open button to the file chooser.

use adw::prelude::*;
use gtk::gio;

use crate::ui::{
    archive_view,
    helpers::show_error,
    state::SharedState,
};

/// Connects the open button to the supported-archive file chooser.
pub(crate) fn connect_open_action(
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
