import * as THREE from "three";
import type { BodySnapshot } from "./units";

export interface TnwFrame {
  tHat: THREE.Vector3;
  nHat: THREE.Vector3;
  rHat: THREE.Vector3;
  origin: THREE.Vector3;
}

export function tnwFrameForVessel(
  bodies: BodySnapshot[],
  vesselName: string,
  centralName: string,
): TnwFrame | null {
  const vessel = bodies.find((b) => b.name === vesselName);
  const central = bodies.find((b) => b.name === centralName);
  if (!vessel || !central) return null;

  const relPos = new THREE.Vector3(
    vessel.position[0] - central.position[0],
    vessel.position[1] - central.position[1],
    vessel.position[2] - central.position[2],
  );
  const relVel = new THREE.Vector3(
    vessel.velocity[0] - central.velocity[0],
    vessel.velocity[1] - central.velocity[1],
    vessel.velocity[2] - central.velocity[2],
  );

  const rHat = relPos.lengthSq() > 0 ? relPos.clone().normalize() : new THREE.Vector3(1, 0, 0);
  const h = new THREE.Vector3().crossVectors(relPos, relVel);
  const nHat = h.lengthSq() > 0 ? h.clone().normalize() : new THREE.Vector3(0, 1, 0);
  const tHat = new THREE.Vector3().crossVectors(nHat, rHat).normalize();

  return {
    tHat,
    nHat,
    rHat,
    origin: new THREE.Vector3(vessel.position[0], vessel.position[1], vessel.position[2]),
  };
}

/** Project screen drag (pixels) onto a world axis → Δv m/s delta. */
export function axisDragDelta(
  axisWorldUnit: THREE.Vector3,
  originScene: THREE.Vector3,
  deltaX: number,
  deltaY: number,
  camera: THREE.Camera,
  sensitivity = 1.0,
): number {
  const origin = originScene.clone().project(camera);
  const tip = originScene.clone().add(axisWorldUnit).project(camera);
  const axisScreen = new THREE.Vector2(tip.x - origin.x, tip.y - origin.y);
  if (axisScreen.lengthSq() < 1e-8) return -deltaY * sensitivity * 12;
  axisScreen.normalize();
  const mouse = new THREE.Vector2(deltaX, -deltaY);
  // Scale with camera distance so drag feel is consistent across zoom levels.
  const dist = camera.position.distanceTo(originScene);
  const zoomFactor = Math.min(Math.max(dist * 0.35, 0.04), 2.5);
  return mouse.dot(axisScreen) * sensitivity * 14 * zoomFactor;
}

export function timedFlatToScenePoints(flat: number[]): THREE.Vector3[] {
  const pts: THREE.Vector3[] = [];
  for (let i = 0; i + 3 < flat.length; i += 4) {
    pts.push(new THREE.Vector3(flat[i + 1], flat[i + 2], flat[i + 3]));
  }
  return pts;
}
