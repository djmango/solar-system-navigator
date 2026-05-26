# AGENTS.md

## Cursor Cloud specific instructions

### Project overview

Solar System Navigator is a Rust/Bevy 0.18 **3D** desktop application for N-body and patched-conics orbital mechanics (map mode, maneuver nodes, SOI, Hohmann helpers). There is no backend, database, or Docker. See `README.md` for controls and assets.

### Build & run

- **Build:** `cargo build` — `build.rs` auto-runs `scripts/fetch_textures.sh` when planet JPEGs are missing (gitignored). Use `SOLAR_TEXTURE_RES=2k` for faster CI-sized downloads or `SOLAR_SKIP_TEXTURE_FETCH=1` offline.
- **Run:** `DISPLAY=:99 cargo run --release` (virtual framebuffer in cloud VMs)
- **Lint:** `cargo fmt --all -- --check` and `cargo clippy --all-targets -- -D warnings`
- **Test:** `cargo test` (unit tests in `src/`)

### Headless GUI rendering

Bevy needs a display and GPU stack. In Cloud Agent VMs:

1. Start Xvfb if needed: `Xvfb :99 -screen 0 1280x1024x24 &`
2. Run with `DISPLAY=:99`. Mesa llvmpipe provides OpenGL.
3. Set `XDG_RUNTIME_DIR=/tmp/xdg-runtime` (create the dir) if Vulkan/EGL tools complain.
4. ALSA “No audio device” warnings are harmless.
5. SSAO / `R16Float` warnings with software rendering are harmless.

### System dependencies (Linux)

CI installs: `libasound2-dev`, `libudev-dev`, `libxkbcommon-dev`, `libwayland-dev`, `libvulkan-dev`, `pkg-config`. Mesa drivers help local/VM rendering.

### Screenshots

`DISPLAY=:99 scrot /path/to/screenshot.png` if `scrot` is installed.
