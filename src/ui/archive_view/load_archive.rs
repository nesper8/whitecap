//! Loads archive entries and handles password retries.

use std::path::PathBuf;

use crate::archive;
use crate::ui::{
    helpers::show_error,
    password_dialog::request_password,
    state::SharedState,
};

#[allow(clippy::too_many_arguments)]
/// Loads archive entries and retries through the password dialog when needed.
pub(crate) fn load_archive(
    path: PathBuf,
    password: Option<String>,
    window: adw::ApplicationWindow,
    overlay: adw::ToastOverlay,
    state: SharedState,
    list: gtk::ListBox,
    stack: gtk::Stack,
    title: gtk::Label,
    archive_label: gtk::Label,
    extract_button: gtk::Button,
    up_button: gtk::Button,
) {
    match archive::load_with_password(&path, password.as_deref()) {
        Ok(entries) => super::show_archive::show_archive(
            &path,
            entries,
            password,
            &state,
            &list,
            &stack,
            &title,
            &archive_label,
            &extract_button,
            &up_button,
        ),
        Err(error) => match error.downcast_ref::<archive::PasswordError>() {
            Some(password_error) => {
                let prompt = if *password_error == archive::PasswordError::Invalid {
                    "The password was incorrect. Try again."
                } else {
                    "Enter the password to open this encrypted archive."
                };
                let retry_window = window.clone();
                request_password(&window, prompt, move |password| {
                    if let Some(password) = password {
                        load_archive(
                            path,
                            Some(password),
                            retry_window,
                            overlay,
                            state,
                            list,
                            stack,
                            title,
                            archive_label,
                            extract_button,
                            up_button,
                        );
                    }
                });
            }
            None => show_error(&overlay, error),
        },
    }
}
