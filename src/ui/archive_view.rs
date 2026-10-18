use std::path::Path;

use adw::prelude::*;

use crate::archive::ArchiveEntry;

use super::{archive_contents::entries_in_directory, archive_row::archive_entry_row, state::SharedState};

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
