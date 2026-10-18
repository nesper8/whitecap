use std::{
    cell::RefCell,
    error::Error,
    io,
    path::{Path, PathBuf},
    rc::Rc,
};

use adw::prelude::*;
use gtk::{gdk, gio, glib};

use crate::archive::{self, ArchiveEntry};

#[derive(Default)]
struct AppState {
    archive_path: Option<PathBuf>,
    entries: Vec<ArchiveEntry>,
    visible_entries: Vec<ArchiveEntry>,
    current_directory: String,
    drag_exports: Vec<tempfile::TempDir>,
    toast_overlay: Option<adw::ToastOverlay>,
}

pub fn run() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id("io.github.whitecap.Archive")
        .build();

    app.connect_activate(build_window);
    app.run()
}

fn build_window(app: &adw::Application) {
    let state = Rc::new(RefCell::new(AppState::default()));
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

    let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    toolbar.set_margin_start(18);
    toolbar.set_margin_end(18);
    toolbar.set_margin_top(12);
    toolbar.set_margin_bottom(12);
    let up_button = gtk::Button::from_icon_name("go-up-symbolic");
    up_button.set_tooltip_text(Some("Go to parent folder"));
    up_button.set_sensitive(false);
    toolbar.append(&up_button);
    let archive_label = gtk::Label::new(Some("Drop an archive here, or open one"));
    archive_label.set_xalign(0.0);
    archive_label.set_hexpand(true);
    archive_label.add_css_class("dim-label");
    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some("Search contents"));
    search.set_width_chars(24);
    toolbar.append(&archive_label);
    toolbar.append(&search);
    content.append(&toolbar);
    content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::Crossfade);
    stack.set_vexpand(true);
    stack.set_hexpand(true);

    let welcome = adw::StatusPage::builder()
        .icon_name("package-x-generic-symbolic")
        .title("Open an archive")
        .description("Browse its contents, then extract everything or just the files you need.")
        .build();
    stack.add_named(&welcome, Some("welcome"));

    let list = gtk::ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::Multiple);
    list.set_activate_on_single_click(false);
    list.add_css_class("boxed-list");
    list.set_margin_start(18);
    list.set_margin_end(18);
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
        move |_, key, _, modifiers| {
            let is_select_all = key
                .name()
                .is_some_and(|name| name.eq_ignore_ascii_case("a"));
            if is_select_all && modifiers.contains(gdk::ModifierType::CONTROL_MASK) {
                list.select_all();
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        }
    });
    list.add_controller(select_all_keys);

    connect_open_action(
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
    connect_extract_action(&extract_button, &window, &overlay, &state, &list);
    connect_up_action(&up_button, &state, &list, &archive_label);
    connect_row_activation(&list, &state, &list, &archive_label, &up_button);
    connect_archive_drop(
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
    window.present();
}

fn connect_open_action(
    button: &gtk::Button,
    window: &adw::ApplicationWindow,
    overlay: &adw::ToastOverlay,
    state: &Rc<RefCell<AppState>>,
    list: &gtk::ListBox,
    stack: &gtk::Stack,
    title: &gtk::Label,
    archive_label: &gtk::Label,
    extract_button: &gtk::Button,
    up_button: &gtk::Button,
) {
    button.connect_clicked({
        let window = window.clone();
        let overlay = overlay.clone();
        let state = state.clone();
        let list = list.clone();
        let stack = stack.clone();
        let title = title.clone();
        let archive_label = archive_label.clone();
        let extract_button = extract_button.clone();
        let up_button = up_button.clone();
        move |_| {
            let dialog = gtk::FileDialog::new();
            dialog.set_title("Open Archive");
            let filter = gtk::FileFilter::new();
            filter.set_name(Some("Supported archives"));
            for pattern in [
                "*.zip", "*.tar", "*.tar.gz", "*.tgz", "*.tar.bz2", "*.tbz2", "*.tar.xz",
                "*.txz", "*.tar.zst", "*.tzst", "*.7z", "*.rar", "*.arc", "*.ace",
            ] {
                filter.add_pattern(pattern);
            }
            dialog.set_default_filter(Some(&filter));
            dialog.open(Some(&window), None::<&gio::Cancellable>, {
                let overlay = overlay.clone();
                let state = state.clone();
                let list = list.clone();
                let stack = stack.clone();
                let title = title.clone();
                let archive_label = archive_label.clone();
                let extract_button = extract_button.clone();
                let up_button = up_button.clone();
                move |result| match result {
                    Ok(file) => {
                        if let Some(path) = file.path() {
                            match archive::load(&path) {
                                Ok(entries) => show_archive(
                                    &path,
                                    entries,
                                    &state,
                                    &list,
                                    &stack,
                                    &title,
                                    &archive_label,
                                    &extract_button,
                                    &up_button,
                                ),
                                Err(error) => show_error(&overlay, error),
                            }
                        }
                    }
                    Err(error) if error.matches(gtk::DialogError::Dismissed) => {}
                    Err(error) => show_error(&overlay, error),
                }
            });
        }
    });
}

fn connect_extract_action(
    button: &gtk::Button,
    window: &adw::ApplicationWindow,
    overlay: &adw::ToastOverlay,
    state: &Rc<RefCell<AppState>>,
    list: &gtk::ListBox,
) {
    button.connect_clicked({
        let window = window.clone();
        let overlay = overlay.clone();
        let state = state.clone();
        let list = list.clone();
        move |_| {
            let dialog = gtk::FileDialog::new();
            dialog.set_title("Choose Extraction Folder");
            dialog.select_folder(Some(&window), None::<&gio::Cancellable>, {
                let overlay = overlay.clone();
                let state = state.clone();
                let list = list.clone();
                move |result| match result {
                    Ok(folder) => {
                        if let Some(destination) = folder.path() {
                            let state = state.borrow();
                            if let Some(archive_path) = &state.archive_path {
                                let selected = list
                                    .selected_rows()
                                    .iter()
                                    .filter_map(|row| {
                                        row.widget_name()
                                            .strip_prefix("archive-entry-")?
                                            .parse::<usize>()
                                            .ok()
                                            .and_then(|index| state.visible_entries.get(index))
                                    })
                                    .map(|entry| (entry.path.clone(), entry.is_dir))
                                    .collect::<Vec<_>>();
                                match archive::extract(archive_path, &destination, &selected) {
                                    Ok(count) => overlay.add_toast(adw::Toast::new(&format!(
                                        "Extracted {count} {}",
                                        if count == 1 { "item" } else { "items" }
                                    ))),
                                    Err(error) => show_error(&overlay, error),
                                }
                            }
                        }
                    }
                    Err(error) if error.matches(gtk::DialogError::Dismissed) => {}
                    Err(error) => show_error(&overlay, error),
                }
            });
        }
    });
}

fn connect_archive_drop(
    window: &adw::ApplicationWindow,
    overlay: &adw::ToastOverlay,
    state: &Rc<RefCell<AppState>>,
    list: &gtk::ListBox,
    stack: &gtk::Stack,
    title: &gtk::Label,
    archive_label: &gtk::Label,
    extract_button: &gtk::Button,
    up_button: &gtk::Button,
) {
    let file_list_formats = gdk::ContentFormats::for_type(gdk::FileList::static_type());
    let uri_formats = gdk::ContentFormats::new(&["text/uri-list"]);
    let formats = file_list_formats.union(&uri_formats);
    let drop_target = gtk::DropTargetAsync::new(Some(formats), gdk::DragAction::COPY);
    drop_target.set_propagation_phase(gtk::PropagationPhase::Capture);
    drop_target.connect_drop({
        let overlay = overlay.clone();
        let state = state.clone();
        let list = list.clone();
        let stack = stack.clone();
        let title = title.clone();
        let archive_label = archive_label.clone();
        let extract_button = extract_button.clone();
        let up_button = up_button.clone();
        move |_, drop, _, _| {
            let drop_for_callback = drop.clone();
            let overlay = overlay.clone();
            let state = state.clone();
            let list = list.clone();
            let stack = stack.clone();
            let title = title.clone();
            let archive_label = archive_label.clone();
            let extract_button = extract_button.clone();
            let up_button = up_button.clone();
            drop.read_value_async(
                gdk::FileList::static_type(),
                glib::Priority::DEFAULT,
                None::<&gio::Cancellable>,
                move |result| match result {
                    Ok(value) => {
                        if let Ok(files) = value.get::<gdk::FileList>() {
                            if let Some(path) =
                                files.files().first().and_then(gio::File::path)
                            {
                                load_dropped_archive(
                                    &path,
                                    &overlay,
                                    &state,
                                    &list,
                                    &stack,
                                    &title,
                                    &archive_label,
                                    &extract_button,
                                    &up_button,
                                );
                            } else {
                                show_error(&overlay, "The dropped item is not a local file");
                            }
                        } else {
                            show_error(&overlay, "The dropped data is not a file list");
                        }
                        drop_for_callback.finish(gdk::DragAction::COPY);
                    }
                    Err(_) => {
                        read_dropped_uri_list(
                            &drop_for_callback,
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
            );
            true
        }
    });
    window.add_controller(drop_target);
}

fn read_dropped_uri_list(
    drop: &gdk::Drop,
    overlay: &adw::ToastOverlay,
    state: &Rc<RefCell<AppState>>,
    list: &gtk::ListBox,
    stack: &gtk::Stack,
    title: &gtk::Label,
    archive_label: &gtk::Label,
    extract_button: &gtk::Button,
    up_button: &gtk::Button,
) {
    let drop = drop.clone();
    let overlay = overlay.clone();
    let state = state.clone();
    let list = list.clone();
    let stack = stack.clone();
    let title = title.clone();
    let archive_label = archive_label.clone();
    let extract_button = extract_button.clone();
    let up_button = up_button.clone();
    let drop_to_finish = drop.clone();
    drop.read_async(
        &["text/uri-list"],
        glib::Priority::DEFAULT,
        None::<&gio::Cancellable>,
        move |result| match result {
            Ok((stream, _)) => {
                stream.read_bytes_async(
                    64 * 1024,
                    glib::Priority::DEFAULT,
                    None::<&gio::Cancellable>,
                    move |result| match result {
                        Ok(bytes) => {
                            let text = String::from_utf8_lossy(bytes.as_ref());
                            if let Some(path) = text
                                .lines()
                                .map(str::trim)
                                .find(|line| !line.is_empty() && !line.starts_with('#'))
                                .and_then(|uri| gio::File::for_uri(uri).path())
                            {
                                load_dropped_archive(
                                    &path,
                                    &overlay,
                                    &state,
                                    &list,
                                    &stack,
                                    &title,
                                    &archive_label,
                                    &extract_button,
                                    &up_button,
                                );
                            } else {
                                show_error(&overlay, "No local file was found in the dropped data");
                            }
                            drop_to_finish.finish(gdk::DragAction::COPY);
                        }
                        Err(error) => {
                            show_error(&overlay, error);
                            drop_to_finish.finish(gdk::DragAction::empty());
                        }
                    },
                );
            }
            Err(error) => {
                show_error(&overlay, error);
                drop_to_finish.finish(gdk::DragAction::empty());
            }
        },
    );
}

fn load_dropped_archive(
    path: &Path,
    overlay: &adw::ToastOverlay,
    state: &Rc<RefCell<AppState>>,
    list: &gtk::ListBox,
    stack: &gtk::Stack,
    title: &gtk::Label,
    archive_label: &gtk::Label,
    extract_button: &gtk::Button,
    up_button: &gtk::Button,
) {
    match archive::load(path) {
        Ok(entries) => show_archive(
            path,
            entries,
            state,
            list,
            stack,
            title,
            archive_label,
            extract_button,
            up_button,
        ),
        Err(error) => show_error(overlay, error),
    }
}

fn show_archive(
    path: &Path,
    entries: Vec<ArchiveEntry>,
    state: &Rc<RefCell<AppState>>,
    list: &gtk::ListBox,
    stack: &gtk::Stack,
    title: &gtk::Label,
    archive_label: &gtk::Label,
    extract_button: &gtk::Button,
    up_button: &gtk::Button,
) {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Archive")
        .to_owned();
    {
        let mut state = state.borrow_mut();
        state.archive_path = Some(path.to_path_buf());
        state.entries = entries;
        state.visible_entries.clear();
        state.current_directory.clear();
    }
    render_directory(state, list, archive_label, up_button, &file_name);

    title.set_text(&file_name);
    extract_button.set_sensitive(true);
    stack.set_visible_child_name("contents");
}

fn render_directory(
    state: &Rc<RefCell<AppState>>,
    list: &gtk::ListBox,
    location_label: &gtk::Label,
    up_button: &gtk::Button,
    archive_name: &str,
) {
    let (current_directory, entries) = {
        let state = state.borrow();
        (state.current_directory.clone(), state.entries.clone())
    };
    let visible_entries = entries_in_directory(&entries, &current_directory);

    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    for (index, entry) in visible_entries.iter().enumerate() {
        list.append(&archive_entry_row(index, entry, state, list));
    }

    let location = if current_directory.is_empty() {
        format!("{archive_name} /")
    } else {
        format!("{archive_name} / {current_directory}")
    };
    location_label.set_text(&location);
    up_button.set_sensitive(!current_directory.is_empty());
    state.borrow_mut().visible_entries = visible_entries;
}

fn entries_in_directory(entries: &[ArchiveEntry], directory: &str) -> Vec<ArchiveEntry> {
    use std::collections::BTreeMap;

    let prefix = if directory.is_empty() {
        String::new()
    } else {
        format!("{directory}/")
    };
    let mut children = BTreeMap::<String, ArchiveEntry>::new();

    for entry in entries {
        let Some(remainder) = entry.path.strip_prefix(&prefix) else {
            continue;
        };
        if remainder.is_empty() {
            continue;
        }
        if let Some((child_name, _)) = remainder.split_once('/') {
            let child_path = format!("{prefix}{child_name}");
            children.entry(child_path.clone()).or_insert(ArchiveEntry {
                path: child_path,
                is_dir: true,
                size: 0,
            });
        } else {
            children.insert(entry.path.clone(), entry.clone());
        }
    }

    let mut children = children.into_values().collect::<Vec<_>>();
    children.sort_by(|left, right| {
        right
            .is_dir
            .cmp(&left.is_dir)
            .then_with(|| left.path.to_lowercase().cmp(&right.path.to_lowercase()))
    });
    children
}

fn connect_up_action(
    button: &gtk::Button,
    state: &Rc<RefCell<AppState>>,
    list: &gtk::ListBox,
    location_label: &gtk::Label,
) {
    let state = state.clone();
    let list = list.clone();
    let location_label = location_label.clone();
    let button_ref = button.clone();
    button.connect_clicked(move |_| {
        let archive_name = state
            .borrow()
            .archive_path
            .as_deref()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or("Archive")
            .to_owned();
        {
            let mut state = state.borrow_mut();
            if let Some((parent, _)) = state.current_directory.rsplit_once('/') {
                state.current_directory = parent.to_owned();
            } else {
                state.current_directory.clear();
            }
        }
        render_directory(&state, &list, &location_label, &button_ref, &archive_name);
    });
}

fn connect_row_activation(
    list: &gtk::ListBox,
    state: &Rc<RefCell<AppState>>,
    archive_list: &gtk::ListBox,
    location_label: &gtk::Label,
    up_button: &gtk::Button,
) {
    let state = state.clone();
    let archive_list = archive_list.clone();
    let location_label = location_label.clone();
    let up_button = up_button.clone();
    list.connect_row_activated(move |_, row| {
        let Some(index) = row
            .widget_name()
            .strip_prefix("archive-entry-")
            .and_then(|index| index.parse::<usize>().ok())
        else {
            return;
        };
        let mut state_ref = state.borrow_mut();
        let Some(entry) = state_ref.visible_entries.get(index) else {
            return;
        };
        if !entry.is_dir {
            return;
        }
        let directory = entry.path.clone();
        state_ref.current_directory = directory;
        let archive_name = state_ref
            .archive_path
            .as_deref()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or("Archive")
            .to_owned();
        drop(state_ref);
        render_directory(
            &state,
            &archive_list,
            &location_label,
            &up_button,
            &archive_name,
        );
    });
}

fn archive_entry_row(
    index: usize,
    entry: &ArchiveEntry,
    state: &Rc<RefCell<AppState>>,
    list: &gtk::ListBox,
) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.set_widget_name(&format!("archive-entry-{index}"));
    let line = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    line.set_margin_start(12);
    line.set_margin_end(12);
    line.set_margin_top(8);
    line.set_margin_bottom(8);

    let icon = gtk::Image::from_icon_name(if entry.is_dir {
        "folder-symbolic"
    } else {
        "text-x-generic-symbolic"
    });
    let name = gtk::Label::new(Some(
        Path::new(&entry.path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&entry.path),
    ));
    name.set_xalign(0.0);
    name.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    name.set_hexpand(true);
    let size_text = if entry.is_dir {
        String::new()
    } else {
        format_size(entry.size)
    };
    let size = gtk::Label::new(Some(&size_text));
    size.add_css_class("dim-label");
    line.append(&icon);
    line.append(&name);
    line.append(&size);
    row.set_child(Some(&line));

    let drag_source = gtk::DragSource::new();
    drag_source.set_actions(gdk::DragAction::COPY);
    drag_source.connect_prepare({
        let state = state.clone();
        let list = list.clone();
        move |_, _, _| {
            let export = match create_drag_export(&state, &list, index) {
                Ok(export) => export,
                Err(error) => {
                    if let Some(overlay) = &state.borrow().toast_overlay {
                        show_error(overlay, error);
                    }
                    return None;
                }
            };
            Some(export)
        }
    });
    row.add_controller(drag_source);
    row
}

fn create_drag_export(
    state: &Rc<RefCell<AppState>>,
    list: &gtk::ListBox,
    dragged_index: usize,
) -> Result<gdk::ContentProvider, Box<dyn Error>> {
    let (archive_path, selected) = {
        let state = state.borrow();
        let archive_path = state
            .archive_path
            .clone()
            .ok_or_else(|| io::Error::other("No archive is open"))?;
        let selected_rows = list.selected_rows();
        let selected_indices = if selected_rows.iter().any(|row| {
            row.widget_name()
                .strip_prefix("archive-entry-")
                .and_then(|index| index.parse::<usize>().ok())
                == Some(dragged_index)
        }) {
            selected_rows
                .iter()
                .filter_map(|row| {
                    row.widget_name()
                        .strip_prefix("archive-entry-")?
                        .parse::<usize>()
                        .ok()
                })
                .collect::<Vec<_>>()
        } else {
            vec![dragged_index]
        };
        let selected = selected_indices
            .into_iter()
            .filter_map(|index| state.visible_entries.get(index))
            .map(|entry| (entry.path.clone(), entry.is_dir))
            .collect::<Vec<_>>();
        (archive_path, selected)
    };

    if selected.is_empty() {
        return Err(io::Error::other("No archive entries are selected for dragging").into());
    }

    let export_directory = tempfile::Builder::new()
        .prefix("whitecap-drag-")
        .tempdir()?;
    archive::extract(&archive_path, export_directory.path(), &selected)?;

    let files = selected
        .iter()
        .map(|(path, _)| gio::File::for_path(export_directory.path().join(path)))
        .collect::<Vec<_>>();
    let file_list = gdk::FileList::from_array(&files);
    let file_provider = gdk::ContentProvider::for_value(&file_list.to_value());
    let uri_list = files
        .iter()
        .map(|file| file.uri().to_string())
        .collect::<Vec<_>>()
        .join("\r\n")
        + "\r\n";
    let uri_bytes = glib::Bytes::from_owned(uri_list.into_bytes());
    let uri_provider = gdk::ContentProvider::for_bytes("text/uri-list", &uri_bytes);
    let provider = gdk::ContentProvider::new_union(&[file_provider, uri_provider]);

    state.borrow_mut().drag_exports.push(export_directory);
    Ok(provider)
}

fn show_error(overlay: &adw::ToastOverlay, error: impl std::fmt::Display) {
    overlay.add_toast(adw::Toast::new(&format!("Could not complete operation: {error}")));
}

fn format_size(size: u64) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_contents_are_grouped_into_immediate_children() {
        let entries = vec![
            ArchiveEntry {
                path: "project/src/main.rs".to_owned(),
                is_dir: false,
                size: 120,
            },
            ArchiveEntry {
                path: "project/README.md".to_owned(),
                is_dir: false,
                size: 80,
            },
            ArchiveEntry {
                path: "LICENSE".to_owned(),
                is_dir: false,
                size: 40,
            },
        ];

        let root = entries_in_directory(&entries, "");
        assert_eq!(root.len(), 2);
        assert_eq!(root[0].path, "project");
        assert!(root[0].is_dir);
        assert_eq!(root[1].path, "LICENSE");

        let project = entries_in_directory(&entries, "project");
        assert_eq!(project.len(), 2);
        assert_eq!(project[0].path, "project/src");
        assert_eq!(project[1].path, "project/README.md");
    }
}
