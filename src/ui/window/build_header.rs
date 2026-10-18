use adw::prelude::*;

pub(super) struct HeaderWidgets {
    pub(super) header: adw::HeaderBar,
    pub(super) title: gtk::Label,
    pub(super) open_button: gtk::Button,
    pub(super) extract_button: gtk::Button,
}

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
