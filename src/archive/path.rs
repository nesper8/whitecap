use std::path::{Component, Path};

pub(super) fn normalize(path: &str) -> String {
    Path::new(path)
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

pub(super) fn is_selected(path: &str, selected: &[(String, bool)]) -> bool {
    selected.is_empty()
        || selected.iter().any(|(selected_path, is_dir)| {
            path == selected_path || (*is_dir && is_descendant(path, selected_path))
        })
}

fn is_descendant(path: &str, directory: &str) -> bool {
    let directory = directory.trim_end_matches('/');
    path.strip_prefix(directory)
        .is_some_and(|suffix| suffix.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_archive_paths() {
        assert_eq!(normalize("./docs/readme.txt"), "docs/readme.txt");
        assert_eq!(normalize("../outside.txt"), "outside.txt");
    }

    #[test]
    fn selects_directory_contents() {
        let selected = vec![("docs".to_owned(), true)];
        assert!(is_selected("docs/readme.txt", &selected));
        assert!(!is_selected("other.txt", &selected));
    }
}
