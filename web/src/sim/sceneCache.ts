import { flatKey } from "@/lib/flatPath";
import {
  bumpSceneCacheEpoch,
  maneuverMarkersFlatRef,
  maneuverPreviewFlatRef,
  orbitPathsCacheRef,
  type OrbitPathCacheEntry,
} from "./sceneRefs";

export interface SceneCacheSnapshot {
  orbit_paths: { name: string; flat: number[] }[];
  maneuver_preview: number[];
  maneuver_markers: number[];
}

export function applySceneCache(cache: SceneCacheSnapshot) {
  orbitPathsCacheRef.current = (cache.orbit_paths ?? []).map(
    (entry): OrbitPathCacheEntry => ({
      name: entry.name,
      flat: entry.flat,
      key: flatKey(entry.flat),
    }),
  );
  maneuverPreviewFlatRef.current = cache.maneuver_preview ?? [];
  maneuverMarkersFlatRef.current = cache.maneuver_markers ?? [];
  bumpSceneCacheEpoch();
}

export function applyManeuverLayers(preview: number[], markers: number[]) {
  maneuverPreviewFlatRef.current = preview;
  maneuverMarkersFlatRef.current = markers;
  bumpSceneCacheEpoch();
}

/**
 * RAF scene update: refreshes orbit paths + markers every call, but only swaps the
 * (expensive) maneuver preview when `preview` is provided. Passing `undefined` leaves
 * the previous preview untouched so it can refresh on a slower cadence without clearing.
 */
export function applyRafScene(
  orbitPaths: { name: string; flat: number[] }[],
  markers: number[],
  preview?: number[],
) {
  orbitPathsCacheRef.current = (orbitPaths ?? []).map(
    (entry): OrbitPathCacheEntry => ({
      name: entry.name,
      flat: entry.flat,
      key: flatKey(entry.flat),
    }),
  );
  maneuverMarkersFlatRef.current = markers ?? [];
  if (preview !== undefined) {
    maneuverPreviewFlatRef.current = preview;
  }
  bumpSceneCacheEpoch();
}
