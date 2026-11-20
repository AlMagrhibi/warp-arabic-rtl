# Warp Arabic / RTL

**Unofficial community fork of the open-source [Warp](https://github.com/warpdotdev/warp) terminal that adds native Arabic (and Hebrew/Syriac/Thaana) right-to-left rendering and Arabic cursive shaping to the grid renderer.**

> ⚠️ This is **not** an official Warp product. It is an independent fork built from the
> upstream open-source code base. Do not report bugs found here to Warp; report them in this
> repository. Upstream is licensed AGPL-3.0 / MIT and all upstream notices are preserved.

---

## Documentation

| Document | Contents |
| --- | --- |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Real implementation: pipeline, BiDi, shaping, glyph atlas, GPU, packaging |
| [docs/IMAGE-PROTOCOLS.md](docs/IMAGE-PROTOCOLS.md) | Kitty graphics and iTerm2 inline images: verified matrix and limits |
| [docs/INSTALL.md](docs/INSTALL.md) | Rootless one-step install, manual launch |
| [docs/BUILD.md](docs/BUILD.md) | Building the binary and the AppImage |
| [docs/UPDATE-POLICY.md](docs/UPDATE-POLICY.md) | Why official Warp updates are disabled |
| [docs/UNINSTALL.md](docs/UNINSTALL.md) | Removing the app, keeping or purging settings |
| [CHANGELOG.md](CHANGELOG.md) | Release history |

## The problem

Terminal cells are stored in *logical* order and every classic terminal draws them left to
right, one cell at a time. For Arabic that is wrong twice over:

1. **Direction** — RTL runs must be laid out right to left, so the visual column of a cell is
   not its grid column.
2. **Shaping** — Arabic letters change shape depending on their neighbours. Shaping a cell in
   isolation produces isolated letter forms and broken words.

The result in a stock Warp build is disconnected letters, reversed words, and garbled TUI
output (vim, top, tmux, `dialog`, progress bars) as soon as a row contains Arabic.

## What this fork changes

Two source files carry the behaviour, plus one module:

| Path | Change |
|---|---|
| `app/src/terminal/bidi_row.rs` | New module: row detection (`is_bidi_or_joining_char`, `row_needs_bidi`), an implementation of the Unicode Bidirectional Algorithm's reordering step (rule L2) over grid cells (`analyze_row`, `reorder_map`), and unit tests. |
| `app/src/terminal/grid_renderer.rs` | Rows containing RTL or cursive-joining characters are routed through the whole-row shaping path regardless of the user's ligature setting (`grid_rows_need_bidi_rendering`), and their glyphs are painted at the shaper's pen positions (`paint_line(..., use_shaped_positions)`) instead of being snapped to monospace cell columns. |
| `app/Cargo.toml` | Adds the `unicode-bidi` crate. |

Everything else is upstream Warp, unmodified.

### Why not just flip the ligature setting?

Warp already had a whole-row layout path for ligatures, but it was only used when the user
enabled ligatures, and it still snapped each glyph back to its cell column. RTL rows fall
apart under per-cell placement: the shaper emits right-to-left pen positions, so glyphs are
painted in the wrong order and Arabic joins break. This fork keeps non-BiDi rows on the
original fast per-cell path and only pays the shaping cost for rows that need it.

## Features

- Arabic, Persian, Urdu, Hebrew, Syriac, Thaana, N'Ko and Adlam text rendered joined and
  right-to-left in normal shell output, `less`, `vim`, `top`, and other TUIs.
- Mixed Arabic/English lines keep Latin words intact and in place (UAX #9 UBA ordering).
- Combining marks (harakat) stack on the correct base letter.
- Numbers, `%`, dates and punctuation inside RTL text follow the bidi algorithm.
- Box-drawing tables (`┌─┬─┐`) stay aligned with Arabic cell content.
- Cursor sits at the logical end of a typed RTL prompt; typing after Arabic text works.
- Pasting Arabic text into the terminal preserves the exact bytes.
- No change for rows that contain only LTR text — the original fast path is untouched.

## Known limitations

- Selection is stored in **logical** grid columns, so dragging across part of a mixed
  direction line selects the logical cells under the pointer rather than the mirrored visual
  glyphs. Selecting a whole line (triple click) copies the exact logical text, which is what
  was verified for this release.
- Lines wider than the window are clipped by the terminal viewport as usual; they are not
  wrapped, same as upstream.
- Bidirectional text in the *input line* (the prompt) is rendered by the same shaped path,
  but caret movement inside a mixed-direction input line may not match the logical cursor
  position in every case.
- Only Linux x86_64 is built and tested here.

## Install (AppImage)

```bash
# download from the latest release, then
chmod +x warp-arabic-1.0.0-x86_64.AppImage
./warp-arabic-1.0.0-x86_64.AppImage
```

Optional desktop entry:

```bash
mkdir -p ~/.local/bin ~/.local/share/applications
mv warp-arabic-*.AppImage ~/.local/bin/warp-arabic
chmod +x ~/.local/bin/warp-arabic
cp packaging/dev.warp.WarpOss-ArArabic.desktop ~/.local/share/applications/
```

See [docs/INSTALL.md](docs/INSTALL.md) for details, [docs/BUILD.md](docs/BUILD.md) to build
it yourself, [docs/UPDATE-POLICY.md](docs/UPDATE-POLICY.md) for the update policy, and
[docs/UNINSTALL.md](docs/UNINSTALL.md) to remove it.

## Build from source

```bash
git clone https://github.com/AlMagrhibi/warp-arabic-rtl.git
cd warp-arabic-rtl
cargo build --release --bin warp-oss --features release_bundle
./script/bundle -c oss --packages appimage --skip-build
```

Full prerequisites (Rust toolchain, `protoc`, Wayland/X11 dev headers, `lld`) are listed in
[docs/BUILD.md](docs/BUILD.md).

## Tests

```bash
cargo test -p warp --lib terminal::bidi_row          # 5 tests: ordering + detection
cargo test -p warp --lib terminal::grid_renderer     # 15 tests: renderer regressions
```

Both suites pass on the released revision. The renderer suite is upstream's and is run here
to prove the BiDi path did not break box drawing, selection maths or background bounds.

## Verification for v1.0.0

The published AppImage was tested from outside the build directory, in an isolated `HOME`,
on Fedora 45 / KDE Plasma with an X11 (`XWayland`) display:

| Check | Result |
|---|---|
| Arabic shell output, cursive joining, RTL order | pass |
| Mixed Arabic + English line keeps Latin words intact | pass |
| Numbers, `%`, punctuation inside RTL runs | pass |
| Harakat and Hebrew (no tofu, no missing glyphs) | pass |
| Box-drawing tables aligned with Arabic cells | pass |
| `vim` editing a file containing an Arabic string | pass |
| `top` in the alt screen | pass |
| Cursor after Arabic input, typing continues correctly | pass |
| Paste Arabic into the terminal — clipboard bytes `d985 d8b1 …` intact | pass |
| Mouse select an Arabic line + `Ctrl+Shift+C` — clipboard equals the logical text | pass |
| Shell, `git`, `ssh`, file permissions inside the AppImage environment | pass |
| `cargo test -p warp --lib terminal::bidi_row` | 5 passed |
| `cargo test -p warp --lib terminal::grid_renderer` | 15 passed, 1 ignored |

SHA-256 of `warp-arabic-1.0.0-x86_64.AppImage`:
`8504c59bddcac1dcc755d42db214c71acf056b88fa13486faa6a5f1b4dd7916f`

## Updates — nothing updates itself

This build **cannot** update itself and will never be silently replaced by an official Warp
release:

- The `autoupdate` Cargo feature is **not** compiled into this build
  (`FeatureFlag::Autoupdate` is `cfg(feature = "autoupdate")` only), so the update poll loop
  does not exist in the binary.
- The build runs on the **OSS channel**, and upstream explicitly refuses updates for it:
  `app/src/autoupdate/mod.rs` bails with *"Local, integration, and open-source channel
  binaries don't support autoupdate"* and treats the OSS release-asset URL as unreachable
  (`unreachable!("local/integration/oss autoupdate not supported")`).
- No RPM, systemd unit, timer or repository file is installed by this project, so no package
  manager can replace it either.

Updates, if any, are manual: download the new AppImage from *this* repository's GitHub
Releases and replace the old file. See [docs/UPDATE-POLICY.md](docs/UPDATE-POLICY.md).

## Repository history

This repository was started from the upstream master source tarball rather than from a git
clone, so the first commit (`chore: add complete upstream source snapshot`) contains the whole
upstream tree as a single snapshot. The fork's own changes sit on top of it as ordinary
commits; the upstream diff for this release is
`git diff 535c99ed..v1.0.0 -- app/src/terminal app/Cargo.toml`.

## Attribution

- Upstream project: <https://github.com/warpdotdev/warp> — AGPL-3.0 (`LICENSE-AGPL`) and MIT
  (`LICENSE-MIT`). Both licenses are kept in this repository and apply to the derived work.
- The BiDi integration is published under the same terms.
- Warp, the Warp logo and the Warp name are trademarks of Warp. This fork is unaffiliated and
  is distributed under its own name.

---

## العربية

**نسخة مجتمعية غير رسمية من طرفية Warp مفتوحة المصدر، تدعم العربية بشكل أصلي: تشكيل الحروف
واتجاه الكتابة من اليمين إلى اليسار.**

المشكلة في الطرفيات التقليدية أن كل خلية شبكية تُرسم من اليسار لليمين وبمعزل عن جاراتها، فيظهر
النص العربي مقطّع الحروف ومعكوس الترتيب. هذا المستودع يعدّل مُصيّر الشبكة في Warp بحيث:

- تُشكَّل الصفوف التي تحتوي حروفًا عربية/عبرية كسطر كامل (وليس خلية بخلية).
- تُرسم المحارف في المواضع التي يحددها محرك التشكيل بدل تصحيحها على أعمدة الخلايا.
- تبقى الصفوف اللاتينية على المسار السريع القديم دون أي تغيير.

**التركيب:**

```bash
chmod +x warp-terminal-arabic-*.AppImage
./warp-terminal-arabic-*.AppImage
```

**الاختبارات:**

```bash
cargo test -p warp --lib terminal::bidi_row
cargo test -p warp --lib terminal::grid_renderer
```

**التحديثات:** هذه النسخة **لا تحدّث نفسها أبدًا**. ميزة التحديث التلقائي غير مضمَّنة في
البناء أصلًا، وقناة OSS في Warp ترفض التحديث برسالة صريحة. أي تحديث يتم يدويًا من صفحة
الإصدارات في هذا المستودع فقط.

**ملاحظة:** هذه نسخة غير رسمية ولا علاقة لها بشركة Warp. رخصة المشروع الأصلي (AGPL-3.0/MIT)
محفوظة كما هي.
