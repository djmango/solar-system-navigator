# Planet textures

Bundled maps are **2K** downloads from [Solar System Scope textures](https://www.solarsystemscope.com/textures/) (free for educational/non-commercial use; credit the site in derivatives).

| File | Body |
|------|------|
| `sun.jpg` | Sun |
| `earth.jpg` | Earth |
| `mars.jpg` | Mars |
| `venus.jpg` | Venus |

Re-download:

```bash
./scripts/fetch_textures.sh
```

## Other sources (not bundled)

- **NASA Visible Earth / 3D Resources** — public-domain U.S. government imagery ([visibleearth.nasa.gov](https://visibleearth.nasa.gov/))
- **Celestia** — GPL texture packs (check license if redistributing)
- **KSP mods** — often **all-rights-reserved** or mod-specific licenses; do not commit Squad/third-party assets without permission. Use them locally only if the license allows.

Reference a texture in scenario TOML:

```toml
texture = "textures/earth.jpg"
```
