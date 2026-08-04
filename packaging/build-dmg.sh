#!/usr/bin/env bash
# Build PDF Opener.app and a drag-to-Applications DMG.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/dist"
APP="$DIST/PDF Opener.app"
DMG="$DIST/PDF-Opener-0.1.0.dmg"
VOL="PDF Opener"
STAGE="$DIST/dmg-stage"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

echo "==> Building release binary"
(cd "$ROOT" && cargo build --release)

BIN="$CARGO_TARGET_DIR/release/pdf-opener"
if [[ ! -x "$BIN" ]]; then
  echo "Missing binary: $BIN" >&2
  exit 1
fi

echo "==> Assembling app bundle"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$ROOT/packaging/Info.plist" "$APP/Contents/Info.plist"
cp "$BIN" "$APP/Contents/MacOS/pdf-opener"
chmod +x "$APP/Contents/MacOS/pdf-opener"

if [[ -f "$ROOT/dist/AppIcon.icns" ]]; then
  cp "$ROOT/dist/AppIcon.icns" "$APP/Contents/Resources/AppIcon.icns"
elif [[ -f "$APP/Contents/Resources/AppIcon.icns" ]]; then
  :
else
  echo "Warning: AppIcon.icns missing — run packaging/make-icon.sh first" >&2
fi

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
