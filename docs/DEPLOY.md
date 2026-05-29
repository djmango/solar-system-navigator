# Deploying Solar System Navigator

Native desktop (Bevy + Vulkan) and **browser** (WASM + WebGPU). For **https://solar.skg.gg**, the default is: **open the URL, the sim loads** — no separate marketing landing page.

| Goal | Approach |
|------|----------|
| Play in browser | Trunk `dist/` at site root (Option D) |
| Linux download | GitHub Releases (`continuous` tag on master) or optional `deploy/landing/` mirror |
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

## Option B — Cloudflare Pages (CI deploy)

On every push to `master`, GitHub Actions (`.github/workflows/deploy.yml`):

1. Builds the Linux release tarball and publishes to the **`continuous`** GitHub Release
2. Builds the WASM bundle (`scripts/cloudflare-pages-build.sh`)
3. Deploys `dist/` to Cloudflare Pages project **`solar`**

**One-time setup:**

1. Cloudflare → **My Profile → API Tokens → Create Token** → template *Edit Cloudflare Workers* (includes Pages) or custom with **Account → Cloudflare Pages → Edit**.
2. Cloudflare dashboard → any zone → right sidebar **Account ID**.
3. GitHub repo → **Settings → Secrets and variables → Actions**:
   - `CLOUDFLARE_API_TOKEN` — token from step 1
   - `CLOUDFLARE_ACCOUNT_ID` — from step 2
4. Pages → project **solar** → **Custom domains** → add `solar.skg.gg` (DNS must be on Cloudflare for `skg.gg`).

Manual deploy:

```bash
chmod +x scripts/deploy-cloudflare-pages.sh
./scripts/deploy-cloudflare-pages.sh
```

Optional download-only mirror: use `deploy/landing/` as output instead of `dist/` (see `deploy/landing/README.md`).

---

## Option C — GitHub Releases (tagged)

```bash
git tag v1.0.0 && git push origin v1.0.0
```

See `.github/workflows/release.yml` for tagged release builds.

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

**Cloudflare Pages (CI):**

1. [ ] Add `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` to GitHub Actions secrets
2. [ ] Push to `master` (or run **Deploy** workflow manually)
3. [ ] Add custom domain `solar.skg.gg` on the Pages project
4. [ ] Open `https://solar.skg.gg` in Chrome — sim loads at root
5. [ ] Test Linux download from GitHub Releases

**Self-hosted (Proxmox + Caddy):**

1. [ ] Build: `./scripts/build-release.sh && ./scripts/package-release.sh`
2. [ ] Copy `deploy/landing/*` + tarball to `/var/www/solar.skg.gg/releases/`
3. [ ] Caddy or nginx + TLS
4. [ ] DNS `solar` → server or Cloudflare Tunnel
5. [ ] Test download link and run binary on a clean machine
