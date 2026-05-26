#!/usr/bin/env bash
# Copy or link KSP / mod equirectangular textures into assets/textures/.
# Mod assets are NOT redistributed by this repo — you must own the game/mods.
#
# Usage (manual files):
#   ./scripts/import_ksp_textures.sh earth=/path/to/Earth_8k.png mars=/path/to/Mars_8k.png
#
# Usage (scan a mod folder for *earth*, *mars*, etc.):
#   KSP_TEXTURE_PACK_DIR=~/Games/KSP/GameData/SomeTextureMod/8k \
#     ./scripts/import_ksp_textures.sh --scan
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/assets/textures"
mkdir -p "$OUT"

copy_one() {
  local name="$1"
  local src="$2"
  if [ ! -f "$src" ]; then
    echo "missing: $src" >&2
    return 1
  fi
  local ext="${src##*.}"
  ext=$(echo "$ext" | tr '[:upper:]' '[:lower:]')
  case "$ext" in
    jpg | jpeg | png | webp) ;;
    dds)
      echo "  note: $src is DDS — convert to PNG/JPG first (e.g. ImageMagick, texconv)" >&2
      return 1
      ;;
    *)
      echo "  unsupported type: $src" >&2
      return 1
      ;;
  esac
  local dst="$OUT/${name}.${ext}"
  cp -f "$src" "$dst"
  echo "  $name -> $dst"
}

scan_pack() {
  local dir="${KSP_TEXTURE_PACK_DIR:?Set KSP_TEXTURE_PACK_DIR to your mod texture folder}"
  if [ ! -d "$dir" ]; then
    echo "directory not found: $dir" >&2
    exit 1
  fi
  echo "Scanning $dir ..."
  local body
  for body in sun earth mars venus mercury moon jupiter saturn; do
  local hit
    hit=$(find "$dir" -type f \( -iname "*${body}*" -o -iname "*${body^}*" \) \
      \( -iname "*.jpg" -o -iname "*.jpeg" -o -iname "*.png" -o -iname "*.webp" \) \
      2>/dev/null | head -n 1 || true)
    if [ -n "$hit" ]; then
      copy_one "$body" "$hit" || true
    fi
  done
}

if [ "${1:-}" = "--scan" ]; then
  scan_pack
  exit 0
fi

if [ "$#" -eq 0 ]; then
  cat <<'EOF'
Import KSP / mod planet textures (local use only).

Examples:
  ./scripts/import_ksp_textures.sh \
    earth=~/RSS-Textures/Earth/Color_8k.png \
    mars=~/RSS-Textures/Mars/Color_8k.png

  KSP_TEXTURE_PACK_DIR=~/my-8k-pack ./scripts/import_ksp_textures.sh --scan

Stock KSP often uses DDS under GameData/ — convert to PNG/JPG before import.
For hassle-free 8K maps, use: ./scripts/fetch_textures.sh
EOF
  exit 0
fi

for arg in "$@"; do
  if [[ "$arg" != *"="* ]]; then
    echo "bad arg (expected name=path): $arg" >&2
    exit 1
  fi
  name="${arg%%=*}"
  path="${arg#*=}"
  path="${path/#\~/$HOME}"
  copy_one "$name" "$path"
done

echo "Done. Point scenario TOML at textures/<name>.<ext>"
