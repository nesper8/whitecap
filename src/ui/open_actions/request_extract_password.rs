//! Retries extraction after the user supplies a password.

use crate::ui::{password_dialog::request_password, state::SharedState};

/// Reopens the extraction flow after the user supplies a password.
pub(crate) fn request_extract_password(
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
            super::extract_selected::extract_selected(
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
