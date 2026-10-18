# Whitecap

Whitecap is a GTK 4 and libadwaita archive browser for GNOME. It provides a
File Roller-inspired window for opening, browsing archive contents by folder,
searching the current folder, selecting, and extracting. Drop an archive onto
the window to open it.

Double-click a folder to browse into it; use the up button to return to its
parent. Press Ctrl+A while the contents list is focused to select all visible
entries. Drag selected entries from the list to a file manager for quick
extraction.

Supported formats:

- ZIP
- RAR4 and RAR5
- 7z
- ACE (`.ace`)
- FreeArc (`.arc`; requires the `unarc` executable on `PATH`)
- TAR
- TAR compressed with gzip, bzip2, xz, or zstd

When extracting from the Extract button, select one or more entries to extract
only those items. Leave the selection empty to extract the full archive.

## Build and run

Install Rust and the GTK 4/libadwaita development packages, then run:

```sh
cargo run
```

## Project layout

- `src/main.rs` starts the application.
- `src/ui.rs` builds the GTK/libadwaita interface and wires its actions.
- `src/archive/mod.rs` detects formats and dispatches archive operations.
- `src/archive/zip.rs`, `tar.rs`, `rar.rs`, `sevenz.rs`, `ace.rs`, and
  `freearc.rs` handle their respective formats.
- `src/archive/path.rs` contains shared archive-path and selection helpers.

FreeArc archives use the separately installed `unarc` extractor at runtime.
Whitecap does not bundle that executable; install a compatible FreeArc `unarc`
build, such as [xredor/unarc](https://github.com/xredor/unarc), and make sure
it is available on `PATH`.

ACE support uses the `unarc-rs` library. Password-protected and multi-volume
ACE archives are not supported, and individual entries are limited to 1 GiB.

## RAR library notice

RAR support uses `unrar-rs`, which includes the following UnRAR source-code
terms:

> UnRAR source code may be used in any software to handle RAR archives without
> limitations free of charge, but cannot be used to develop RAR (WinRAR)
> compatible archiver and to re-create RAR compression algorithm, which is
> proprietary. Distribution of modified UnRAR source code in separate form or
> as a part of other software is permitted, provided that full text of this
> paragraph, starting from "UnRAR source code" words, is included in license,
> or in documentation if license is not available, and in source code comments
> of resulting package.
