# Solar System Navigator

A **3D** Rust application for visualizing and experimenting with celestial trajectories in our solar system. Built with [Bevy 0.18](https://bevy.org/), it loads multi-body scenarios from TOML, simulates N-body gravity with velocity Verlet integration, and provides an in-app editor for velocities and mission presets.

> **Units:** SI — meters, kilograms, seconds (`G = 6.67430×10⁻¹¹`). Orbits use circular, coplanar, mean-distance states (not a specific ephemeris date). For mission-grade ephemerides use [GMAT](https://gmat.sourceforge.io/) or STK. Planet **mesh size** is exaggerated via `visual_radius` so bodies remain visible at true orbital scale.

## Features

- **3D real-time visualization** — PBR spheres, orbit trails, orbit camera (pan/zoom/rotate)
- **Data-driven scenarios** — Add or edit bodies in `assets/**/*.toml` without recompiling
- **Mission presets** — Inner system, simplified Apollo 11, simplified OSIRIS-REx
- **Simulation controls** — Speed, substeps, pause, single-step, reset
- **Trajectory editor** — Select a body (Tab), adjust velocity with sliders
- **Probe spawning** — Launch a probe from the selected body with configurable Δv
- **Energy diagnostics** — Live kinetic/potential/total energy readout
- **KSP-style route planner** — Maneuver nodes (TNW Δv), predicted orbit paths, map mode
- **Starfield & improved materials** — Emissive sun/planets, optional atmosphere tint in TOML

## Controls

| Input | Action |
|-------|--------|
| Right-drag | Orbit camera |
| Middle-drag | Pan |
| Scroll / W/S | Zoom |
| Space | Pause / resume |
| N | Advance one step |
| R | Reload scenario |
| 1 / 2 / 3 | Switch preset |
| Tab | Cycle selected body |
| P | Spawn probe at selection |
| [ / ] | Decrease / increase probe Δv |
| +/- | Simulation speed |
| M | Toggle map mode (top-down system view) |
| Q / E | Rotate map view (in map mode) |
| B | Add maneuver node at current draft Δv |
| C | Clear maneuver nodes |
| V | Toggle orbit previews |
| O | Toggle automatic SOI central-body switching |
| H | Compute Hohmann transfer (sets prograde Δv to 1st burn) |
| Shift+H | Add Hohmann departure + arrival maneuver nodes |
| , / . | Decrease / increase time until next burn |

## Getting started

```bash
git clone https://github.com/djmango/solar-system-navigator.git
cd solar-system-navigator
chmod +x scripts/fetch_textures.sh scripts/build-release.sh
./scripts/fetch_textures.sh    # required once (JPEG/PNG maps are not in git)
cargo run --release
```

Release bundle (binary + `assets/`):

```bash
./scripts/build-release.sh
cd dist/solar-system-navigator && ./solar-system-navigator
```

## Project layout

```
assets/
  scenarios/default.toml    # Sun + inner planets (toy scale)
  missions/apollo11.toml
  missions/osiris_rex.toml
src/
  physics.rs      # N-body gravity + integrator (unit tested)
  orbit.rs        # Two-body Kepler previews + TNW frame
  maneuver.rs     # Burn execution at maneuver nodes
  map_view.rs     # Map mode camera + orbit gizmos
  planner.rs      # Sim clock + route planner sync
  scenario.rs     # TOML loader
  spawn.rs        # 3D entity spawning
  camera.rs       # Orbit camera
  ui/             # HUD + sliders
```

## Planet textures (8K)

After clone, download **8K** equirectangular maps (KSP-quality, CC-BY from Solar System Scope):

```bash
chmod +x scripts/fetch_textures.sh scripts/import_ksp_textures.sh
./scripts/fetch_textures.sh
```

Use `SOLAR_TEXTURE_RES=2k` for smaller downloads. To use **your own KSP mod textures** locally (not redistributed), see `scripts/import_ksp_textures.sh` and `assets/textures/README.md`.

Reference in TOML: `texture = "textures/earth.jpg"`. Missing files fall back to solid `color`.

## Recording a demo video

On Linux with Xvfb and ffmpeg:

```bash
xvfb-run -a ./scripts/record_demo.sh
# optional: SOLAR_DEMO_SECONDS=25 xvfb-run -a ./scripts/record_demo.sh
```

Output: `artifacts/demo.mp4` (duration matches `SOLAR_DEMO_SECONDS`, default 22s)

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
