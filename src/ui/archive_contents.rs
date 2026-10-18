//! Builds the immediate children visible for the current archive directory.

use std::collections::BTreeMap;

use crate::archive::ArchiveEntry;

/// Returns the immediate children of `directory`, synthesizing folder rows
/// when an archive omits explicit directory entries.
pub(super) fn entries_in_directory(entries: &[ArchiveEntry], directory: &str) -> Vec<ArchiveEntry> {
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
                encrypted: false,
            });
        } else {
            children.insert(entry.path.clone(), entry.clone());
        }
    }

    let mut children = children.into_values().collect::<Vec<_>>();
    children.sort_by_cached_key(|entry| (!entry.is_dir, entry.path.to_lowercase()));
    children
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
                encrypted: false,
            },
            ArchiveEntry {
                path: "project/README.md".to_owned(),
                is_dir: false,
                size: 80,
                encrypted: false,
            },
            ArchiveEntry {
                path: "LICENSE".to_owned(),
                is_dir: false,
                size: 40,
                encrypted: false,
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
