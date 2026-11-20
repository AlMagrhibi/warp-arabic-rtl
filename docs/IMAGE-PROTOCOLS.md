# Inline image protocols

This document records what the Warp Arabic RTL build actually does with terminal
inline-image protocols, how it was verified, and what it deliberately does not
support. Every claim below was tested against an artifact, not inferred from
upstream documentation.

- Audit date: 2026-11-20
- Artifacts tested: `warp-arabic-1.0.0-x86_64.AppImage` (published release) and the
  development build of the same revision
- Platform: Fedora Linux 45, KDE Plasma, X11 session (XWayland), Vulkan backend
- Isolated `HOME` and `WARP_DATA_PROFILE` per test run

## Result matrix

| Protocol | Case | Result | Evidence |
| --- | --- | --- | --- |
| Kitty graphics (APC `ESC_G`) | Transmit and display a PNG (`f=100,a=T,i=1`) | **PASS** | image rendered in the grid, screenshot `docs/screenshots/06-kitty-inline.png` |
| Kitty graphics | Reply to the client | **PASS (by design)** | `ESC_Gi=1;OK ESC\` is written to the pty; a shell that is not reading commands shows it as stray input |
| Kitty graphics | Chunked payload (`m=1` … `m=0`) | **NOT TESTED** | code path `parse_kitty_chunk` + `further_chunks` in `crates/warp_terminal/src/model/ansi/mod.rs` |
| Kitty graphics | Delete actions (`a=d`) | **NOT TESTED** | `KittyAction::Delete` exists in `crates/warp_terminal/src/model/kitty.rs` |
| Kitty graphics | Animations, Unicode placeholders, image numbers | **UNSUPPORTED** | no parser branches for them in `crates/warp_terminal/src/model/kitty.rs`, so they are ignored rather than faked |
| iTerm2 inline images (OSC 1337 `File=`) | `inline=1` with base64 PNG | **PASS** | image rendered below the prompt, same screenshot, second image |
| iTerm2 | `inline=0` (download hint) and width/height units | **NOT TESTED** | `crates/warp_terminal/src/model/iterm_image.rs` parses `width`, `height`, `inline`, `preserveAspectRatio`, `doNotMoveCursor` |
| Capability query | DA1 / Kitty query advertisement | **NOT TESTED** | see "Open questions" below |

Rendering is what was verified. Terminal-protocol *detection* (whether an
application can discover the support before trying) is a separate question and is
not claimed.

## Source map

| Concern | File |
| --- | --- |
| OSC/APC dispatch, feature gating | `crates/warp_terminal/src/model/ansi/mod.rs` (`apc_end`, `b"1337"` branch) |
| Kitty control data, chunking, action parsing | `crates/warp_terminal/src/model/kitty.rs` |
| iTerm2 metadata parsing | `crates/warp_terminal/src/model/iterm_image.rs` |
| Image storage, ids, placements | `crates/warp_terminal/src/model/image_map.rs` |
| App-side handling and replies | `app/src/terminal/model/terminal_model.rs` (`end_kitty_action_receiving`, `handle_completed_iterm_image`) |
| Grid placement and drawing | `app/src/terminal/grid_renderer.rs`, `crates/warp_terminal/src/model/blockgrid.rs` |
| Compile-time switch | cargo feature `kitty_images` (enabled by default, `app/Cargo.toml`) |

Both protocols are upstream code paths. This fork did not add or replace an image
renderer; it only verified them and documented the results.

## Reproducing the tests

```bash
# Kitty graphics: one PNG, transmit and display
python3 kitty-test.py kitty-test.png

# iTerm2: OSC 1337 with a base64 payload
./iterm-test.sh kitty-test.png
```

`kitty-test.py`:

```python
import base64, sys
data = base64.b64encode(open(sys.argv[1], 'rb').read()).decode()
sys.stdout.write("\x1b_Gf=100,a=T,i=1,m=0;" + data + "\x1b\\\n")
```

`iterm-test.sh`:

```bash
#!/bin/bash
img="$1"
test -r "$img" && printf '\033]1337;File=inline=1:%s\a\n' "$(base64 -w0 "$img")"
```

## Constraints and behaviour

- The Kitty reply is sent to the pty, so a shell that is not expecting it shows
  `^[_Gi=1;OK^[\`. That is protocol-correct; a program that issues the graphics
  command reads the reply instead.
- Images are placed in the terminal grid and scroll with their block. Terminals
  in this build are not asked to keep image pixels alive after the block leaves
  scrollback; data is dropped with the block.
- Malformed payloads are rejected by the parser and logged; they do not crash the
  terminal. No fuzzing campaign has been run, so this is a "does not crash on the
  cases tried" statement, not a proof.
- Feature flags can compile both protocols out. If a future build disables
  `kitty_images`, the APC handler returns early and images are silently ignored.

## Open questions

- Whether `DA1` replies advertise graphics support in this revision. Not
  inspected yet; applications that probe first may conclude "no graphics".
- Chunked transfers larger than one APC and delete semantics are implemented in
  source but untested in the GUI.

## Relationship to the rest of the roadmap

Inline image protocols are unrelated to color emoji: emoji are font glyphs in
grid cells, images are protocol payloads placed over cells. Neither feature
depends on the other.
