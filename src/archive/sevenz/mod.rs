mod add;
mod delete;
mod extract;
mod load_with_password;
mod map_password_error;

pub(super) use add::add;
pub(super) use delete::delete;
pub(super) use extract::extract;
pub(super) use load_with_password::load_with_password;

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::archive::{add_files, delete_entries, temporary_directory};

    #[test]
    fn extracts_only_selected_7z_entry() {
        let root = temporary_directory();
        let source = root.join("source");
        fs::create_dir_all(source.join("docs")).unwrap();
        fs::write(source.join("docs/readme.txt"), b"hello").unwrap();
        fs::write(source.join("other.txt"), b"skip").unwrap();

        let archive_path = root.join("sample.7z");
        sevenz_rust2::compress_to_path(&source, &archive_path).unwrap();
        let entries = load_with_password(&archive_path, None).unwrap();
        assert!(entries.iter().any(|entry| entry.path == "docs/readme.txt"));

        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        let selected = vec![("docs/readme.txt".to_owned(), false)];
        extract(&archive_path, &destination, &selected, None).unwrap();

        assert_eq!(
            fs::read(destination.join("docs/readme.txt")).unwrap(),
            b"hello"
        );
        assert!(!destination.join("other.txt").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn adds_files_and_preserves_existing_7z_entries() {
        let root = temporary_directory();
        let original = root.join("original");
        fs::create_dir_all(&original).unwrap();
        fs::write(original.join("existing.txt"), b"old").unwrap();
        let archive_path = root.join("sample.7z");
        sevenz_rust2::compress_to_path(&original, &archive_path).unwrap();

        let added = root.join("added.txt");
        fs::write(&added, b"new").unwrap();
        assert_eq!(
            add_files(&archive_path, std::slice::from_ref(&added)).unwrap(),
            1
        );

        let archive_entries = load_with_password(&archive_path, None).unwrap();
        assert!(archive_entries.iter().any(|entry| entry.path == "existing.txt"));
        assert!(archive_entries.iter().any(|entry| entry.path == "added.txt"));
        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        extract(&archive_path, &destination, &[], None).unwrap();
        assert_eq!(fs::read(destination.join("existing.txt")).unwrap(), b"old");
        assert_eq!(fs::read(destination.join("added.txt")).unwrap(), b"new");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn deletes_selected_7z_folder_and_preserves_other_entries() {
        let root = temporary_directory();
        let source = root.join("source");
        fs::create_dir_all(source.join("docs")).unwrap();
        fs::write(source.join("docs/readme.txt"), b"remove").unwrap();
        fs::write(source.join("keep.txt"), b"keep").unwrap();
        let archive_path = root.join("sample.7z");
        sevenz_rust2::compress_to_path(&source, &archive_path).unwrap();

        let selected = vec![("docs".to_owned(), true)];
        assert_eq!(delete_entries(&archive_path, &selected).unwrap(), 2);
        let entries = load_with_password(&archive_path, None).unwrap();
        assert!(!entries.iter().any(|entry| entry.path.starts_with("docs")));
        assert!(entries.iter().any(|entry| entry.path == "keep.txt"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn prompts_for_password_and_extracts_encrypted_7z_entries() {
        let root = temporary_directory();
        let source = root.join("source");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("secret.txt"), b"private data").unwrap();
        let archive_path = root.join("encrypted.7z");
        sevenz_rust2::compress_to_path_encrypted(
            &source,
            &archive_path,
            sevenz_rust2::Password::new("correct horse"),
        )
        .unwrap();

        let load_error = load_with_password(&archive_path, None).unwrap_err();
        assert!(matches!(
            load_error.downcast_ref::<super::super::PasswordError>(),
            Some(super::super::PasswordError::Required)
        ));
        let entries = load_with_password(&archive_path, Some("correct horse")).unwrap();
        assert!(entries.iter().any(|entry| entry.encrypted));
        let selected = vec![("secret.txt".to_owned(), false)];
        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();

        let missing_password = extract(&archive_path, &destination, &selected, None).unwrap_err();
        assert!(matches!(
            missing_password.downcast_ref::<super::super::PasswordError>(),
            Some(super::super::PasswordError::Required)
        ));

        let wrong_password =
            extract(&archive_path, &destination, &selected, Some("wrong")).unwrap_err();
        assert!(matches!(
            wrong_password.downcast_ref::<super::super::PasswordError>(),
            Some(super::super::PasswordError::Invalid)
        ));

        assert_eq!(
            extract(
                &archive_path,
                &destination,
                &selected,
                Some("correct horse")
            )
            .unwrap(),
            1
        );
        assert_eq!(
            fs::read(destination.join("secret.txt")).unwrap(),
            b"private data"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
