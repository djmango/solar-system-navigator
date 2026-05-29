// Bundled for the browser by scripts/compress-wasm-brotli.mjs (esbuild).
const decode = require('brotli/decompress');

exports.decompress = function (u8) {
  return new Uint8Array(
    decode(Buffer.from(u8.buffer, u8.byteOffset, u8.byteLength)),
  );
};
