import { useMemo } from "react";
import type { ThreeEvent } from "@react-three/fiber";
import * as THREE from "three";
import { getWasmSim, simAction } from "@/sim/useSimulation";
import { METERS_PER_UNIT, toScene } from "@/lib/units";

interface ClickableOrbitPathProps {
  points: THREE.Vector3[];
  color: string;
  opacity?: number;
  pickable?: boolean;
  tubeRadius?: number;
}

/** Visible orbit line + invisible pick tube (KSP-style click-to-add-node). */
export function ClickableOrbitPath({
  points,
  color,
  opacity = 0.55,
  pickable = true,
  tubeRadius = 0.025,
}: ClickableOrbitPathProps) {
  const curve = useMemo(() => {
    if (points.length < 2) return null;
    return new THREE.CatmullRomCurve3(points, true, "centripetal");
  }, [points]);

  const lineGeom = useMemo(() => {
    if (!curve) return null;
    return new THREE.BufferGeometry().setFromPoints(curve.getPoints(Math.max(points.length * 2, 64)));
  }, [curve, points.length]);

  const pickGeom = useMemo(() => {
    if (!curve || !pickable) return null;
    return new THREE.TubeGeometry(curve, Math.max(points.length * 3, 96), tubeRadius, 6, true);
  }, [curve, pickable, points.length, tubeRadius]);

  const onPick = (e: ThreeEvent<MouseEvent>) => {
    e.stopPropagation();
    const sim = getWasmSim();
    if (!sim) return;
    const meters = e.point.clone().multiplyScalar(METERS_PER_UNIT);
    simAction(() => {
      sim.add_maneuver_node_at_world_position(meters.x, meters.y, meters.z);
    });
  };

  if (!lineGeom) return null;

  return (
    <group>
      <line geometry={lineGeom}>
        <lineBasicMaterial color={color} transparent opacity={opacity} />
      </line>
      {pickGeom && (
        <mesh geometry={pickGeom} onClick={onPick} onDoubleClick={onPick}>
          <meshBasicMaterial transparent opacity={0} depthWrite={false} />
        </mesh>
      )}
    </group>
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
