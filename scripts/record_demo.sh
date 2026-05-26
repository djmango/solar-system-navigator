#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

mkdir -p artifacts

cargo build --release

export SOLAR_DEMO_RECORD=1
export SOLAR_DEMO_FRAMES=900

./target/release/solar-system-navigator &
APP_PID=$!

cleanup() {
  kill "$APP_PID" 2>/dev/null || true
}
trap cleanup EXIT

sleep 6

ffmpeg -y \
  -f x11grab \
  -draw_mouse 0 \
  -video_size 1280x720 \
  -framerate 30 \
  -i "${DISPLAY}" \
  -t 20 \
  -crf 18 \
  artifacts/demo.mp4

echo "Wrote artifacts/demo.mp4"
