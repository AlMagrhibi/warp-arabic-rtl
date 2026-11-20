# Installing the Arabic/RTL Warp AppImage

## Requirements

- Linux x86_64 with either X11 or Wayland.
- FUSE 2 for the AppImage runtime (`fuse-libs` on Fedora, `libfuse2` on Debian/Ubuntu).
  Without FUSE, run the AppImage with `--appimage-extract-and-run`.
- A GPU/driver that Warp can drive: Vulkan (`mesa-vulkan-drivers`) or OpenGL. On AMD/Intel
  Mesa is enough; on NVIDIA install the proprietary driver.

## Install

```bash
mkdir -p ~/.local/bin
cp warp-terminal-arabic-<version>-x86_64.AppImage ~/.local/bin/warp-arabic
chmod +x ~/.local/bin/warp-arabic
```

Check the checksum first — every release publishes a `.sha256` file:

```bash
sha256sum warp-terminal-arabic-<version>-x86_64.AppImage
# compare with the value in the release notes / .sha256 asset
```

Make sure `~/.local/bin` is on your `PATH`, then start it:

```bash
warp-arabic
```

## Desktop entry (optional)

```bash
mkdir -p ~/.local/share/applications
cp packaging/dev.warp.WarpOss-ArArabic.desktop ~/.local/share/applications/
update-desktop-database ~/.local/share/applications 2>/dev/null || true
```

## Data locations

The fork keeps its state in the same OSS-channel locations as upstream, so it is easy to
find and to remove:

| Purpose | Path |
|---|---|
| Settings and preferences | `~/.config/warp-oss/` |
| Application data | `~/.local/share/warp-oss/` |
| Config used by the AppImage | `~/.config/warp-oss/` |

## First run

Warp opens with its own onboarding on a fresh profile. If you already use another Warp build,
this fork does not read or write its settings.

## Troubleshooting

| Symptom | Fix |
|---|---|
| `dlopen(): error loading libfuse.so.2` | Install `fuse-libs` (Fedora) or `libfuse2` (Debian), or run with `--appimage-extract-and-run`. |
| Blank window / GPU error | Install Mesa Vulkan drivers (`mesa-vulkan-drivers`), or force another backend with `WGPU_BACKEND=gl`. |
| Arabic still looks disconnected | Make sure you launched `warp-arabic`, not another Warp build; check the About box for the fork version. |
| Font shows empty boxes for Arabic | Install an Arabic-capable font (e.g. `google-noto-sans-arabic-fonts`). |
