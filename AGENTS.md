# Whitecap Agent Guide

## Project overview

Whitecap is a GTK 4 and libadwaita archive browser for GNOME. It is a Rust
binary crate using edition 2024. The application has two main layers:

- `src/archive/` contains format detection, archive listing, extraction,
  password handling, path validation, and archive mutation.
- `src/ui/` contains GTK window construction, actions, archive browsing state,
  and drag-and-drop integration.

The application ID is `io.github.whitecap.Archive`. Keep that identifier
consistent across the desktop entry, icon filenames, MIME registration, and
GTK application setup.

## Important files

- `Cargo.toml` is the package manifest and source of the application version.
- `Cargo.lock` is committed and must remain synchronized with `Cargo.toml`.
- `README.md` documents user-facing features, supported formats, limitations,
  and build instructions.
- `packaging/` contains the desktop entry and archive MIME registration.
- `icons/` contains the packaged PNG and scalable SVG application icons.
- `.github/workflows/packages.yml` builds and publishes DEB and RPM packages.

## Development workflow

Use the locked dependency graph for normal development and CI:

```sh
cargo test --locked
cargo build --release --locked
```

Before submitting a change, also run formatting and lint checks when the Rust
toolchain components are installed:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

Do not regenerate `Cargo.lock` with a different dependency graph casually. If
the package version changes, update the root `whitecap` package entry in both
`Cargo.toml` and `Cargo.lock`. Dependency changes should be intentional and
should include the resulting lockfile update.

## Archive implementation rules

The public archive API is re-exported from `src/archive/mod.rs`; UI code should
use that API rather than depending directly on format-specific crates.

Supported formats are ZIP, TAR and compressed TAR, 7z, RAR, ACE, and FreeArc.
Only ZIP, 7z, and TAR support modification. RAR, ACE, and FreeArc must remain
read-only in the UI and archive layer.

Archive entry paths are untrusted input. Preserve these rules when changing
archive code:

- Normalize paths for display and selection comparisons.
- Validate extraction paths before writing to disk.
- Never allow `..`, absolute paths, drive-prefixed paths, or unsafe archive
  paths to escape the chosen extraction directory.
- Reject symbolic links and unsupported filesystem items when adding files.
- Do not overwrite existing archive entries when adding files.
- Use sibling temporary files for archive rewrites and persist them only after
  the replacement archive has been completely written.
- Preserve password error distinctions: missing passwords are `Required`, and
  rejected supplied passwords are `Invalid`.

Format-specific behavior belongs in its format module. Shared source
collection, collision checking, path handling, and rewrite-file behavior
belongs in the shared archive helpers.

## UI conventions

`AppState` is shared through `Rc<RefCell<_>>`. Keep borrows short, especially
before invoking GTK callbacks or rendering functions. Clone GTK objects when
moving them into callbacks, and avoid holding a mutable state borrow while
calling code that may access the same state.

The UI should report recoverable archive and dialog errors through the existing
toast helpers. Keep archive operations independent of GTK so they remain
testable.

## Packaging rules

The desktop entry uses `Icon=io.github.whitecap.Archive`. Package icons under
the standard hicolor paths for 32px, 64px, 128px, and scalable assets. When
changing packaging paths or metadata, update both the DEB and RPM sections of
`.github/workflows/packages.yml`.

The FreeArc implementation requires an external `unarc` executable on `PATH`;
it is not bundled into the application packages.

## Documentation and code style

Document public types and operations with rustdoc comments, especially when
they describe supported formats, selection semantics, security guarantees,
or error behavior. Keep `README.md` aligned with user-visible behavior and
limitations.

Prefer small, focused modules and helpers over large format-independent
functions. Avoid abstractions that obscure format-specific behavior. Preserve
existing error messages when they are user-facing unless the behavior is being
deliberately changed.

Do not commit build artifacts, generated package files, temporary archives, or
local environment configuration.
