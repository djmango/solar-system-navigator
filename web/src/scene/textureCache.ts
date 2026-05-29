import * as THREE from "three";

const cache = new Map<string, THREE.Texture>();
const loader = new THREE.TextureLoader();

export function loadPlanetTexture(url: string): Promise<THREE.Texture> {
  const cached = cache.get(url);
  if (cached) return Promise.resolve(cached);
  return new Promise((resolve, reject) => {
    loader.load(
      url,
      (tex) => {
        tex.colorSpace = THREE.SRGBColorSpace;
        tex.anisotropy = 4;
        cache.set(url, tex);
        resolve(tex);
      },
      undefined,
      reject,
    );
  });
}

export function clearTextureCache() {
  for (const tex of cache.values()) tex.dispose();
  cache.clear();
}
