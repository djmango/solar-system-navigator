#!/usr/bin/env bash
# Build browser WASM + WebGPU bundle into dist/ (Trunk).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export SOLAR_TEXTURE_RES="${SOLAR_TEXTURE_RES:-2k}"

if ! compgen -G "assets/textures/*.jpg" > /dev/null; then
  echo "Fetching 2k planet textures for WASM asset bundle…"
  ./scripts/fetch_textures.sh
fi

if ! command -v trunk >/dev/null 2>&1; then
  echo "Installing trunk…"
  cargo install trunk --locked
fi

rustup target add wasm32-unknown-unknown 2>/dev/null || true

echo "Building WASM (profile wasm-release, features web + WebGPU)…"
trunk build --release --no-default-features --features web

node scripts/compress-wasm-brotli.mjs dist

echo "Done. Open dist/index.html via trunk serve, or: npx wrangler deploy"
