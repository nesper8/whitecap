//! Re-exports the action wiring used by the main window.

#[path = "archive_actions.rs"]
mod archive_actions;
#[path = "open_actions/mod.rs"]
mod open_actions;

pub(super) use archive_actions::connect_delete_action;
pub(super) use open_actions::{connect_extract_action, connect_open_action};
