//! Performs extraction and routes password failures back to the dialog.

use crate::archive;
use crate::ui::{helpers::show_error, state::SharedState};

/// Extracts the current selection or asks for a missing archive password.
pub(crate) fn extract_selected(
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
        super::request_extract_password::request_extract_password(
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
                super::request_extract_password::request_extract_password(
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
