# Whitecap

Whitecap is a GTK 4 and libadwaita archive browser for GNOME. It provides a
File Roller-inspired window for opening, browsing archive contents by folder,
searching the current folder, selecting, and extracting. Drop an archive onto
the archive location bar to open it.

Double-click a folder to browse into it; use the up button to return to its
parent. Press Ctrl+A while the contents list is focused to select all visible
entries. Drag selected entries from the list to a file manager for quick
extraction. Delete selected entries with the toolbar's trash button or the
Delete key, then confirm the action. Selecting a folder removes its contents
recursively.

Supported formats

Formats that support editing and extraction:

- ZIP
- 7z
- TAR, including gzip, bzip2, xz, and zstd compressed TAR archives

Formats that support viewing and extraction only:

- RAR4 and RAR5
- ACE (`.ace`)
- FreeArc (`.arc`; requires the `unarc` executable on `PATH`)

The location bar prefixes read-only archives with a 🔒 padlock.

When extracting from the Extract button, select one or more entries to extract
only those items. Leave the selection empty to extract the full archive.

## Build and run

Install Rust and the GTK 4/libadwaita development packages, then run:

```sh
cargo run
```

## Linux packages

The GitHub Actions workflow builds `.deb` and `.rpm` packages on pushes, pull
requests, and manual runs. Successful builds on the default branch publish
prereleases with the `.deb` and `.rpm` attached as separate release assets;
pushing a `v*` tag publishes a versioned release. Pull requests and manual runs
build the packages without publishing a release.
The packages install Whitecap and its desktop launcher, and require GTK 4.10
and libadwaita 1.5 or newer. They also register Whitecap for all supported
archive MIME types in the file manager. FreeArc extraction additionally
requires the separate `unarc` executable on `PATH`.

When an archive is open, drop files or folders onto its contents to add them
to ZIP, 7z, or TAR archives (including compressed TAR variants). Dropped
folders keep their folder names and internal paths. Whitecap does not modify
RAR, ACE, or FreeArc archives; dropping items onto those archives displays a
warning. Deletion is also supported for ZIP, 7z, and TAR archives and their
compressed variants; RAR, ACE, and FreeArc remain read-only and display a
warning if deletion is attempted. Deleting entries rebuilds TAR archives and
compressed TAR archives, so large archives can take time and require temporary
disk space. Existing archive paths are not overwritten.
Adding to ZIP stages a copy of the archive; adding to TAR rebuilds it, and
adding to 7z recompresses its existing entries. Large archive updates can take
time and require additional temporary disk space.

Whitecap prompts for passwords when opening encrypted-header 7z or RAR
archives, and when extracting encrypted ZIP (ZipCrypto), 7z, RAR, or ACE
entries. A supplied password is kept in memory only while that archive is
open; encrypted archive creation is not supported. WinZip AES-encrypted ZIP
entries and password-protected FreeArc archives are not currently supported.

## Project layout

- `src/main.rs` starts the application.
- `src/ui/` contains the GTK/libadwaita window, separate open/archive actions,
  archive listing and row rendering, drag-and-drop handling, and shared UI
  state.
- `src/archive/mod.rs` exposes the archive API; `format.rs` detects formats
  and `operations.rs` dispatches archive operations.
- `src/archive/zip/`, `tar/`, `rar/`, `sevenz/`, `ace/`, and `freearc/` handle
  their respective formats.
- `src/archive/path.rs` contains shared archive-path and selection helpers.

FreeArc archives use the separately installed `unarc` extractor at runtime.
Whitecap does not bundle that executable; install a compatible FreeArc `unarc`
build, such as [xredor/unarc](https://github.com/xredor/unarc), and make sure
it is available on `PATH`.

ACE support uses the `unarc-rs` library. Multi-volume ACE archives are not
supported, and individual entries are limited to 1 GiB.

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
