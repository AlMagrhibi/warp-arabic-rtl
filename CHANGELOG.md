# Changelog

All notable changes to this fork. Format follows [Keep a Changelog](https://keepachangelog.com/).
This project is a fork of [warpdotdev/warp](https://github.com/warpdotdev/warp); upstream
changes are not listed here.

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
