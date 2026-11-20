# Changelog

All notable changes to this fork. Format follows [Keep a Changelog](https://keepachangelog.com/).
This project is a fork of [warpdotdev/warp](https://github.com/warpdotdev/warp); upstream
changes are not listed here.

## [1.0.1] — 2026-11-20

Desktop integration release. The terminal binary is unchanged from 1.0.0; this
release adds the packaging and verification layers around it.

### Added

- `scripts/install.sh`: rootless, per-user installer for the published AppImage.
  Verifies the SHA-256, proves the AppImage contains the Arabic renderer
  (`bidi_row.rs`), installs the binary, a desktop entry, and five icon sizes, and
  writes a manifest for clean removal. No sudo, no system package.
- `scripts/uninstall.sh`: removes exactly the files in the manifest, keeps
  settings unless `--purge-settings` is passed.
- `scripts/verify-install.sh`: checks the binary, the renderer inside it, the
  desktop entry (`StartupWMClass`, `Exec`, `desktop-file-validate`), the icon, and
  the running window class.
- `assets/icon/`: project icon (SVG + PNG 32–256) so the launcher and taskbar are
  distinguishable from the official Warp icon.
- `docs/ARCHITECTURE.md`: source-grounded description of the pipeline, BiDi path,
  shaping, glyph atlas, GPU renderer, image protocols, packaging and update
  isolation.
- `docs/IMAGE-PROTOCOLS.md`: verified matrix for Kitty graphics and iTerm2 inline
  images on the released AppImage.
- `docs/screenshots/06-kitty-inline.png`: both protocols rendering inline images.

### Fixed

- KDE had no launcher entry for the app, so the application menu had no icon and
  the task manager could not bind windows to a launcher. The installer now
  registers a desktop entry with `StartupWMClass=dev.warp.WarpOss` (the class the
  binary actually reports) plus icons.

### Verified

- Kitty graphics and iTerm2 inline images both render in the released AppImage.
- Installed launcher starts the app from `~/.local/bin/warp-arabic`; the running
  window class matches the desktop entry, and KDE shows the project icon in the
  task manager.

## [1.0.0] — 2026-11-20

First packaged release of the Arabic/RTL fork.

### Added

- `app/src/terminal/bidi_row.rs`: detection of rows containing RTL or cursive-joining text,
  and an implementation of the Unicode Bidirectional Algorithm reordering step (UAX #9, rule
  L2) over terminal grid cells, with unit tests covering pure RTL, mixed LTR/RTL, single-cell
  and ZWJ/emoji rows.
- `unicode-bidi` dependency (`app/Cargo.toml`).

### Changed

- `app/src/terminal/grid_renderer.rs`: rows that contain Arabic, Hebrew, Syriac, Thaana, N'Ko,
  Adlam or the bidi/joining control characters are routed through the whole-row shaping path
  even when ligatures are disabled, so letters are shaped in context and the alt-screen TUIs
  (vim, top, tmux) no longer break on such rows.
- Glyphs of those rows are painted at the shaper's pen positions instead of being snapped to
  monospace cell columns, which fixes RTL word order, Arabic joins and combining-mark
  stacking.
- Latin-only rows keep the original per-cell fast path unchanged.

### Fixed

- Arabic text rendering as isolated letters and/or mirrored word order in shell output,
  `vim`, `top` and box-drawing tables.
- Cursor placement at the end of an Arabic prompt.

### Verified

- `cargo test -p warp --lib terminal::bidi_row` — 5 passed.
- `cargo test -p warp --lib terminal::grid_renderer` — 15 passed, 1 ignored (macOS-only font
  assumption).
- Manual: Arabic/Hebrew shell output, mixed Arabic/English commands, harakat, box tables,
  `vim` on an Arabic file, `top` in the alt screen, Arabic prompt + cursor, byte-exact paste
  of Arabic into the terminal.
- Mouse-selected Arabic line copied with `Ctrl+Shift+C` gives back the exact logical text
  (`ARABICCOPY-مرحبا-بالعالم-END`, UTF-8 `d985 d8b1 …`).
- Packaged AppImage run from outside the build directory with an isolated `HOME` on Fedora 45 /
  KDE (XWayland): Arabic renders correctly, and `git`, `ssh`, `chmod` and clipboard work.

### Known limitations

- Selection uses logical grid columns, so a partial drag inside a mixed-direction line does
  not map to the mirrored visual glyphs; whole-line selection copies the exact text.
- Lines wider than the window are clipped, not wrapped (upstream behaviour).
- Linux x86_64 only.
