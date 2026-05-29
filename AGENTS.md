# AGENTS.md

## Cursor Cloud specific instructions

### Project overview

Solar System Navigator is a **web-first** orbital mechanics sim: Rust `sim-core` (f64 N-body physics), `sim-wasm` bindings, and a Vite/React/Three.js front end in `web/`. The legacy Bevy desktop app lives in `crates/navigator-legacy/` but is not built in CI. There is no backend, database, or Docker. See `README.md` for controls and assets.

### Build & run

- **Web build:** `bash scripts/cloudflare-pages-build.sh` (WASM + Vite → `dist/`)
- **Rust sim:** `cargo build -p sim-core -p sim-wasm`
- **Lint:** `cargo fmt --all -- --check` and `cargo clippy -p sim-core -p sim-wasm --all-targets -- -D warnings`
- **Test:** `cargo test -p sim-core -p sim-wasm`
- **Textures:** `scripts/fetch_textures.sh` when planet JPEGs are missing (gitignored). Use `SOLAR_TEXTURE_RES=2k` for faster downloads or `SOLAR_SKIP_TEXTURE_FETCH=1` offline.

### Legacy Bevy desktop (local only)

- **Build:** `cargo build --release -p navigator-legacy` (needs Linux GUI deps: ALSA, udev, xkbcommon, wayland, Vulkan)
- **Run:** `DISPLAY=:99 cargo run --release -p navigator-legacy` in cloud VMs with Xvfb + Mesa llvmpipe

### Screenshots

Browser: deploy preview or local `cd web && npm run dev`. Legacy desktop: `DISPLAY=:99 scrot /path/to/screenshot.png` if `scrot` is installed.
