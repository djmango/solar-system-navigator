#!/usr/bin/env bash
# Fetch textures (if needed) and build a release binary with assets for distribution.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

RES="${SOLAR_TEXTURE_RES:-8k}"
export SOLAR_TEXTURE_RES="$RES"

if [ ! -f assets/textures/earth.jpg ]; then
  echo "Downloading planet textures ($RES)..."
  ./scripts/fetch_textures.sh
else
  echo "Textures already present in assets/textures/ (set SOLAR_TEXTURE_RES to re-fetch)."
fi

echo "Building release binary..."
cargo build --release

OUT="$ROOT/dist/solar-system-navigator"
rm -rf "$OUT"
mkdir -p "$OUT"
cp target/release/solar-system-navigator "$OUT/"
cp -r assets "$OUT/"
cp README.md LICENSE-APACHE LICENSE-MIT "$OUT/" 2>/dev/null || true

echo ""
echo "Release bundle ready: $OUT"
echo "Run from that directory:"
echo "  cd dist/solar-system-navigator && ./solar-system-navigator"
