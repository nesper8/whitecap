use adw::prelude::*;
use gtk::{gdk, glib};

pub(super) fn connect_search_and_shortcuts(
    window: &adw::ApplicationWindow,
    search: &gtk::SearchEntry,
    stack: &gtk::Stack,
    list: &gtk::ListBox,
) {
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
}
