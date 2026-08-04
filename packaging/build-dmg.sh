#!/usr/bin/env bash
# Build PDF Opener.app and a drag-to-Applications DMG.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/dist"
APP="$DIST/PDF Opener.app"
VOL="PDF Opener"
STAGE="$DIST/dmg-stage"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

# Version: PDF_OPENER_VERSION env, else Cargo.toml, else 0.0.0
if [[ -n "${PDF_OPENER_VERSION:-}" ]]; then
  VERSION="$PDF_OPENER_VERSION"
else
  VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)"
  VERSION="${VERSION:-0.0.0}"
fi
# Strip leading v if present
VERSION="${VERSION#v}"
DMG="$DIST/PDF-Opener-${VERSION}.dmg"

echo "==> Version $VERSION"
echo "==> Building release binary"
(cd "$ROOT" && cargo build --release)

BIN="$CARGO_TARGET_DIR/release/pdf-opener"
if [[ ! -x "$BIN" ]]; then
  echo "Missing binary: $BIN" >&2
  exit 1
fi

# Ensure AppIcon.icns exists (CI / fresh clones)
ICNS="$ROOT/packaging/AppIcon.icns"
if [[ ! -f "$ICNS" ]]; then
  if [[ -f "$ROOT/dist/AppIcon.icns" ]]; then
    cp "$ROOT/dist/AppIcon.icns" "$ICNS"
  else
    echo "==> Generating AppIcon.icns"
    bash "$ROOT/packaging/make-icon.sh"
  fi
fi

echo "==> Assembling app bundle"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$ROOT/packaging/Info.plist" "$APP/Contents/Info.plist"
# Stamp version into Info.plist copy
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $VERSION" "$APP/Contents/Info.plist" 2>/dev/null \
  || true
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $VERSION" "$APP/Contents/Info.plist" 2>/dev/null \
  || true
cp "$BIN" "$APP/Contents/MacOS/pdf-opener"
chmod +x "$APP/Contents/MacOS/pdf-opener"
cp "$ICNS" "$APP/Contents/Resources/AppIcon.icns"

# Ad-hoc sign so Gatekeeper is less angry for local installs
codesign --force --deep --sign - "$APP" 2>/dev/null || true

echo "==> Creating DMG"
rm -rf "$STAGE" "$DMG"
mkdir -p "$STAGE"
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"

hdiutil create \
  -volname "$VOL" \
  -srcfolder "$STAGE" \
  -ov \
  -format UDZO \
  -imagekey zlib-level=9 \
  "$DMG" >/dev/null

rm -rf "$STAGE"

echo ""
echo "Done."
echo "  App: $APP"
echo "  DMG: $DMG"
echo ""
echo "Install: open the DMG and drag PDF Opener into Applications."
echo "Or:     open \"$APP\""
echo "First launch if blocked: right-click → Open (adhoc-signed local build)."
