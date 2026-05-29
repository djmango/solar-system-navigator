#!/usr/bin/env bash
# Merge landing page + WASM app build into site/ for a single Pages deploy.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ ! -f dist/index.html ]] || ! compgen -G "dist/*.wasm" > /dev/null; then
  echo "Run ./scripts/build-wasm.sh first." >&2
  exit 1
fi

rm -rf site
mkdir -p site/app site/releases

cp -a deploy/landing/. site/
cp -a dist/. site/app/

if [[ -f dist/solar-system-navigator-linux-x86_64.tar.gz ]]; then
  cp dist/solar-system-navigator-linux-x86_64.tar.gz site/releases/
elif [[ -f dist/../solar-system-navigator-linux-x86_64.tar.gz ]]; then
  cp dist/../solar-system-navigator-linux-x86_64.tar.gz site/releases/ 2>/dev/null || true
fi

echo "Assembled site/ — deploy with: wrangler pages deploy site --project-name=solar"
