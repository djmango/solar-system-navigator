# Deploying Solar System Navigator

Native desktop (Bevy + Vulkan) and **browser** (WASM + WebGPU). For **https://solar.skg.gg**, the default is: **open the URL, the sim loads** — no separate marketing landing page.

| Goal | Approach |
|------|----------|
| Play in browser | Cloudflare Pages (Git) → Trunk `dist/` at `/` |
| Linux download | GitHub **`continuous`** release on every `master` push |
| Self-hosted desktop | Release tarball |

---

## Option A — Cloudflare Pages (connect repo) — recommended

Cloudflare builds and deploys on every push to `master`. No GitHub Actions secrets needed.

### One-time setup (Workers & Pages → Connect to Git)

Cloudflare runs **two steps** on each push: **build** then **deploy**. Fill in both.

1. [Cloudflare dashboard](https://dash.cloudflare.com) → **Workers & Pages** → **Create** → connect GitHub → **`djmango/solar-system-navigator`**
2. **Production branch:** `master`
3. **Project / Worker name:** `solar-system-navigator` (must match `name` in `wrangler.toml`)

| Setting | Value |
|---------|-------|
| **Build command** | `npm run build` |
| **Deploy command** | `npm run deploy` |
| **Root directory** (Advanced) | **Leave completely empty** — not `/`, not `dist` |

> **Common failure:** `Failed: root directory not found` means Root directory is set to a path that does not exist in git (often `dist`, which is build output and gitignored). Clear the field and save.

**Non-production branch deploy command** (previews): `npx wrangler deploy`

The build step writes WASM + assets to `dist/`; **Brotli compression** then replaces `*_bg.wasm` with `*.wasm.br` (typically ~10 MiB vs ~36 MiB raw) so Cloudflare’s **25 MiB per-file** upload limit is satisfied. The browser loads `.wasm.br` via `wasm-brotli-shim.js` (no feature cuts).

4. **Environment variables** (optional):

   | Name | Value |
   |------|-------|
   | `SOLAR_TEXTURE_RES` | `2k` |

5. **Save and Deploy** — first build may take 15–25 minutes.
6. **Custom domains** → add **`solar.skg.gg`**

Cloudflare **auto-builds on every push** to `master`. It does **not** auto-retry a failed build for the same commit — push again or click **Retry** in the dashboard / GitHub check run.

### What the build does

`scripts/cloudflare-pages-build.sh` installs Rust 1.95, Trunk, fetches 2k planet textures, and runs `trunk build` → `dist/` with the WASM game at `/`.

### Manual CLI deploy (optional)

```bash
./scripts/deploy-cloudflare-pages.sh   # build-wasm + wrangler pages deploy
```

Requires `npx wrangler login` once.

---

## Option B — Proxmox + Caddy (native download mirror)

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

## Option C — GitHub Releases

**Continuous (auto):** every `master` push → `.github/workflows/continuous-release.yml` publishes the Linux tarball to the **`continuous`** tag.

**Tagged:**

```bash
git tag v1.0.0 && git push origin v1.0.0
```

See `.github/workflows/release.yml` for semver releases.

---

## Local WASM dev

```bash
./scripts/build-wasm.sh
unset NO_COLOR
trunk serve --no-default-features --features web --open
```

**Notes**

- WebGPU required (Chrome/Edge 113+). Loading UI in `web/index.html` is the only pre-game screen.
- `_redirects` sends old `/app/` links to `/`.
- Scenarios embedded in WASM; textures bundled from `assets/` via Trunk.

---

## Native binary after download

```bash
tar xzf solar-system-navigator-linux-x86_64.tar.gz
cd solar-system-navigator
./solar-system-navigator
```

---

## Checklist for `solar.skg.gg`

1. [ ] Cloudflare Pages → Connect Git → repo + branch `master`
2. [ ] Build command `bash scripts/cloudflare-pages-build.sh`, output `dist`
3. [ ] Custom domain `solar.skg.gg`
4. [ ] Open `https://solar.skg.gg` in Chrome — sim loads at root
5. [ ] Linux download works from GitHub Releases (`continuous`)
