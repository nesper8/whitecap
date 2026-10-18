//! Extracts selected entries through the external FreeArc `unarc` tool.

use std::{error::Error, fs, io, path::Path, process::Command};

use crate::archive::path;

use super::{
    absolute_path::absolute_path,
    list_archive::list_archive,
    validate_relative_path::validate_relative_path,
};

/// Extracts all or selected FreeArc entries after path validation.
pub(crate) fn extract(
    archive_path: &Path,
    destination: &Path,
    selected: &[(String, bool)],
) -> Result<usize, Box<dyn Error>> {
    let entries = list_archive(archive_path)?;
    let selected_paths = if selected.is_empty() {
        for entry in &entries {
            validate_relative_path(&entry.archive_path)?;
        }
        Vec::new()
    } else {
        entries
            .iter()
            .filter(|entry| path::is_selected(&entry.entry.path, selected))
            .map(|entry| {
                validate_relative_path(&entry.archive_path)?;
                Ok(entry.archive_path.clone())
            })
            .collect::<Result<Vec<_>, io::Error>>()?
    };

    if !selected.is_empty() && selected_paths.is_empty() {
        return Ok(0);
    }

    let extracted_count = if selected.is_empty() {
        entries.len()
    } else {
        selected_paths.len()
    };
    let destination = absolute_path(destination)?;
    fs::create_dir_all(&destination)?;

    let mut command = Command::new("unarc");
    command
        .arg("x")
        .arg("--noarcext")
        .arg("-o+")
        .arg(format!("-dp{}", destination.display()))
        .arg(absolute_path(archive_path)?);
    if !selected.is_empty() {
        command.args(&selected_paths);
    }
    super::run_unarc::run_unarc(command)?;
    Ok(extracted_count)
}
