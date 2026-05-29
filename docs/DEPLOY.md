# Deploying Solar System Navigator

Native desktop (Bevy + Vulkan) and **browser** (WASM + WebGPU). For **https://solar.skg.gg**, the default is: **open the URL, the sim loads** — no separate marketing landing page.

| Goal | Approach |
|------|----------|
| Play in browser | Trunk `dist/` at site root (Option D) |
| Linux download | GitHub Releases or optional `deploy/landing/` mirror (Options A–C) |
| Self-hosted desktop | Release tarball |

---

## Option A — Proxmox + Caddy (native download mirror)

Optional static page in `deploy/landing/` plus a Linux `.tar.gz` — **not** required for the WASM site.

```bash
./scripts/build-release.sh
./scripts/package-release.sh
sudo mkdir -p /var/www/solar.skg.gg
sudo cp -r deploy/landing/* /var/www/solar.skg.gg/
sudo cp dist/solar-system-navigator-linux-x86_64.tar.gz /var/www/solar.skg.gg/releases/
```

See `deploy/caddy/Caddyfile.example` for TLS.

---

## Option B — Cloudflare Pages (download landing only)

If you only want a download page (no in-browser sim):

- Build output: `deploy/landing`
- Link the `.tar.gz` from GitHub Releases or R2

---

## Option C — GitHub Releases (CI)

```bash
git tag v1.0.0 && git push origin v1.0.0
```

---

## Option D — `solar.skg.gg` (WASM at `/`)

Visiting **/** serves `dist/index.html`: a short loading screen, then the Bevy app. In-app UI covers controls and scenarios.

**Build**

```bash
./scripts/build-wasm.sh
# → dist/index.html, *.wasm, assets/
```

**Local**

```bash
unset NO_COLOR
trunk serve --no-default-features --features web --open
```

**Cloudflare Pages (dashboard)**

1. Connect repo → branch `master`
2. **Build command:** `bash scripts/cloudflare-pages-build.sh`
3. **Output directory:** `dist`
4. Custom domain: `solar.skg.gg`
5. Env (optional): `SOLAR_TEXTURE_RES=2k`

**Wrangler CLI**

```bash
./scripts/build-wasm.sh
npx wrangler login
npx wrangler pages project create solar --production-branch master
npx wrangler pages deploy dist --project-name=solar
```

`wrangler.toml` sets `pages_build_output_dir = "dist"`.

**Notes**

- WebGPU required (Chrome/Edge 113+). Loading UI in `web/index.html` is the only pre-game screen.
- `_redirects` sends old `/app/` links to `/`.
- Scenarios embedded in WASM; textures from `assets/` via Trunk.
- First CI/Pages build may take 15–25 minutes.

---

## Native binary after download

```bash
tar xzf solar-system-navigator-linux-x86_64.tar.gz
cd solar-system-navigator
./solar-system-navigator
```

---

## Checklist for `solar.skg.gg`

1. [ ] `./scripts/build-wasm.sh`
2. [ ] `wrangler pages deploy dist --project-name=solar` (or Pages CI → `dist`)
3. [ ] DNS → Pages
4. [ ] Open `https://solar.skg.gg` in Chrome — sim loads at root
