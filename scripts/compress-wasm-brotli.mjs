#!/usr/bin/env node
/**
 * Brotli-compress Trunk *_bg.wasm for Cloudflare (25 MiB per-file upload limit).
 * Removes raw .wasm, patches JS/HTML to load .wasm.br via wasm-brotli-shim.js.
 */
import { brotliCompressSync, constants } from 'node:zlib';
import { execFileSync } from 'node:child_process';
import {
  readFileSync,
  writeFileSync,
  unlinkSync,
  readdirSync,
  copyFileSync,
  statSync,
} from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const DIST = process.argv[2] ? join(process.cwd(), process.argv[2]) : join(ROOT, 'dist');
const MAX_BYTES = 25 * 1024 * 1024;

function formatMiB(bytes) {
  return `${(bytes / (1024 * 1024)).toFixed(2)} MiB`;
}

function bundleBrotliDecoder() {
  const entry = join(ROOT, 'scripts/brotli-decode-entry.cjs');
  const out = join(DIST, 'brotli-decode.mjs');
  execFileSync(
    'npx',
    [
      'esbuild',
      entry,
      '--bundle',
      '--platform=browser',
      '--format=esm',
      `--outfile=${out}`,
    ],
    { stdio: 'inherit', cwd: ROOT },
  );
  console.log(`Bundled brotli decoder → ${out} (${formatMiB(statSync(out).size)})`);
}

function patchJsFiles(wasmBaseName) {
  const jsFiles = readdirSync(DIST).filter((f) => f.endsWith('.js'));
  for (const file of jsFiles) {
    const path = join(DIST, file);
    let js = readFileSync(path, 'utf8');
    if (!js.includes('_bg.wasm')) {
      continue;
    }
    js = js.replace(/_bg\.wasm(?!\.br)/g, '_bg.wasm.br');
    writeFileSync(path, js);
    console.log(`Patched ${file} → _bg.wasm.br`);
  }
}

function patchIndexHtml() {
  const indexPath = join(DIST, 'index.html');
  let html = readFileSync(indexPath, 'utf8');
  let changed = false;

  const patchedWasmRefs = html.replace(/_bg\.wasm(?!\.br)/g, '_bg.wasm.br');
  if (patchedWasmRefs !== html) {
    html = patchedWasmRefs;
    changed = true;
    console.log('Patched index.html → _bg.wasm.br');
  }

  // Post-Trunk patches change JS/wasm bytes; SRI hashes in index.html would block loads.
  const withoutIntegrity = html.replace(/\s+integrity="[^"]*"/g, '');
  if (withoutIntegrity !== html) {
    html = withoutIntegrity;
    changed = true;
    console.log('Removed stale integrity attributes from index.html');
  }

  const shimTag =
    '<script type="module" src="./wasm-brotli-shim.js"></script>';
  if (!html.includes('wasm-brotli-shim.js')) {
    if (html.includes('type="module"')) {
      html = html.replace(
        /(\s*<script type="module")/,
        `\n  ${shimTag}\n$1`,
      );
    } else {
      html = html.replace('</body>', `  ${shimTag}\n</body>`);
    }
    changed = true;
    console.log('Inserted wasm-brotli-shim.js into index.html');
  }

  if (changed) {
    writeFileSync(indexPath, html);
  }
}

const wasmFiles = readdirSync(DIST).filter((f) => f.endsWith('_bg.wasm'));
if (wasmFiles.length === 0) {
  console.error(`No *_bg.wasm files found in ${DIST}`);
  process.exit(1);
}

for (const wasmFile of wasmFiles) {
  const wasmPath = join(DIST, wasmFile);
  const raw = readFileSync(wasmPath);
  const compressed = brotliCompressSync(raw, {
    params: {
      [constants.BROTLI_PARAM_QUALITY]: 11,
    },
  });

  const ratio = ((100 * compressed.length) / raw.length).toFixed(1);
  console.log(
    `${wasmFile}: ${formatMiB(raw.length)} → ${formatMiB(compressed.length)} (brotli ${ratio}%)`,
  );

  if (compressed.length > MAX_BYTES) {
    console.error(
      `ERROR: ${wasmFile}.br is ${formatMiB(compressed.length)} — still above Cloudflare 25 MiB limit.`,
    );
    process.exit(1);
  }

  writeFileSync(`${wasmPath}.br`, compressed);
  unlinkSync(wasmPath);
  console.log(`Removed ${wasmFile} (deploying ${wasmFile}.br only)`);

  patchJsFiles(wasmFile.replace('_bg.wasm', ''));
}

copyFileSync(
  join(ROOT, 'deploy/cloudflare/wasm-brotli-shim.js'),
  join(DIST, 'wasm-brotli-shim.js'),
);
bundleBrotliDecoder();
patchIndexHtml();

console.log('Brotli WASM packaging complete.');
