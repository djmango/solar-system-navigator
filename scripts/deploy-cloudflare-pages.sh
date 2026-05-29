#!/usr/bin/env bash
# Deploy the static landing page to Cloudflare Pages (manual).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PROJECT="${CLOUDFLARE_PAGES_PROJECT:-solar}"

if ! command -v wrangler >/dev/null 2>&1; then
  echo "Installing wrangler…"
  npm install -g wrangler
fi

echo "Deploying deploy/landing → Cloudflare Pages project: $PROJECT"
wrangler pages deploy deploy/landing --project-name="$PROJECT"

echo ""
echo "Set custom domain solar.skg.gg in Cloudflare Pages → $PROJECT → Custom domains"
