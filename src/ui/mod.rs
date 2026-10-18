//! GTK application setup and UI orchestration.
//!
//! Widgets and callbacks live in focused submodules. The UI keeps the current
//! archive, visible directory, password, and temporary drag-export files in a
//! shared state object; archive format details remain in [`crate::archive`].

mod actions;
mod archive_contents;
mod archive_row;
mod archive_view;
mod drop_transport;
mod drag_drop;
mod helpers;
mod password_dialog;
mod state;
mod window;

use adw::prelude::*;
use gtk::{gio, glib};

/// Builds and runs the Whitecap application.
///
/// The application accepts archives opened by the desktop environment through
/// its `HANDLES_OPEN` application flag and also supports opening an empty
/// window when launched without a file.
pub fn run() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id("io.github.whitecap.Archive")
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_activate(|app| window::build_window(app, None));
    app.connect_open(|app, files, _| {
        if files.is_empty() {
            window::build_window(app, None);
            return;
        }
        for file in files {
            window::build_window(app, file.path());
        }
    });
    app.run()
}
