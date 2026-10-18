use std::{
    error::Error,
    fs,
    io,
    path::{Component, Path, PathBuf},
    process::{Command, Output},
};

use super::{ArchiveEntry, path};

struct ListedEntry {
    entry: ArchiveEntry,
    archive_path: String,
}

pub(super) fn load(archive_path: &Path) -> Result<Vec<ArchiveEntry>, Box<dyn Error>> {
    Ok(list_archive(archive_path)?
        .into_iter()
        .map(|entry| entry.entry)
        .collect())
}

pub(super) fn extract(
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
    run_unarc(command)?;
    Ok(extracted_count)
}

fn list_archive(archive_path: &Path) -> Result<Vec<ListedEntry>, Box<dyn Error>> {
    let mut command = Command::new("unarc");
    command
        .arg("l")
        .arg("--noarcext")
        .arg(absolute_path(archive_path)?);
    let output = run_unarc(command)?;
    parse_listing(&String::from_utf8_lossy(&output.stdout))
}

fn run_unarc(mut command: Command) -> Result<Output, Box<dyn Error>> {
    let output = command.output().map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            io::Error::new(
                io::ErrorKind::NotFound,
                "FreeArc support requires the `unarc` executable on PATH. Install the FreeArc unarc extractor and try again.",
            )
        } else {
            error
        }
    })?;
    if output.status.success() {
        Ok(output)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        Err(io::Error::other(format!("unarc failed: {detail}")).into())
    }
}

fn parse_listing(output: &str) -> Result<Vec<ListedEntry>, Box<dyn Error>> {
    let mut entries = Vec::new();
    for line in output.lines() {
        let line = line.trim_end_matches('\r');
        if !has_timestamp_prefix(line) {
            continue;
        }
        if line.len() < 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Malformed FreeArc listing row",
            )
            .into());
        }

        let fields = &line[20..];
        let (is_dir, size, filename_offset) = if let Some(marker) = fields.find("-dir-") {
            (true, 0, 20 + marker + "-dir-".len() + 1)
        } else {
            let numeric = fields.trim_start();
            let leading_spaces = fields.len() - numeric.len();
            let digit_count = numeric.bytes().take_while(u8::is_ascii_digit).count();
            if digit_count == 0 || numeric.len() <= digit_count {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Malformed FreeArc listing row size field",
                )
                .into());
            }
            let size = numeric[..digit_count].parse::<u64>()?;
            (false, size, 20 + leading_spaces + digit_count + 1)
        };
        let path_start = line[filename_offset..].to_owned();
        if path_start.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "FreeArc listing row has no filename",
            )
            .into());
        }
        let display_path = path::normalize(&path_start.replace('\\', "/"));
        entries.push(ListedEntry {
            entry: ArchiveEntry {
                path: display_path,
                is_dir,
                size,
                encrypted: false,
            },
            archive_path: path_start,
        });
    }
    Ok(entries)
}

fn has_timestamp_prefix(line: &str) -> bool {
    let bytes = line.as_bytes();
    bytes.len() >= 31
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b' '
        && bytes[13] == b':'
        && bytes[16] == b':'
        && [0..4, 5..7, 8..10, 11..13, 14..16, 17..19]
            .iter()
            .all(|range| bytes[range.clone()].iter().all(u8::is_ascii_digit))
}

fn validate_relative_path(path: &str) -> Result<(), io::Error> {
    let normalized = path.replace('\\', "/");
    let path = Path::new(&normalized);
    let has_drive_prefix = normalized
        .as_bytes()
        .get(1)
        .is_some_and(|byte| *byte == b':');
    if has_drive_prefix
        || path.components().any(|component| {
            !matches!(component, Component::Normal(_) | Component::CurDir)
        })
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Refusing to extract a FreeArc entry with an unsafe path",
        ));
    }
    Ok(())
}

fn absolute_path(path: &Path) -> Result<PathBuf, io::Error> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_file_and_directory_rows_and_keeps_spaces_in_names() {
        let output = concat!(
            "UnArc 0.60RC unpacker\n",
            ". Listing archive: sample.arc\n",
            "Date/time                  Size Filename\n",
            "----------------------------------------\n",
            "2024-01-02 03:04:05         123 folder/my file.txt\n",
            "2024-01-02 03:04:05       -dir- folder\n",
            "----------------------------------------\n",
            "1 files, 123 bytes, 100 compressed\n",
        );

        let entries = parse_listing(output).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].entry.path, "folder/my file.txt");
        assert_eq!(entries[0].entry.size, 123);
        assert!(!entries[0].entry.is_dir);
        assert_eq!(entries[1].entry.path, "folder");
        assert!(entries[1].entry.is_dir);
    }

    #[test]
    fn rejects_unsafe_extraction_paths() {
        assert!(validate_relative_path("../outside.txt").is_err());
        assert!(validate_relative_path("folder/../../outside.txt").is_err());
        assert!(validate_relative_path(r"folder\..\outside.txt").is_err());
        assert!(validate_relative_path("/outside.txt").is_err());
        assert!(validate_relative_path("folder/readme.txt").is_ok());
    }
}
