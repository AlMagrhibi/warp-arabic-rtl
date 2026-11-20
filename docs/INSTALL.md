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

## One-step rootless install (recommended)

Download `warp-arabic-<version>-x86_64.AppImage` and its `.sha256` from the
[releases page](https://github.com/AlMagrhibi/warp-arabic-rtl/releases), then:

```bash
chmod +x warp-arabic-*.AppImage
./install.sh warp-arabic-1.0.1-x86_64.AppImage
```

`install.sh` (in this repository, also attached to each release) verifies the
checksum, proves the AppImage contains the Arabic renderer, and installs:

| What | Where |
| --- | --- |
| Application | `~/.local/bin/warp-arabic` |
| Launcher | `~/.local/share/applications/dev.warp.WarpOss-ArArabic.desktop` |
| Icons | `~/.local/share/icons/hicolor/{256,128,64,48,32}/apps/dev.warp.WarpOss-ArArabic.png` |
| Manifest | `~/.local/share/warp-arabic/manifest.txt` |

No `sudo`, no system package, no change to your existing settings. The desktop
entry sets `StartupWMClass=dev.warp.WarpOss`, which is the class the binary
reports, so KDE binds the running window to the launcher and shows the project
icon in the task manager.

Check the result:

```bash
scripts/verify-install.sh
```

Remove it again, keeping your settings:

```bash
scripts/uninstall.sh
```

`--purge-settings` also removes this app's own config/state directories.

## Manual launch without installing

```bash
chmod +x warp-arabic-1.0.1-x86_64.AppImage
./warp-arabic-1.0.1-x86_64.AppImage
```

Without a desktop entry KDE has nothing to bind the window to, which is why the
taskbar shows a generic icon in that case.
