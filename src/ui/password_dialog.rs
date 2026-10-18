//! Presents the password prompt used for encrypted archives.

use adw::prelude::*;
use gtk::gio;

/// Presents an archive password dialog and invokes `callback` with its result.
pub(super) fn request_password(
    window: &adw::ApplicationWindow,
    message: &str,
    callback: impl FnOnce(Option<String>) + 'static,
) {
    let entry = gtk::PasswordEntry::new();
    entry.set_placeholder_text(Some("Archive password"));
    entry.set_show_peek_icon(true);

    let dialog = adw::AlertDialog::new(Some("Password required"), Some(message));
    dialog.set_extra_child(Some(&entry));
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("unlock", "Unlock");
    dialog.set_default_response(Some("unlock"));
    dialog.set_close_response("cancel");
    dialog.choose(Some(window), None::<&gio::Cancellable>, move |response| {
        callback((response == "unlock").then(|| entry.text().to_string()));
    });
}
