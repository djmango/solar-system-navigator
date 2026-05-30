import { Line } from "@react-three/drei";
import { useEffect, useMemo, useState } from "react";
import type { ThreeEvent } from "@react-three/fiber";
import * as THREE from "three";
import { simDispatch } from "@/sim/useSimulation";
import { useSimStore } from "@/store/simStore";
import { flatKey } from "@/lib/flatPath";
import { METERS_PER_UNIT, toScene } from "@/lib/units";

interface OrbitLineProps {
  flat: number[];
  color: string;
  opacity?: number;
  pickable?: boolean;
  pickRadius?: number;
}

function flatToPoints(flat: number[]): THREE.Vector3[] {
  const pts: THREE.Vector3[] = [];
  for (let i = 0; i + 2 < flat.length; i += 3) {
    pts.push(new THREE.Vector3(toScene(flat[i]), toScene(flat[i + 1]), toScene(flat[i + 2])));
  }
  return pts;
}

export function OrbitLine({
  flat,
  color,
  opacity = 0.5,
  pickable = false,
  pickRadius = 0.028,
}: OrbitLineProps) {
  const points = useMemo(() => flatToPoints(flat), [flatKey(flat)]);
  const linePoints = useMemo(
    () => points.map((p) => [p.x, p.y, p.z] as [number, number, number]),
    [points],
  );

  const [pickGeometry, setPickGeometry] = useState<THREE.BufferGeometry | null>(null);

  useEffect(() => {
    if (!pickable || points.length < 2) {
      setPickGeometry(null);
      return;
    }
    const curve = new THREE.CatmullRomCurve3(points, true, "centripetal");
    const segments = Math.min(Math.max(points.length * 2, 64), 192);
    const geo = new THREE.TubeGeometry(curve, segments, pickRadius, 6, true);
    setPickGeometry(geo);
    return () => geo.dispose();
  }, [points, pickable, pickRadius]);

  const onPick = (e: ThreeEvent<MouseEvent>) => {
    if (!pickable) return;
    e.stopPropagation();
    const meters = e.point.clone().multiplyScalar(METERS_PER_UNIT);
    simDispatch(
      {
        cmd: "add_node_at_world",
        x: meters.x,
        y: meters.y,
        z: meters.z,
      },
      {
        notice: "Maneuver node placed on orbit",
        onComplete: () => {
          const planner = useSimStore.getState().planner;
          if (planner?.nodes.length) {
            useSimStore.getState().setSelectedNodeIndex(planner.nodes.length - 1);
          }
        },
      },
    );
  };

  if (points.length < 2) return null;

  return (
    <group>
      <Line
        points={linePoints}
        color={color}
        transparent
        opacity={opacity}
        depthWrite={false}
      />
      {pickGeometry && (
        <mesh geometry={pickGeometry} onClick={onPick} onDoubleClick={onPick}>
          <meshBasicMaterial transparent opacity={0} depthWrite={false} side={THREE.DoubleSide} />
        </mesh>
      )}
    </group>
  );
}

export { flatKey } from "@/lib/flatPath";
