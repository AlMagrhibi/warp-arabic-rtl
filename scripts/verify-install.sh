#!/usr/bin/env bash
# Checks that the rootless installation is complete and that KDE can bind the
# running window to the installed launcher.
#
# Usage:
#   ./verify-install.sh [--prefix <dir>]
#
# Exit code is 0 only when every check passes.
set -uo pipefail

PREFIX="${XDG_DATA_HOME:-$HOME/.local}"
APP_ID="dev.warp.WarpOss"
DESKTOP_ID="$APP_ID-ArArabic"
BIN_NAME="warp-arabic"
FAILURES=0

while [ $# -gt 0 ]; do
  case "$1" in
    --prefix) PREFIX="${2:-}"; shift 2 ;;
    -h|--help) sed -n '2,10p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

pass() { printf 'PASS  %s\n' "$1"; }
fail() { printf 'FAIL  %s\n' "$1"; FAILURES=$((FAILURES + 1)); }
skip() { printf 'SKIP  %s\n' "$1"; }

BIN="$PREFIX/bin/$BIN_NAME"
DESKTOP="$PREFIX/share/applications/$DESKTOP_ID.desktop"
ICON="$PREFIX/share/icons/hicolor/256x256/apps/$DESKTOP_ID.png"

# 1. binary
if [ -x "$BIN" ]; then
  pass "executable present: $BIN"
else
  fail "executable missing or not executable: $BIN"
fi

# 2. the installed binary is the Arabic build. The installed file is a
# compressed AppImage, so the inner executable is extracted before inspection.
if [ -x "$BIN" ]; then
  CHECK_DIR=$(mktemp -d)
  INNER_BIN="opt/warpdotdev/warp-terminal-oss/warp-oss"
  if ( cd "$CHECK_DIR" && "$BIN" --appimage-extract "$INNER_BIN" >/dev/null 2>&1 ) \
    && [ "$(strings -a "$CHECK_DIR/squashfs-root/$INNER_BIN" 2>/dev/null | grep -c "bidi_row.rs")" -gt 0 ]; then
    pass "installed AppImage contains the Arabic BiDi renderer (bidi_row.rs)"
  else
    fail "installed AppImage does not contain the Arabic BiDi renderer"
  fi
  rm -rf "$CHECK_DIR"
fi

# 3. desktop entry
if [ -f "$DESKTOP" ]; then
  pass "desktop entry present: $DESKTOP"
  if grep -q "^StartupWMClass=$APP_ID$" "$DESKTOP"; then
    pass "desktop entry binds StartupWMClass=$APP_ID"
  else
    fail "desktop entry is missing StartupWMClass=$APP_ID"
  fi
  exec_path=$(sed -n 's/^Exec=\([^ ]*\).*/\1/p' "$DESKTOP" | head -1)
  if [ -n "$exec_path" ] && [ -x "$exec_path" ]; then
    pass "Exec target is executable: $exec_path"
  else
    fail "Exec target is missing: ${exec_path:-<none>}"
  fi
  if command -v desktop-file-validate >/dev/null 2>&1; then
    if desktop-file-validate "$DESKTOP" 2>/dev/null; then
      pass "desktop-file-validate"
    else
      fail "desktop-file-validate reported problems"
    fi
  else
    skip "desktop-file-validate not installed"
  fi
else
  fail "desktop entry missing: $DESKTOP"
fi

# 4. icon
if [ -f "$ICON" ]; then
  pass "icon present: $ICON"
else
  fail "icon missing: $ICON"
fi

# 5. desktop database sees the entry
if command -v desktop-file-validate >/dev/null 2>&1 || [ -f "$PREFIX/share/applications/mimeinfo.cache" ]; then
  if grep -qs "$DESKTOP_ID" "$PREFIX/share/applications/mimeinfo.cache" 2>/dev/null; then
    pass "desktop database references $DESKTOP_ID"
  else
    skip "desktop database does not reference the entry (harmless; KDE reads the file directly)"
  fi
fi

# 6. running window identity, when a session is available
if command -v xdotool >/dev/null 2>&1 && [ -n "${DISPLAY:-}" ]; then
  window_id=$(xdotool search --class "$APP_ID" 2>/dev/null | tail -1 || true)
  if [ -n "$window_id" ]; then
    class=$(xdotool getwindowclassname "$window_id" 2>/dev/null || true)
    if [ "$class" = "$APP_ID" ]; then
      pass "running window class is $APP_ID (matches StartupWMClass)"
    else
      fail "running window class is '${class:-unknown}', expected $APP_ID"
    fi
  else
    skip "no running Warp Arabic RTL window; start it and rerun to check taskbar binding"
  fi
else
  skip "DISPLAY or xdotool unavailable; skipped window-class check"
fi

echo
if [ "$FAILURES" -eq 0 ]; then
  echo "verify-install: OK"
  exit 0
fi
echo "verify-install: $FAILURES check(s) failed"
exit 1
