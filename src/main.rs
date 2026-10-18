//! Whitecap is a GNOME archive browser built with GTK 4 and libadwaita.
//!
//! The application is split into two major subsystems:
//!
//! - [`archive`] provides format detection, listing, extraction, and the
//!   supported archive mutations.
//! - [`ui`] owns the application lifecycle and the GTK presentation layer.

mod archive;
mod ui;

fn main() -> gtk::glib::ExitCode {
    ui::run()
}
