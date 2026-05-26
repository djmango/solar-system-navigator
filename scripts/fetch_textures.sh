#!/usr/bin/env bash
# Download 2K planet textures (Solar System Scope — see assets/textures/README.md).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/assets/textures"
BASE="https://www.solarsystemscope.com/textures/download"

mkdir -p "$OUT"

fetch() {
  local src="$1" dst="$2"
  echo "Fetching $dst ..."
  curl -fsSL "$BASE/$src" -o "$OUT/$dst"
}

fetch "2k_sun.jpg" "sun.jpg"
fetch "2k_earth_daymap.jpg" "earth.jpg"
fetch "2k_mars.jpg" "mars.jpg"
fetch "2k_venus_surface.jpg" "venus.jpg"

echo "Done. Textures in $OUT"
