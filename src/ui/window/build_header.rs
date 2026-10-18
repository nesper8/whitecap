//! Builds the application header and its primary controls.

use adw::prelude::*;

/// Header widgets shared by action callbacks.
pub(super) struct HeaderWidgets {
    /// Application header bar.
    pub(super) header: adw::HeaderBar,
    /// Label used for the current archive title.
    pub(super) title: gtk::Label,
    /// Button that opens an archive chooser.
    pub(super) open_button: gtk::Button,
    /// Button that starts extraction.
    pub(super) extract_button: gtk::Button,
}

/// Constructs the application header and its primary actions.
pub(super) fn build_header() -> HeaderWidgets {
    let header = adw::HeaderBar::new();
    let title = gtk::Label::new(Some("Whitecap"));
    title.add_css_class("title");
    header.set_title_widget(Some(&title));

    let open_button = gtk::Button::from_icon_name("document-open-symbolic");
    open_button.set_tooltip_text(Some("Open an archive"));
    header.pack_start(&open_button);

    let extract_button = gtk::Button::with_label("Extract");
    extract_button.add_css_class("suggested-action");
    extract_button.set_sensitive(false);
    header.pack_end(&extract_button);

    HeaderWidgets {
        header,
        title,
        open_button,
        extract_button,
    }
}
