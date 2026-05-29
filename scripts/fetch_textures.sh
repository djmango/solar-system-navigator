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

# Minimal valid 1×1 JPEG — CI fallback when Solar System Scope blocks datacenter IPs.
write_placeholder_jpeg() {
  local dst="$1"
  base64 -d >"$OUT/$dst" <<'EOF'
/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAP//////////////////////////////////////////////////////////////////////////////////////2wBDAf//////////////////////////////////////////////////////////////////////////////////////wAARCAABAAEDAREAAhEBAxEB/8QAFAABAAAAAAAAAAAAAAAAAAAAB//EABQQAQAAAAAAAAAAAAAAAAAAAAD/xAAUAQEAAAAAAAAAAAAAAAAAAAAA/9oADAMBAAIRAxEAPwCwAA8A/9k=
EOF
}

is_image_file() {
  local f="$1"
  local magic
  magic=$(head -c 8 "$f" | od -An -tx1 | tr -d ' \n')
  [[ "$magic" == ffd8ff* ]] && return 0
  [[ "$magic" == 89504e470d0a1a0a* ]] && return 0
  return 1
}

fetch() {
  local src="$1" dst="$2"
  local url="$BASE/$src"
  local tmp="$OUT/.$dst.part"
  echo "Fetching $dst ($RES) ..."
  if curl -fsSL \
    -A "SolarSystemNavigator/1.0 (+https://github.com/djmango/solar-system-navigator)" \
    -H "Accept: image/jpeg,image/png,*/*" \
    --retry 2 --retry-delay 2 \
    "$url" -o "$tmp"; then
    if is_image_file "$tmp"; then
      mv "$tmp" "$OUT/$dst"
      echo "  -> $(wc -c <"$OUT/$dst") bytes"
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
  if [ "${CI:-}" = "true" ] || [ "${GITHUB_ACTIONS:-}" = "true" ]; then
    echo "CI: upstream texture CDN blocked or unavailable — writing placeholder JPEGs" >&2
    for name in sun.jpg earth.jpg mars.jpg venus.jpg mercury.jpg moon.jpg; do
      if [ ! -f "$OUT/$name" ]; then
        write_placeholder_jpeg "$name"
        echo "  placeholder $name"
      fi
    done
    echo "Done (CI placeholders). Run ./scripts/fetch_textures.sh locally for real maps."
    exit 0
  fi
  echo "Completed with $failed required failure(s). See $OUT" >&2
  exit 1
fi
