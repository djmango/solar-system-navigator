#!/usr/bin/env bash
# Cloudflare Workers build: Rust WASM + Vite/React → dist/
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.cargo}"
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "Installing Rust toolchain…"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.95
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

rustup toolchain install 1.95 --profile minimal 2>/dev/null || true
rustup default 1.95
rustup target add wasm32-unknown-unknown

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "Installing wasm-pack…"
  cargo install wasm-pack --locked
fi

# Optional textures for /assets/textures (solid colors used if missing).
CI= GITHUB_ACTIONS= ./scripts/fetch_textures.sh || true
for tex in assets/textures/*.jpg; do
  if [ -f "$tex" ] && [ "$(wc -c <"$tex")" -lt 4096 ]; then
    echo "Removing placeholder texture $(basename "$tex")"
    rm -f "$tex"
  fi
done

cd web
if [ ! -d node_modules ]; then
  npm ci
fi
npm run build
cd "$ROOT"

# Static assets + Cloudflare headers
mkdir -p dist/assets
cp -R assets/scenarios assets/missions dist/assets/ 2>/dev/null || true
if compgen -G "assets/textures/*.jpg" >/dev/null; then
  mkdir -p dist/assets/textures
  cp assets/textures/*.jpg dist/assets/textures/ 2>/dev/null || true
fi
cp deploy/cloudflare/_headers deploy/cloudflare/_redirects dist/ 2>/dev/null || true

echo "Build complete: dist/ ($(du -sh dist | cut -f1))"
