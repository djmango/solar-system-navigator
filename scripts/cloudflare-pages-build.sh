#!/usr/bin/env bash
# Cloudflare Pages build command (set in dashboard or wrangler).
# Installs Rust + Trunk, fetches 2k textures, builds WASM into dist/.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export SOLAR_TEXTURE_RES="${SOLAR_TEXTURE_RES:-2k}"
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

if ! command -v trunk >/dev/null 2>&1; then
  echo "Installing trunk…"
  cargo install trunk --locked
fi

./scripts/fetch_textures.sh

unset NO_COLOR
trunk build --release --no-default-features --features web

echo "Pages build complete: dist/ ($(du -sh dist | cut -f1))"
