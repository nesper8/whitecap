use adw::prelude::*;

pub(super) struct ArchiveControls {
    pub(super) toolbar: gtk::Box,
    pub(super) location_bar: gtk::Box,
    pub(super) archive_label: gtk::Label,
    pub(super) up_button: gtk::Button,
    pub(super) search: gtk::SearchEntry,
    pub(super) delete_button: gtk::Button,
}

pub(super) fn build_archive_controls() -> ArchiveControls {
    let toolbar = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let location_bar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    location_bar.set_margin_start(8);
    location_bar.set_margin_end(8);
    location_bar.set_margin_top(8);
    location_bar.set_margin_bottom(8);
    location_bar.set_height_request(56);
    let archive_label = gtk::Label::new(Some("Drop an archive here, or open one."));
    archive_label.set_xalign(0.0);
    archive_label.set_hexpand(true);
    archive_label.add_css_class("dim-label");
    location_bar.append(&archive_label);
    toolbar.append(&location_bar);
    toolbar.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    let action_bar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    action_bar.set_margin_start(8);
    action_bar.set_margin_end(8);
    action_bar.set_margin_top(8);
    action_bar.set_margin_bottom(8);
    let up_button = gtk::Button::from_icon_name("go-up-symbolic");
    up_button.set_tooltip_text(Some("Go to parent folder"));
    up_button.set_sensitive(false);
    action_bar.append(&up_button);
    action_bar.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some("Search contents"));
    search.set_width_chars(24);
    search.set_hexpand(true);
    action_bar.append(&search);
    action_bar.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    let delete_button = gtk::Button::from_icon_name("user-trash-symbolic");
    delete_button.set_tooltip_text(Some("Delete selected entries"));
    delete_button.set_sensitive(false);
    action_bar.append(&delete_button);
    toolbar.append(&action_bar);

    ArchiveControls {
        toolbar,
        location_bar,
        archive_label,
        up_button,
        search,
        delete_button,
    }
}
