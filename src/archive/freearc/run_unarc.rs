//! Executes `unarc` and normalizes process failures.

use std::{
    error::Error,
    io,
    process::{Command, Output},
};

/// Runs a FreeArc command and reports missing executables or failed status.
pub(crate) fn run_unarc(mut command: Command) -> Result<Output, Box<dyn Error>> {
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
