import * as THREE from "three";
import type { BodySnapshot } from "./units";
import { toScene } from "./units";

/** Scale a world-space point around a central body for exaggerated intra-SOI display. */
export function scalePointAroundCentral(
  pointM: THREE.Vector3,
  centralM: THREE.Vector3,
  displayScale: number,
): THREE.Vector3 {
  if (displayScale <= 1.0001) return pointM;
  return centralM.clone().add(pointM.clone().sub(centralM).multiplyScalar(displayScale));
}

export function centralBodyMeters(
  bodies: BodySnapshot[],
  centralName: string | null,
): THREE.Vector3 | null {
  if (!centralName) return null;
  const central = bodies.find((b) => b.name === centralName);
  if (!central) return null;
  return new THREE.Vector3(central.position[0], central.position[1], central.position[2]);
}

/** Flat [x,y,z,...] world meters → scene points with optional intra-SOI exaggeration. */
export function flatToScaledScenePoints(
  flat: number[],
  centralM: THREE.Vector3 | null,
  displayScale: number,
): THREE.Vector3[] {
  const pts: THREE.Vector3[] = [];
  for (let i = 0; i + 2 < flat.length; i += 3) {
    const m = new THREE.Vector3(flat[i], flat[i + 1], flat[i + 2]);
    const scaled = centralM ? scalePointAroundCentral(m, centralM, displayScale) : m;
    pts.push(new THREE.Vector3(toScene(scaled.x), toScene(scaled.y), toScene(scaled.z)));
  }
  return pts;
}

/** [t,x,y,z,...] markers → scene positions with optional exaggeration. */
export function timedMarkersToScene(
  flat: number[],
  centralM: THREE.Vector3 | null,
  displayScale: number,
): THREE.Vector3[] {
  const pts: THREE.Vector3[] = [];
  for (let i = 0; i + 3 < flat.length; i += 4) {
    const m = new THREE.Vector3(flat[i + 1], flat[i + 2], flat[i + 3]);
    const scaled = centralM ? scalePointAroundCentral(m, centralM, displayScale) : m;
    pts.push(new THREE.Vector3(toScene(scaled.x), toScene(scaled.y), toScene(scaled.z)));
  }
  return pts;
}
