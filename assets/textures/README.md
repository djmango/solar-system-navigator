# Planet textures (8K)

High-resolution maps are **not stored in git** (too large). Download after clone:

```bash
./scripts/fetch_textures.sh
```

Default resolution is **8K** (8192×4096 equirectangular, same style as many KSP visual mods). For a lighter install:

```bash
SOLAR_TEXTURE_RES=2k ./scripts/fetch_textures.sh
```

## Source (recommended)

[Solar System Scope textures](https://www.solarsystemscope.com/textures/) — **CC-BY 4.0**, NASA-derived, free for education and commercial use with attribution. These are the same family of maps used in many planet visual packs.

| File | Body |
|------|------|
| `sun.jpg` | Sun |
| `earth.jpg` | Earth (day) |
| `mars.jpg` | Mars |
| `venus.jpg` | Venus |
| `mercury.jpg` | Mercury (optional) |
| `moon.jpg` | Moon (optional) |
| `jupiter.jpg` | Jupiter (optional) |
| `saturn.jpg` | Saturn (optional) |

## KSP mod textures (local only)

You **cannot** commit Squad or third-party KSP mod textures to this repo without permission. If you own KSP + texture mods (RSS, Spectra, etc.), import maps locally:

```bash
./scripts/import_ksp_textures.sh earth=/path/to/YourEarth8k.png mars=/path/to/YourMars8k.png
# or scan a folder:
KSP_TEXTURE_PACK_DIR=~/path/to/mod/8k ./scripts/import_ksp_textures.sh --scan
```

Stock KSP often ships **DDS** files — convert to PNG/JPG first.

## Scenario TOML

```toml
texture = "textures/earth.jpg"
```

If the file is missing, the body falls back to solid `color` from TOML.
