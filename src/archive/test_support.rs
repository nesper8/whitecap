use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

pub(crate) fn temporary_directory() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("whitecap-test-{}-{unique}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}
