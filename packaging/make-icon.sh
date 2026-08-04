#!/usr/bin/env bash
# Build AppIcon.icns from assets/app-icon-master.png
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MASTER="${1:-$ROOT/assets/app-icon-master.png}"
OUT="${2:-$ROOT/packaging/AppIcon.icns}"
ICONSET="$(mktemp -d)/AppIcon.iconset"

if [[ ! -f "$MASTER" ]]; then
  echo "Missing master icon: $MASTER" >&2
  exit 1
fi

mkdir -p "$ICONSET" "$(dirname "$OUT")"

make_size() {
  local size="$1" name="$2"
  sips -z "$size" "$size" "$MASTER" --out "$ICONSET/$name" >/dev/null
}

make_size 16   icon_16x16.png
make_size 32   icon_16x16@2x.png
make_size 32   icon_32x32.png
make_size 64   icon_32x32@2x.png
make_size 128  icon_128x128.png
make_size 256  icon_128x128@2x.png
make_size 256  icon_256x256.png
make_size 512  icon_256x256@2x.png
make_size 512  icon_512x512.png
make_size 1024 icon_512x512@2x.png

iconutil -c icns "$ICONSET" -o "$OUT"
rm -rf "$(dirname "$ICONSET")"
echo "Wrote $OUT"
