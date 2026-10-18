#[path = "archive_actions.rs"]
mod archive_actions;
#[path = "open_actions.rs"]
mod open_actions;

pub(super) use archive_actions::connect_delete_action;
pub(super) use open_actions::{connect_extract_action, connect_open_action};
