import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import fs from "node:fs";
import path from "node:path";
import { defineConfig, type Plugin } from "vite";
import topLevelAwait from "vite-plugin-top-level-await";
import wasm from "vite-plugin-wasm";

// Serve the repo `assets/` folder at `/assets` in dev, AND copy it into the
// build output so the deployed site has textures/scenarios without relying on a
// separate shell step. Keeps dev and production behavior identical.
function serveRepoAssets(): Plugin {
  const assetsRoot = path.resolve(__dirname, "../assets");
  const distAssets = path.resolve(__dirname, "../dist/assets");
  return {
    name: "serve-repo-assets",
    configureServer(server) {
      server.middlewares.use("/assets", (req, res, next) => {
        const urlPath = decodeURIComponent(req.url?.split("?")[0] ?? "");
        if (!urlPath || urlPath.includes("..")) return next();
        const filePath = path.join(assetsRoot, urlPath);
        if (!filePath.startsWith(assetsRoot) || !fs.existsSync(filePath)) return next();
        const ext = path.extname(filePath).toLowerCase();
        const types: Record<string, string> = {
          ".jpg": "image/jpeg",
          ".jpeg": "image/jpeg",
          ".png": "image/png",
          ".toml": "application/toml",
          ".svg": "image/svg+xml",
        };
        res.setHeader("Content-Type", types[ext] ?? "application/octet-stream");
        fs.createReadStream(filePath).pipe(res);
      });
    },
    closeBundle() {
      if (!fs.existsSync(assetsRoot)) return;
      for (const sub of ["textures", "scenarios", "missions"]) {
        const src = path.join(assetsRoot, sub);
        if (!fs.existsSync(src)) continue;
        fs.cpSync(src, path.join(distAssets, sub), { recursive: true });
      }
    },
  };
}

export default defineConfig({
  plugins: [react(), tailwindcss(), wasm(), topLevelAwait(), serveRepoAssets()],
  server: {
    fs: { allow: [".."] },
  },
  build: {
    outDir: "../dist",
    emptyOutDir: true,
    copyPublicDir: true,
  },
  publicDir: "public",
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "src"),
    },
  },
});
