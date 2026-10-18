//! Shared ownership wrapper for application state.

use std::{cell::RefCell, rc::Rc};

use super::app_state::AppState;

/// Reference-counted, interior-mutable state passed to GTK callbacks.
pub(in crate::ui) type SharedState = Rc<RefCell<AppState>>;
