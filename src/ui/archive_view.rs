use std::path::Path;

use adw::prelude::*;

use crate::archive::ArchiveEntry;

use super::{
    drag_drop,
    helpers::{format_size, show_error},
    state::SharedState,
};

pub(super) fn show_archive(
    path: &Path,
    entries: Vec<ArchiveEntry>,
    state: &SharedState,
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

pub(super) fn refresh_archive(
    entries: Vec<ArchiveEntry>,
    state: &SharedState,
    list: &gtk::ListBox,
    archive_label: &gtk::Label,
    up_button: &gtk::Button,
) {
    let archive_name = {
        let mut state = state.borrow_mut();
        state.entries = entries;
        state
            .archive_path
            .as_deref()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or("Archive")
            .to_owned()
    };
    render_directory(state, list, archive_label, up_button, &archive_name);
}

fn render_directory(
    state: &SharedState,
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

pub(super) fn connect_up_action(
    button: &gtk::Button,
    state: &SharedState,
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

pub(super) fn connect_row_activation(
    list: &gtk::ListBox,
    state: &SharedState,
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
    state: &SharedState,
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
    drag_source.set_actions(gtk::gdk::DragAction::COPY);
    drag_source.connect_prepare({
        let state = state.clone();
        let list = list.clone();
        move |_, _, _| {
            let export = match drag_drop::create_drag_export(&state, &list, index) {
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
