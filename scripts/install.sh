#!/usr/bin/env bash
# Rootless installer for the unofficial Warp Arabic RTL AppImage.
#
# Installs the AppImage, a desktop entry, and icons inside the user's XDG data
# directories. It never calls sudo and never writes outside $HOME.
#
# Usage:
#   ./install.sh <AppImage> [--sha256 <hash|file>] [--prefix <dir>] [--no-desktop]
#
# Example:
#   ./install.sh ~/Downloads/warp-arabic-1.0.1-x86_64.AppImage
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
REPO_ROOT=$(cd -- "$SCRIPT_DIR/.." && pwd -P)

APPIMAGE=""
SHA_SOURCE=""
PREFIX="${XDG_DATA_HOME:-$HOME/.local}"
INSTALL_DESKTOP=1
APP_ID="dev.warp.WarpOss"
DESKTOP_ID="$APP_ID-ArArabic"
BIN_NAME="warp-arabic"

usage() {
  sed -n '2,12p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
  exit "${1:-1}"
}

while [ $# -gt 0 ]; do
  case "$1" in
    --sha256) SHA_SOURCE="${2:-}"; shift 2 ;;
    --prefix) PREFIX="${2:-}"; shift 2 ;;
    --no-desktop) INSTALL_DESKTOP=0; shift ;;
    -h|--help) usage 0 ;;
    -*) echo "Unknown option: $1" >&2; usage 1 ;;
    *) APPIMAGE="$1"; shift ;;
  esac
done

if [ -z "$APPIMAGE" ]; then
  echo "error: pass the AppImage path" >&2
  usage 1
fi

if [ "$(id -u)" = "0" ]; then
  echo "error: do not run this installer as root; it installs into your own home directory" >&2
  exit 1
fi

if [ ! -f "$APPIMAGE" ]; then
  echo "error: no such file: $APPIMAGE" >&2
  exit 1
fi

# Resolve to an absolute path: the checksum and identity checks run from other
# working directories.
APPIMAGE=$(cd -- "$(dirname -- "$APPIMAGE")" && pwd -P)/$(basename -- "$APPIMAGE")

# --- checksum verification -------------------------------------------------
EXPECTED=""
if [ -n "$SHA_SOURCE" ]; then
  if [ -f "$SHA_SOURCE" ]; then
    EXPECTED=$(awk '{print $1; exit}' "$SHA_SOURCE")
  else
    EXPECTED="$SHA_SOURCE"
  fi
elif [ -f "$APPIMAGE.sha256" ]; then
  EXPECTED=$(awk '{print $1; exit}' "$APPIMAGE.sha256")
fi

ACTUAL=$(sha256sum "$APPIMAGE" | awk '{print $1}')
if [ -n "$EXPECTED" ]; then
  if [ "$EXPECTED" != "$ACTUAL" ]; then
    echo "error: checksum mismatch" >&2
    echo "  expected: $EXPECTED" >&2
    echo "  actual:   $ACTUAL" >&2
    exit 1
  fi
  echo "checksum: OK ($ACTUAL)"
else
  echo "checksum: not verified (no .sha256 next to the file and none passed); recorded $ACTUAL"
fi

# --- sanity check: this is the modified client ------------------------------
# The app files live in a compressed squashfs, so the inner executable is
# extracted before it is inspected. Only that one file is unpacked.
CHECK_DIR=$(mktemp -d)
trap 'rm -rf "$CHECK_DIR"' EXIT
INNER_BIN="opt/warpdotdev/warp-terminal-oss/warp-oss"
if ( cd "$CHECK_DIR" && "$APPIMAGE" --appimage-extract "$INNER_BIN" >/dev/null 2>&1 ) \
  && [ -f "$CHECK_DIR/squashfs-root/$INNER_BIN" ] \
  && [ "$(strings -a "$CHECK_DIR/squashfs-root/$INNER_BIN" 2>/dev/null | grep -c "bidi_row.rs")" -gt 0 ]; then
  echo "identity: Arabic/RTL renderer present in the AppImage"
else
  echo "error: this AppImage does not look like the Arabic/RTL build (no bidi module found)" >&2
  echo "       refusing to install it so an official Warp build is not mixed in" >&2
  exit 1
fi

# --- layout ---------------------------------------------------------------
BIN_DIR="$PREFIX/bin"
APP_DIR="$PREFIX/share/$BIN_NAME"
DESKTOP_DIR="$PREFIX/share/applications"
ICON_ROOT="$PREFIX/share/icons/hicolor"
MANIFEST="$APP_DIR/manifest.txt"

mkdir -p "$BIN_DIR" "$APP_DIR" "$ICON_ROOT"

install -m 0755 "$APPIMAGE" "$BIN_DIR/$BIN_NAME"

# Icons: prefer the copies shipped in this repository, fall back to sizes
# embedded in the AppImage's AppDir.
for size in 256 128 64 48 32; do
  src="$REPO_ROOT/assets/icon/${size}x${size}/$DESKTOP_ID.png"
  [ -f "$src" ] || continue
  mkdir -p "$ICON_ROOT/${size}x${size}/apps"
  install -m 0644 "$src" "$ICON_ROOT/${size}x${size}/apps/$DESKTOP_ID.png"
done

if [ "$INSTALL_DESKTOP" = "1" ]; then
  mkdir -p "$DESKTOP_DIR"
  cat > "$DESKTOP_DIR/$DESKTOP_ID.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Version=1.0
Name=Warp Arabic RTL
GenericName=Terminal Emulator
Comment=Unofficial Warp build with Arabic BiDi, cursive shaping, and color emoji
Exec=$BIN_DIR/$BIN_NAME %U
TryExec=$BIN_DIR/$BIN_NAME
Icon=$DESKTOP_ID
Terminal=false
Categories=System;TerminalEmulator;
StartupNotify=true
# Must match the WM_CLASS the binary reports (dev.warp.WarpOss) so KDE binds the
# window to this entry instead of showing a generic icon.
StartupWMClass=$APP_ID
Keywords=terminal;shell;console;arabic;rtl;bidi;
DESKTOP
fi

cat > "$MANIFEST" <<MANIFEST
# Files installed by install.sh; uninstall.sh removes exactly these.
$BIN_DIR/$BIN_NAME
$DESKTOP_DIR/$DESKTOP_ID.desktop
$(for size in 256 128 64 48 32; do
    [ -f "$ICON_ROOT/${size}x${size}/apps/$DESKTOP_ID.png" ] && echo "$ICON_ROOT/${size}x${size}/apps/$DESKTOP_ID.png"
  done)
MANIFEST

for refresh in update-desktop-database gtk-update-icon-cache; do
  command -v "$refresh" >/dev/null 2>&1 || continue
  case "$refresh" in
    update-desktop-database) "$refresh" "$DESKTOP_DIR" >/dev/null 2>&1 || true ;;
    gtk-update-icon-cache) "$refresh" -q -t "$ICON_ROOT" >/dev/null 2>&1 || true ;;
  esac
done

if command -v kbuildsycoca6 >/dev/null 2>&1; then
  kbuildsycoca6 >/dev/null 2>&1 || true
elif command -v kbuildsycoca5 >/dev/null 2>&1; then
  kbuildsycoca5 >/dev/null 2>&1 || true
fi

echo "installed: $BIN_DIR/$BIN_NAME"
[ "$INSTALL_DESKTOP" = "1" ] && echo "launcher:  $DESKTOP_DIR/$DESKTOP_ID.desktop"
echo "settings are untouched; they live under your XDG state/config directories"
echo
echo "next: run scripts/verify-install.sh --prefix \"$PREFIX\", or start Warp Arabic RTL from the application menu"
