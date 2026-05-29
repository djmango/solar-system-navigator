import type { BodySnapshot } from "@/lib/units";

/** Latest body state — updated every sim tick; read from useFrame (not React). */
export const sceneBodiesRef: { current: BodySnapshot[] } = { current: [] };

/** Bump to rebuild static orbit line geometry. */
export let orbitPathsEpoch = 0;
export function bumpOrbitPaths() {
  orbitPathsEpoch += 1;
}

/** Body metadata (radius, color, texture) — changes on scenario load only. */
export const bodyDefsRef: { current: BodySnapshot[] } = { current: [] };
