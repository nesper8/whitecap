//! Shared toast and file-size presentation helpers.

/// Shows a recoverable archive-operation error in the application overlay.
pub(super) fn show_error(overlay: &adw::ToastOverlay, error: impl std::fmt::Display) {
    overlay.add_toast(adw::Toast::new(&format!(
        "Could not complete operation: {error}"
    )));
}

/// Shows a non-fatal warning in the application overlay.
pub(super) fn show_warning(overlay: &adw::ToastOverlay, message: &str) {
    overlay.add_toast(adw::Toast::new(message));
}

/// Formats a byte count using a human-readable binary unit.
pub(super) fn format_size(size: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = size as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{size} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
