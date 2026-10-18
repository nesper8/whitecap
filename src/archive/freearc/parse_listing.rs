//! Parses structured entry rows emitted by FreeArc listings.

use std::{error::Error, io};

use crate::archive::{ArchiveEntry, path};

use super::{entry::ListedEntry, has_timestamp_prefix::has_timestamp_prefix};

/// Parses files and directories from `unarc l` output.
pub(crate) fn parse_listing(output: &str) -> Result<Vec<ListedEntry>, Box<dyn Error>> {
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
