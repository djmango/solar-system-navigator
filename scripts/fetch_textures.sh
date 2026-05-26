#!/usr/bin/env bash
# Download planet textures from Solar System Scope (CC-BY 4.0).
# https://www.solarsystemscope.com/textures/
#
# Usage:
#   ./scripts/fetch_textures.sh              # default: 8k (KSP-like)
#   SOLAR_TEXTURE_RES=2k ./scripts/fetch_textures.sh   # smaller / faster CI dev
#   SOLAR_TEXTURE_RES=8k ./scripts/fetch_textures.sh   # recommended
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/assets/textures"
RES="${SOLAR_TEXTURE_RES:-8k}"
BASE="https://www.solarsystemscope.com/textures/download"

if [[ "$RES" != "2k" && "$RES" != "8k" ]]; then
  echo "error: SOLAR_TEXTURE_RES must be 2k or 8k (got: $RES)" >&2
  exit 1
fi

mkdir -p "$OUT"

fetch() {
  local src="$1" dst="$2"
  local url="$BASE/$src"
  local tmp="$OUT/.$dst.part"
  echo "Fetching $dst ($RES) ..."
  if curl -fsSL "$url" -o "$tmp"; then
    if file -b "$tmp" | grep -qiE 'jpeg|png|image'; then
      mv "$tmp" "$OUT/$dst"
      file -b "$OUT/$dst" | sed "s/^/  -> /"
    else
      rm -f "$tmp"
      echo "  !! skipped $dst (download was not an image — check $url)" >&2
      return 1
    fi
  else
    rm -f "$tmp"
    echo "  !! failed $dst" >&2
    return 1
  fi
}

failed=0

# Inner-system bodies in default.toml
fetch "${RES}_sun.jpg" sun.jpg || failed=$((failed + 1))
fetch "${RES}_earth_daymap.jpg" earth.jpg || failed=$((failed + 1))
fetch "${RES}_mars.jpg" mars.jpg || failed=$((failed + 1))
fetch "${RES}_venus_surface.jpg" venus.jpg || failed=$((failed + 1))

fetch "${RES}_mercury.jpg" mercury.jpg || failed=$((failed + 1))
fetch "${RES}_moon.jpg" moon.jpg || failed=$((failed + 1))

# Optional extras (for expanded scenarios)
fetch "${RES}_jupiter.jpg" jupiter.jpg || true
fetch "${RES}_saturn.jpg" saturn.jpg || true
fetch "${RES}_saturn_ring_alpha.png" saturn_ring.png || true
fetch "${RES}_stars_milky_way.jpg" stars_milky_way.jpg || true

echo ""
if [ "$failed" -eq 0 ]; then
  echo "Done. $RES textures in $OUT ($(du -sh "$OUT" | cut -f1))"
else
  echo "Completed with $failed required failure(s). See $OUT" >&2
  exit 1
fi
