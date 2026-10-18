//! Archive-open and extraction action wiring.

mod connect_extract_action;
mod connect_open_action;
mod extract_selected;
mod request_extract_password;

pub(crate) use connect_extract_action::connect_extract_action;
pub(crate) use connect_open_action::connect_open_action;
