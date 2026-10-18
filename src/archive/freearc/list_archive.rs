use std::{error::Error, path::Path, process::Command};

use super::{
    absolute_path::absolute_path,
    entry::ListedEntry,
    parse_listing::parse_listing,
    run_unarc::run_unarc,
};

pub(crate) fn list_archive(archive_path: &Path) -> Result<Vec<ListedEntry>, Box<dyn Error>> {
    let mut command = Command::new("unarc");
    command
        .arg("l")
        .arg("--noarcext")
        .arg(absolute_path(archive_path)?);
    let output = run_unarc(command)?;
    parse_listing(&String::from_utf8_lossy(&output.stdout))
}
