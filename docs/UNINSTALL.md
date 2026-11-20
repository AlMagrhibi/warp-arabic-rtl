# Uninstalling

This fork installs a single AppImage plus optional desktop/settings files. Nothing is
installed system-wide, so removal is a few deletes.

## 1. Remove the program

```bash
rm -f ~/.local/bin/warp-arabic
```

If you renamed it differently, remove that file instead. Nothing else on the system knows
about it.

## 2. Remove the desktop entry (if you added it)

```bash
rm -f ~/.local/share/applications/dev.warp.WarpOss-ArArabic.desktop
update-desktop-database ~/.local/share/applications 2>/dev/null || true
```

## 3. Remove settings and data (optional)

```bash
rm -rf ~/.config/warp-oss ~/.local/share/warp-oss
```

> Only do this if no other OSS-channel Warp build is using those directories. They are the
> upstream OSS-channel locations, shared with any other `warp-oss` build you may have.

## 4. Remove the build tree (only if you built from source)

```bash
rm -rf warp-arabic-rtl          # the clone
rm -rf ~/.local/bin/linuxdeploy-x86_64.AppImage   # downloaded by script/bundle
```

## What is *not* removed

Nothing else was installed: there is no RPM/DEB, no systemd unit, no timer, no repository
file, and no autostart entry. If you ever installed the official Warp package separately,
removing this fork does not touch it.
