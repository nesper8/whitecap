use adw::prelude::*;

pub(super) fn build_content_stack(search: &gtk::SearchEntry) -> (gtk::Stack, gtk::ListBox) {
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

    (stack, list)
}
