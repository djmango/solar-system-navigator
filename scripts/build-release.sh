#!/usr/bin/env bash
# Fetch textures (if needed) and build a release binary with assets for distribution.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export SOLAR_TEXTURE_RES="${SOLAR_TEXTURE_RES:-8k}"

echo "Building release binary (textures auto-fetch via build.rs if needed)..."
cargo build --release

OUT="$ROOT/dist/solar-system-navigator"
rm -rf "$OUT"
mkdir -p "$OUT"
cp target/release/solar-system-navigator "$OUT/"
cp -r assets "$OUT/"
cp README.md LICENSE "$OUT/"

echo ""
echo "Release bundle ready: $OUT"
echo "Run from that directory:"
echo "  cd dist/solar-system-navigator && ./solar-system-navigator"
