/**
 * Patches fetch() so wasm-bindgen loads Brotli-compressed *_bg.wasm.br assets.
 * Must run before the Trunk/wasm-bindgen module (inserted by compress-wasm-brotli.mjs).
 */
import brotliDecode from './brotli-decode.mjs';

const decompress = brotliDecode.decompress;

function wasmUrl(input) {
  if (typeof input === 'string') {
    return input;
  }
  if (input instanceof URL) {
    return input.href;
  }
  if (input instanceof Request) {
    return input.url;
  }
  return null;
}

function toBrotliUrl(url) {
  if (!url || !url.includes('_bg.wasm') || url.includes('_bg.wasm.br')) {
    return url;
  }
  return url.replace('_bg.wasm', '_bg.wasm.br');
}

async function decompressWasmResponse(response) {
  if (!response.ok) {
    return response;
  }
  const compressed = new Uint8Array(await response.arrayBuffer());
  const wasm = decompress(compressed);
  return new Response(wasm, {
    status: response.status,
    statusText: response.statusText,
    headers: { 'Content-Type': 'application/wasm' },
  });
}

export function installBrotliWasmFetch() {
  if (globalThis.__solarBrotliFetchInstalled) {
    return;
  }
  globalThis.__solarBrotliFetchInstalled = true;

  const origFetch = globalThis.fetch.bind(globalThis);
  globalThis.fetch = async (input, init) => {
    const url = wasmUrl(input);
    const brotliUrl = toBrotliUrl(url);
    if (brotliUrl && brotliUrl !== url) {
      const target =
        typeof input === 'string'
          ? brotliUrl
          : input instanceof Request
            ? new Request(brotliUrl, input)
            : brotliUrl;
      return decompressWasmResponse(await origFetch(target, init));
    }
    if (url?.includes('_bg.wasm.br')) {
      return decompressWasmResponse(await origFetch(input, init));
    }
    return origFetch(input, init);
  };
}

installBrotliWasmFetch();
