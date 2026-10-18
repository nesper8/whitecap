mod actions;
mod archive_view;
mod drag_drop;
mod helpers;
mod state;
mod window;

use adw::prelude::*;
use gtk::{gio, glib};

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
