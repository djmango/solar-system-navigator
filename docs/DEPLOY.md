# Deploying Solar System Navigator

This project is a **native 3D desktop app** (Bevy + Vulkan/OpenGL). It does **not** run inside a browser today unless you port it to **WASM/WebGPU** (possible later, non-trivial).

For **https://solar.skg.gg** you typically combine:

| Goal | Approach |
|------|----------|
| Public info + downloads | Static site (Cloudflare Pages or Proxmox + Caddy) |
| Run the sim in the browser | WASM build (not shipped yet) |
| Run on your own PC | Linux/Windows release bundle |

---

## Option A — Proxmox + Caddy (recommended for `solar.skg.gg`)

Host a **static landing page** and a **Linux release tarball** on a VM/LXC in Proxmox. Point DNS at your server (or use Cloudflare Tunnel).

### 1. Build the release bundle (on a build machine or the VM)

```bash
git clone https://github.com/djmango/solar-system-navigator.git
cd solar-system-navigator
./scripts/build-release.sh          # builds + fills dist/solar-system-navigator/
./scripts/package-release.sh        # creates dist/solar-system-navigator-linux-x86_64.tar.gz
```

Needs: Rust 1.95, Linux dev libs (same as CI: `libasound2-dev`, `libudev-dev`, `libxkbcommon-dev`, `libwayland-dev`, `libvulkan-dev`), network for textures on first build.

### 2. Install on the web root

```bash
sudo mkdir -p /var/www/solar.skg.gg
sudo cp -r deploy/landing/* /var/www/solar.skg.gg/
sudo cp dist/solar-system-navigator-linux-x86_64.tar.gz /var/www/solar.skg.gg/releases/
sudo chown -R www-data:www-data /var/www/solar.skg.gg
```

### 3. Caddy (TLS via Let's Encrypt)

Copy `deploy/caddy/Caddyfile.example` to `/etc/caddy/Caddyfile` (or a snippet import):

```caddy
solar.skg.gg {
    root * /var/www/solar.skg.gg
    file_server
    encode gzip
}
```

```bash
sudo systemctl reload caddy
```

### 4. DNS

At your DNS provider (Cloudflare recommended):

| Type | Name | Content | Proxy |
|------|------|---------|-------|
| A | `solar` | Your public IP | Proxied (orange) or DNS only |
| AAAA | `solar` | IPv6 if you have it | Same |

If home IP changes, use **Cloudflare Tunnel** (`cloudflared`) from Proxmox instead of opening port 443.

### 5. Cloudflare Tunnel (no open ports)

On the Proxmox guest:

```bash
cloudflared tunnel create solar
cloudflared tunnel route dns solar solar.skg.gg
cloudflared tunnel run --url http://127.0.0.1:80 solar
```

Run Caddy on `:80` locally or serve files directly with `cloudflared tunnel --url file:///var/www/solar.skg.gg` (less common; Caddy + tunnel to localhost:443 is usual).

---

## Option B — Cloudflare Pages (landing only)

Good for a **fast global landing page**; the game itself is still a **download**, not in-browser.

1. Connect the GitHub repo to Cloudflare Pages.
2. **Build settings:**
   - Build command: `(none or echo ok)`
   - Build output directory: `deploy/landing`
3. Custom domain: `solar.skg.gg`
4. Upload the Linux `.tar.gz` to **GitHub Releases** or **R2** and link from `deploy/landing/index.html`.

Pages cannot execute the Rust binary; link to `/releases/...tar.gz` on R2 or GitHub.

### R2 bucket for binaries (optional)

1. Create R2 bucket `solar-releases`.
2. Upload `solar-system-navigator-linux-x86_64.tar.gz`.
3. Public custom domain e.g. `releases.skg.gg` or path on main site.
4. Update download URL in `deploy/landing/index.html`.

---

## Option C — GitHub Releases (CI)

Tag a release; GitHub Actions builds the Linux artifact (see `.github/workflows/release.yml`):

```bash
git tag v1.0.0
git push origin v1.0.0
```

Attach the tarball from Actions to the release; link from your landing page.

---

## Option D — In-browser at `solar.skg.gg` (future)

Requires:

- `wasm32-unknown-unknown` target
- Bevy web features + WebGPU (browser support varies)
- Trunk or `wasm-pack` pipeline
- Assets + textures bundled (no `build.rs` curl on WASM)
- Lower performance and different controls

Track as a separate milestone; not required for self-hosting downloads.

---

## What users run after download

```bash
tar xzf solar-system-navigator-linux-x86_64.tar.gz
cd solar-system-navigator
./solar-system-navigator
```

Linux needs Vulkan or Mesa (same as dev). First launch downloads planet textures if missing (or ship them inside the tarball after `./scripts/fetch_textures.sh` before packaging).

---

## Quick checklist for `solar.skg.gg`

1. [ ] Build: `./scripts/build-release.sh && ./scripts/package-release.sh`
2. [ ] Copy `deploy/landing/*` + tarball to `/var/www/solar.skg.gg`
3. [ ] Caddy or nginx + TLS
4. [ ] DNS `solar` → server or Cloudflare Tunnel
5. [ ] Test download link and run binary on a clean machine
