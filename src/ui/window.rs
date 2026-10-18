use std::path::PathBuf;

use adw::prelude::*;
use gtk::{gdk, glib};

use super::{
    actions,
    archive_view,
    drag_drop,
    state::SharedState,
};

pub(super) fn build_window(app: &adw::Application, initial_archive: Option<PathBuf>) {
    let state: SharedState = Default::default();
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Whitecap")
        .default_width(980)
        .default_height(680)
        .build();

    let overlay = adw::ToastOverlay::new();
    state.borrow_mut().toast_overlay = Some(overlay.clone());
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
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
    content.append(&header);

    let toolbar = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let location_bar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    location_bar.set_margin_start(8);
    location_bar.set_margin_end(8);
    location_bar.set_margin_top(8);
    location_bar.set_margin_bottom(8);
    location_bar.set_height_request(56);
    let up_button = gtk::Button::from_icon_name("go-up-symbolic");
    up_button.set_tooltip_text(Some("Go to parent folder"));
    up_button.set_sensitive(false);
    let archive_label = gtk::Label::new(Some("Drop an archive here, or open one."));
    archive_label.set_xalign(0.0);
    archive_label.set_hexpand(true);
    archive_label.add_css_class("dim-label");
    location_bar.append(&archive_label);
    toolbar.append(&location_bar);
    let row_separator = gtk::Separator::new(gtk::Orientation::Horizontal);
    toolbar.append(&row_separator);

    let action_bar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    action_bar.set_margin_start(8);
    action_bar.set_margin_end(8);
    action_bar.set_margin_top(8);
    action_bar.set_margin_bottom(8);
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
    content.append(&toolbar);
    content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::Crossfade);
    stack.set_vexpand(true);
    stack.set_hexpand(true);

    let welcome = adw::StatusPage::builder()
        .icon_name("package-x-generic-symbolic")
        .title("Open an archive")
        .description(
            concat!(
                "Drop an archive onto the location bar to open it.\n\n",
                "Editing and extraction: ZIP, 7z, and TAR.\n",
                "Viewing and extraction only: RAR, ACE, and FreeArc."
            ),
        )
        .build();
    stack.add_named(&welcome, Some("welcome"));

    let list = gtk::ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::Multiple);
    list.set_activate_on_single_click(false);
    list.add_css_class("boxed-list");
    list.set_margin_start(8);
    list.set_margin_end(8);
    list.set_margin_top(18);
    list.set_margin_bottom(18);
    list.set_filter_func({
        let search = search.clone();
        move |row| {
            let query = search.text().to_string().to_lowercase();
            if query.is_empty() {
                return true;
            }
            row.child()
                .and_then(|child| child.downcast::<gtk::Box>().ok())
                .and_then(|line| line.first_child())
                .and_then(|icon| icon.next_sibling())
                .and_then(|child| child.downcast::<gtk::Label>().ok())
                .is_some_and(|label| label.text().to_lowercase().contains(&query))
        }
    });
    let scrolled = gtk::ScrolledWindow::builder()
        .child(&list)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .build();
    stack.add_named(&scrolled, Some("contents"));
    content.append(&stack);
    overlay.set_child(Some(&content));
    window.set_content(Some(&overlay));

    search.connect_search_changed({
        let list = list.clone();
        move |_| list.invalidate_filter()
    });
    let select_all_keys = gtk::EventControllerKey::new();
    select_all_keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    select_all_keys.connect_key_pressed({
        let list = list.clone();
        let stack = stack.clone();
        let window = window.clone();
        move |_, key, _, modifiers| {
            let is_select_all = key
                .name()
                .is_some_and(|name| name.eq_ignore_ascii_case("a"));
            if is_select_all && modifiers.contains(gdk::ModifierType::CONTROL_MASK) {
                if stack.visible_child_name().as_deref() == Some("contents") {
                    if let Some(focus) = gtk::prelude::GtkWindowExt::focus(&window) {
                        if focus.is::<gtk::Editable>()
                            || focus.ancestor(gtk::Editable::static_type()).is_some()
                        {
                            return glib::Propagation::Proceed;
                        }
                    }
                    list.select_all();
                    list.grab_focus();
                    return glib::Propagation::Stop;
                }
            }
            glib::Propagation::Proceed
        }
    });
    window.add_controller(select_all_keys);

    actions::connect_open_action(
        &open_button,
        &window,
        &overlay,
        &state,
        &list,
        &stack,
        &title,
        &archive_label,
        &extract_button,
        &up_button,
    );
    actions::connect_extract_action(&extract_button, &window, &overlay, &state, &list);
    actions::connect_delete_action(
        &delete_button,
        &window,
        &overlay,
        &state,
        &list,
        &archive_label,
        &up_button,
    );
    archive_view::connect_up_action(&up_button, &state, &list, &archive_label);
    archive_view::connect_row_activation(&list, &state, &list, &archive_label, &up_button);
    drag_drop::connect_archive_drop(
        &location_bar,
        &window,
        &overlay,
        &state,
        &list,
        &stack,
        &title,
        &archive_label,
        &extract_button,
        &up_button,
    );
    drag_drop::connect_add_files_drop(
        &list,
        &overlay,
        &state,
        &list,
        &stack,
        &title,
        &archive_label,
        &extract_button,
        &up_button,
    );
    window.present();
    if let Some(path) = initial_archive {
        drag_drop::load_dropped_archive(
            &path,
            &window,
            &overlay,
            &state,
            &list,
            &stack,
            &title,
            &archive_label,
            &extract_button,
            &up_button,
        );
    }
}
