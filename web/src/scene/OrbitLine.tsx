import { useMemo, useRef } from "react";
import type { ThreeEvent } from "@react-three/fiber";
import * as THREE from "three";
import { getWasmSim, simAction } from "@/sim/useSimulation";
import { METERS_PER_UNIT, toScene } from "@/lib/units";

interface OrbitLineProps {
  /** Flat xyz in meters */
  flat: number[];
  color: string;
  opacity?: number;
  pickable?: boolean;
}

/** Lightweight orbit polyline — geometry built once per path update, not every frame. */
export function OrbitLine({ flat, color, opacity = 0.5, pickable = false }: OrbitLineProps) {
  const geomRef = useRef<THREE.BufferGeometry | null>(null);

  const geometry = useMemo(() => {
    geomRef.current?.dispose();
    const pts: THREE.Vector3[] = [];
    for (let i = 0; i + 2 < flat.length; i += 3) {
      pts.push(new THREE.Vector3(toScene(flat[i]), toScene(flat[i + 1]), toScene(flat[i + 2])));
    }
    if (pts.length < 2) return null;
    const geo = new THREE.BufferGeometry().setFromPoints(pts);
    geomRef.current = geo;
    return geo;
  }, [flat]);

  const onPick = (e: ThreeEvent<MouseEvent>) => {
    if (!pickable) return;
    e.stopPropagation();
    const sim = getWasmSim();
    if (!sim) return;
    const meters = e.point.clone().multiplyScalar(METERS_PER_UNIT);
    simAction(() => {
      sim.add_maneuver_node_at_world_position(meters.x, meters.y, meters.z);
    });
  };

  if (!geometry) return null;

  return (
    <line geometry={geometry} onClick={pickable ? onPick : undefined}>
      <lineBasicMaterial
        color={color}
        transparent
        opacity={opacity}
        depthWrite={false}
        linewidth={1}
      />
    </line>
  );
}

export function flatToScenePoints(flat: number[]): THREE.Vector3[] {
  const pts: THREE.Vector3[] = [];
  for (let i = 0; i + 2 < flat.length; i += 3) {
    pts.push(new THREE.Vector3(toScene(flat[i]), toScene(flat[i + 1]), toScene(flat[i + 2])));
  }
  return pts;
}

export function timedFlatToScene(flat: number[]): THREE.Vector3[] {
  const pts: THREE.Vector3[] = [];
  for (let i = 0; i + 3 < flat.length; i += 4) {
    pts.push(new THREE.Vector3(toScene(flat[i + 1]), toScene(flat[i + 2]), toScene(flat[i + 3])));
  }
  return pts;
}

/** Stable string key so useMemo does not invalidate when array ref changes but data is same. */
export function flatKey(flat: number[]): string {
  if (flat.length === 0) return "";
  return `${flat.length}:${flat[0]}:${flat[flat.length - 1]}`;
}
