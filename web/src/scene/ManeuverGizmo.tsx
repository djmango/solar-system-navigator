import { useMemo, useRef, useState } from "react";
import { useFrame, useThree } from "@react-three/fiber";
import type { ThreeEvent } from "@react-three/fiber";
import * as THREE from "three";
import { useSimStore } from "@/store/simStore";
import { toScene } from "@/lib/units";
import { KSP_COLORS } from "@/lib/ksp";
import { axisDragDelta, tnwFrameForVessel } from "@/lib/orbitPick";
import { getWasmSim, simAction } from "@/sim/useSimulation";
import { sceneBodiesRef } from "@/sim/sceneRefs";

type Axis = "prograde" | "normal" | "radial";

const AXES: { key: Axis; color: string }[] = [
  { key: "prograde", color: KSP_COLORS.prograde },
  { key: "normal", color: KSP_COLORS.normal },
  { key: "radial", color: KSP_COLORS.radial },
];

/** KSP-style TNW Δv handles — scaled to vessel, offset outside surface. */
export function ManeuverGizmo() {
  const groupRef = useRef<THREE.Group>(null);
  const selectedBody = useSimStore((s) => s.selectedBody);
  const planner = useSimStore((s) => s.planner);
  const selectedNodeIndex = useSimStore((s) => s.selectedNodeIndex);
  const showGizmo = useSimStore((s) => s.showOrbits);

  const editing =
    selectedNodeIndex !== null && planner?.nodes[selectedNodeIndex]
      ? planner.nodes[selectedNodeIndex]
      : null;

  const values = useMemo(
    () => ({
      prograde: editing?.prograde ?? planner?.draft_prograde ?? 0,
      normal: editing?.normal ?? planner?.draft_normal ?? 0,
      radial: editing?.radial ?? planner?.draft_radial ?? 0,
    }),
    [editing, planner],
  );

  useFrame(() => {
    if (!groupRef.current || !selectedBody) return;
    const vessel = sceneBodiesRef.current.find((b) => b.name === selectedBody);
    if (!vessel) return;
    groupRef.current.position.set(
      toScene(vessel.position[0]),
      toScene(vessel.position[1]),
      toScene(vessel.position[2]),
    );
  });

  if (!showGizmo || !selectedBody || !planner) return null;

  const vessel = sceneBodiesRef.current.find((b) => b.name === selectedBody);
  const central = planner.central_body ?? "Sun";
  const frame = vessel ? tnwFrameForVessel(sceneBodiesRef.current, vessel.name, central) : null;
  if (!vessel || !frame) return null;

  const bodyRadius = Math.max(toScene(vessel.display_radius), 0.003);
  const handleScale = Math.min(bodyRadius * 0.35, 0.008);
  const arrowLen = bodyRadius * 2.5;

  const setAxisValue = (axis: Axis, value: number) => {
    const p = axis === "prograde" ? value : values.prograde;
    const n = axis === "normal" ? value : values.normal;
    const r = axis === "radial" ? value : values.radial;
    if (selectedNodeIndex !== null) {
      simAction(() => getWasmSim()?.update_maneuver_node(selectedNodeIndex, p, n, r));
    } else {
      simAction(() => getWasmSim()?.set_draft_delta_v(p, n, r));
    }
  };

  return (
    <group ref={groupRef}>
      {AXES.map(({ key, color }) => {
        const val = values[key];
        const dir = (key === "prograde" ? frame.tHat : key === "normal" ? frame.nHat : frame.rHat).clone();
        const sign = Math.sign(val) || 1;
        const end = dir.clone().multiplyScalar(arrowLen * sign * (0.5 + Math.min(Math.abs(val) / 2000, 1)));
        const axisDir = (key === "prograde" ? frame.tHat : key === "normal" ? frame.nHat : frame.rHat).clone().normalize();
        return (
          <group key={key}>
            <AxisLine end={end} color={color} />
            <DvDragHandle
              position={end}
              color={color}
              axis={axisDir}
              handleScale={handleScale}
              value={val}
              onChange={(v) => setAxisValue(key, v)}
            />
          </group>
        );
      })}
    </group>
  );
}

function AxisLine({ end, color }: { end: THREE.Vector3; color: string }) {
  const geom = useMemo(
    () => new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(0, 0, 0), end]),
    [end.x, end.y, end.z],
  );
  return (
    <line geometry={geom}>
      <lineBasicMaterial color={color} transparent opacity={0.7} depthWrite={false} toneMapped={false} />
    </line>
  );
}

function DvDragHandle({
  position,
  color,
  axis,
  handleScale,
  value,
  onChange,
}: {
  position: THREE.Vector3;
  color: string;
  axis: THREE.Vector3;
  handleScale: number;
  value: number;
  onChange: (v: number) => void;
}) {
  const { camera } = useThree();
  const groupRef = useRef<THREE.Group>(null);
  const dragging = useRef(false);
  const startValue = useRef(value);
  const [active, setActive] = useState(false);
  const origin = useRef(new THREE.Vector3());

  useFrame(() => {
    if (groupRef.current?.parent) {
      origin.current.copy(groupRef.current.parent.position);
    }
  });

  const onDown = (e: ThreeEvent<PointerEvent>) => {
    e.stopPropagation();
    dragging.current = true;
    setActive(true);
    startValue.current = value;
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
  };

  const onMove = (e: ThreeEvent<PointerEvent>) => {
    if (!dragging.current) return;
    e.stopPropagation();
    const delta = axisDragDelta(axis, origin.current, e.movementX, e.movementY, camera, 2.5);
    onChange(startValue.current + delta);
  };

  const onUp = (e: ThreeEvent<PointerEvent>) => {
    if (!dragging.current) return;
    dragging.current = false;
    setActive(false);
    (e.target as HTMLElement).releasePointerCapture(e.pointerId);
  };

  return (
    <group ref={groupRef} position={position}>
      <mesh
        scale={active ? handleScale * 1.3 : handleScale}
        onPointerDown={onDown}
        onPointerMove={onMove}
        onPointerUp={onUp}
        onPointerOver={() => setActive(true)}
        onPointerOut={() => !dragging.current && setActive(false)}
      >
        <sphereGeometry args={[1, 12, 12]} />
        <meshBasicMaterial color={color} toneMapped={false} depthTest={false} transparent opacity={0.95} />
      </mesh>
    </group>
  );
}
