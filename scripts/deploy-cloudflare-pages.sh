#!/usr/bin/env bash
# Build WASM and deploy dist/ via Wrangler (Workers static assets).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

./scripts/build-wasm.sh

if ! command -v wrangler >/dev/null 2>&1; then
  echo "Installing wrangler…"
  npm install -g wrangler
fi

echo "Deploying dist/ → Worker solar-system-navigator"
wrangler deploy

echo ""
echo "Set custom domain solar.skg.gg in Cloudflare → solar-system-navigator → Settings → Domains"
