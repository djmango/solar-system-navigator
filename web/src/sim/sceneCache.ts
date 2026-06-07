import { flatKey } from "@/lib/flatPath";
import {
  bumpSceneCacheEpoch,
  maneuverMarkersFlatRef,
  maneuverPreviewFlatRef,
  orbitPathsCacheRef,
  vesselDisplayScaleRef,
  type OrbitPathCacheEntry,
} from "./sceneRefs";

export interface SceneCacheSnapshot {
  orbit_paths: { name: string; flat: number[]; display_scale?: number }[];
  maneuver_preview: number[];
  maneuver_markers: number[];
  vessel_display_scale?: number;
}

function mapOrbitPaths(
  orbitPaths: SceneCacheSnapshot["orbit_paths"],
): OrbitPathCacheEntry[] {
  return (orbitPaths ?? []).map(
    (entry): OrbitPathCacheEntry => ({
      name: entry.name,
      flat: entry.flat,
      key: flatKey(entry.flat),
      displayScale: entry.display_scale ?? 1,
    }),
  );
}

export function applySceneCache(cache: SceneCacheSnapshot) {
  orbitPathsCacheRef.current = mapOrbitPaths(cache.orbit_paths);
  maneuverPreviewFlatRef.current = cache.maneuver_preview ?? [];
  maneuverMarkersFlatRef.current = cache.maneuver_markers ?? [];
  vesselDisplayScaleRef.current = cache.vessel_display_scale ?? 1;
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
  orbitPaths: SceneCacheSnapshot["orbit_paths"],
  markers: number[],
  preview?: number[],
  vesselDisplayScale?: number,
) {
  orbitPathsCacheRef.current = mapOrbitPaths(orbitPaths);
  maneuverMarkersFlatRef.current = markers ?? [];
  if (preview !== undefined) {
    maneuverPreviewFlatRef.current = preview;
  }
  if (vesselDisplayScale !== undefined) {
    vesselDisplayScaleRef.current = vesselDisplayScale;
  }
  bumpSceneCacheEpoch();
}
