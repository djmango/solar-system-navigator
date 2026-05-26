# AGENTS.md

## Cursor Cloud specific instructions

### Project overview

Solar System Navigator is a Rust/Bevy 0.12 desktop GUI application that renders an interactive 2D orbital-mechanics simulation. There is no backend, no database, no Docker, and no external services. See `README.md` for more details.

### Build & run

- **Build:** `cargo build`
- **Run:** `DISPLAY=:99 cargo run` (requires a virtual framebuffer; see below)
- **Lint:** `cargo clippy` (4 warnings are expected in the current codebase, all non-error)
- **No test suite exists** in this codebase — there are no `#[test]` blocks or test files.

### Headless GUI rendering

Bevy requires a display server and GPU driver. In Cloud Agent VMs:

1. Start Xvfb if not already running: `Xvfb :99 -screen 0 1280x1024x24 &`
2. Run the app with `DISPLAY=:99 cargo run`. The Mesa llvmpipe software renderer provides OpenGL on the VM.
3. Set `XDG_RUNTIME_DIR=/tmp/xdg-runtime` (create the dir) if Vulkan/EGL tools complain.
4. ALSA audio warnings ("No audio device found") are expected and harmless.
5. The SSAO warning about `R16Float` is expected with software rendering and harmless.

### System dependencies (pre-installed via update script)

The following apt packages are required for Bevy 0.12 on Linux: `libudev-dev`, `libasound2-dev`, `libxkbcommon-x11-dev`, `libxkbcommon-dev`. These plus Mesa GPU drivers (`libgl1-mesa-dri`, `mesa-vulkan-drivers`) are installed by the update script.

### Taking screenshots of the running app

Use `scrot` (installed): `DISPLAY=:99 scrot /path/to/screenshot.png`
