// RAR library notice required by unrar-rs:
// UnRAR source code may be used in any software to handle RAR archives without limitations free of charge, but cannot be used to develop RAR (WinRAR) compatible archiver and to re-create RAR compression algorithm, which is proprietary. Distribution of modified UnRAR source code in separate form or as a part of other software is permitted, provided that full text of this paragraph, starting from "UnRAR source code" words, is included in license, or in documentation if license is not available, and in source code comments of resulting package.
//! Read-only RAR archive support.
//!
//! RAR archives can be listed and extracted, but Whitecap intentionally does
//! not create or rewrite them.

mod extract;
mod load_with_password;
mod map_password_error;

pub(super) use extract::extract;
pub(super) use load_with_password::load_with_password;
