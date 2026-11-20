#!/usr/bin/env bash
# Removes files the installer created. Nothing outside the manifest is deleted.
#
# Usage:
#   ./uninstall.sh [--prefix <dir>] [--purge-settings]
#
# Settings (XDG config/state under ~/.config and ~/.local/state) are kept unless
# --purge-settings is passed, in which case only this app's own directories are
# removed.
set -euo pipefail

PREFIX="${XDG_DATA_HOME:-$HOME/.local}"
PURGE=0
APP_ID="dev.warp.WarpOss"
DESKTOP_ID="$APP_ID-ArArabic"
BIN_NAME="warp-arabic"

while [ $# -gt 0 ]; do
  case "$1" in
    --prefix) PREFIX="${2:-}"; shift 2 ;;
    --purge-settings) PURGE=1; shift ;;
    -h|--help) sed -n '2,10p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

APP_DIR="$PREFIX/share/$BIN_NAME"
MANIFEST="$APP_DIR/manifest.txt"

if [ ! -f "$MANIFEST" ]; then
  echo "nothing to do: no manifest at $MANIFEST"
  exit 0
fi

while IFS= read -r path; do
  case "$path" in
    ''|'#'*) continue ;;
  esac
  case "$path" in
    "$PREFIX"/*) ;;
    *) echo "skip (outside prefix): $path"; continue ;;
  esac
  if [ -e "$path" ]; then
    rm -f -- "$path"
    echo "removed: $path"
  fi
done < "$MANIFEST"

rm -f -- "$MANIFEST"
rmdir -- "$APP_DIR" 2>/dev/null || true

for size in 256 128 64 48 32; do
  rmdir -- "$PREFIX/share/icons/hicolor/${size}x${size}/apps" 2>/dev/null || true
done

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$PREFIX/share/applications" >/dev/null 2>&1 || true
fi
if command -v kbuildsycoca6 >/dev/null 2>&1; then
  kbuildsycoca6 >/dev/null 2>&1 || true
fi

if [ "$PURGE" = "1" ]; then
  for dir in \
    "${XDG_CONFIG_HOME:-$HOME/.config}/warp-terminal" \
    "$HOME/.local/state/warp-oss"; do
    [ -d "$dir" ] || continue
    echo "purging settings: $dir"
    rm -rf -- "$dir"
  done
else
  echo "settings kept; pass --purge-settings to remove them"
fi

echo "uninstalled"
