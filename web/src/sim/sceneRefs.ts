import { useSimStore } from "@/store/simStore";

/** Latest body state — updated every sim tick; read from useFrame (not React). */
export const sceneBodiesRef: { current: import("@/lib/units").BodySnapshot[] } = { current: [] };

/** Body metadata (radius, color, texture) — changes on scenario load only. */
export const bodyDefsRef: { current: import("@/lib/units").BodySnapshot[] } = { current: [] };

/** True while the user drags/zooms OrbitControls — CameraRig should not fight input. */
export const userCameraControlRef = { current: false };

export interface OrbitPathCacheEntry {
  name: string;
  flat: number[];
  key: string;
}

/** Scene geometry caches — filled only from the WASM access layer, never during React render. */
export const orbitPathsCacheRef: { current: OrbitPathCacheEntry[] } = { current: [] };
export const maneuverPreviewFlatRef: { current: number[] } = { current: [] };
export const maneuverMarkersFlatRef: { current: number[] } = { current: [] };

export function bumpSceneCacheEpoch() {
  useSimStore.getState().bumpSceneCacheEpoch();
}
