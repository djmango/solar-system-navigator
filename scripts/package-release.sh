#!/usr/bin/env bash
# Build release bundle and pack into a distributable .tar.gz
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

ARCH="$(uname -m)"
case "$ARCH" in
  x86_64) ARCH_TAG="x86_64" ;;
  aarch64) ARCH_TAG="aarch64" ;;
  *) ARCH_TAG="$ARCH" ;;
esac

./scripts/build-release.sh

NAME="solar-system-navigator-linux-${ARCH_TAG}"
OUT_DIR="$ROOT/dist/solar-system-navigator"
ARCHIVE="$ROOT/dist/${NAME}.tar.gz"

rm -f "$ARCHIVE"
tar -C "$ROOT/dist" -czf "$ARCHIVE" solar-system-navigator

echo ""
echo "Archive: $ARCHIVE"
echo "Size: $(du -h "$ARCHIVE" | cut -f1)"
echo "Upload to /var/www/solar.skg.gg/releases/ or GitHub Releases"
