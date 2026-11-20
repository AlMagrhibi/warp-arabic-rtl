# Warp Arabic / RTL v1.0.0

First packaged release of the unofficial Arabic/RTL fork of the open-source
[Warp](https://github.com/warpdotdev/warp) terminal.

## What it does

Rows that contain Arabic, Persian, Urdu, Hebrew, Syriac, Thaana, N'Ko, Adlam or Arabic
joining controls are now shaped as a whole line and painted at the shaper's pen positions,
and their cells are reordered with the Unicode Bidirectional Algorithm (UAX #9, rule L2).
Rows with only left-to-right text keep upstream's fast per-cell path.

Result: joined Arabic letters, correct right-to-left order, correct placement of Latin words,
numbers and punctuation inside RTL runs, aligned box-drawing tables, and working `vim`, `top`
and other alt-screen programs.

## Artifact

| File | Size | SHA-256 |
|---|---|---|
| `warp-arabic-1.0.0-x86_64.AppImage` | 203,495,928 bytes | `8504c59bddcac1dcc755d42db214c71acf056b88fa13486faa6a5f1b4dd7916f` |

```bash
chmod +x warp-arabic-1.0.0-x86_64.AppImage
./warp-arabic-1.0.0-x86_64.AppImage
```

Verify the download:

```bash
sha256sum -c warp-arabic-1.0.0-x86_64.AppImage.sha256
```

## Verification

Tested on Fedora 45 / KDE Plasma (X11 via XWayland), x86_64, launched from outside the build
directory with an isolated `HOME`:

- Arabic cursive joining and RTL order in shell output — pass
- Mixed Arabic + English, numbers, `%`, punctuation — pass
- Harakat, Hebrew — pass, no tofu
- Box-drawing tables aligned with Arabic cell content — pass
- `vim` with an Arabic string, `top` in the alt screen — pass
- Cursor placement after Arabic input — pass
- Paste Arabic into the terminal, bytes `d985 d8b1 …` intact — pass
- Mouse-select an Arabic line + `Ctrl+Shift+C` returns the exact logical text — pass
- Shell, `git 2.56.1`, `ssh`, file permissions inside the AppImage — pass
- `cargo test -p warp --lib terminal::bidi_row` — 5 passed
- `cargo test -p warp --lib terminal::grid_renderer` — 15 passed, 1 ignored (macOS-only test)

## Updates

This build cannot update itself. The `autoupdate` Cargo feature is not compiled in, and the
OSS channel in upstream returns "open-source channel binaries don't support autoupdate".
Updates happen only when you download a newer AppImage from this repository's releases.

## Known limitations

- Selection is stored in logical grid columns; a partial drag inside a mixed-direction line
  does not map to the mirrored visual glyphs. Whole-line selection copies the exact text.
- Lines wider than the window are clipped, not wrapped (upstream behaviour).
- Linux x86_64 only.

## License

Derived from Warp, licensed AGPL-3.0 (`LICENSE-AGPL`) and MIT (`LICENSE-MIT`). Upstream
notices are preserved. This is an unofficial fork, not affiliated with Warp.

---

## العربية — ملخص الإصدار

الإصدار الأول من نسخة مجتمعية غير رسمية من طرفية Warp مفتوحة المصدر تدعم العربية أصلاً:
تشكيل الحروف المتصلة، ترتيب الكتابة من اليمين إلى اليسار، ومواضع صحيحة للأرقام والكلمات
اللاتينية داخل النص العربي، مع بقاء الجداول الرسومية وبرامج الشاشة البديلة (`vim`, `top`)
تعمل بشكل سليم.

- الملف: `warp-arabic-1.0.0-x86_64.AppImage`
- البصمة: `8504c59bddcac1dcc755d42db214c71acf056b88fa13486faa6a5f1b4dd7916f`
- التشغيل: `chmod +x warp-arabic-1.0.0-x86_64.AppImage && ./warp-arabic-1.0.0-x86_64.AppImage`
- التحديثات: لا يوجد تحديث تلقائي إطلاقاً؛ التحديث يدوي من صفحة الإصدارات في هذا المستودع.
- القيود: تحديد جزء من سطر مختلط الاتجاه بالفأرة يعتمد على الأعمدة المنطقية؛ تحديد السطر
  كاملاً ينسخ النص الصحيح. الأسطر الأطول من النافذة تُقص ولا تُلف. لينكس x86_64 فقط.
