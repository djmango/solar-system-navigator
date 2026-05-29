import { useMemo, useRef, useState } from "react";
import { useThree } from "@react-three/fiber";
import type { ThreeEvent } from "@react-three/fiber";
import * as THREE from "three";
import { useSimStore } from "@/store/simStore";
import { toScene } from "@/lib/units";
import { KSP_COLORS } from "@/lib/ksp";
import { axisDragDelta, tnwFrameForVessel } from "@/lib/orbitPick";
import { getWasmSim, simAction } from "@/sim/useSimulation";

type Axis = "prograde" | "normal" | "radial";

const AXES: { key: Axis; color: string }[] = [
  { key: "prograde", color: KSP_COLORS.prograde },
  { key: "normal", color: KSP_COLORS.normal },
  { key: "radial", color: KSP_COLORS.radial },
];

/** KSP-style draggable TNW Δv handles at the active vessel. */
export function ManeuverGizmo() {
  const bodies = useSimStore((s) => s.bodies);
  const selectedBody = useSimStore((s) => s.selectedBody);
  const planner = useSimStore((s) => s.planner);
  const selectedNodeIndex = useSimStore((s) => s.selectedNodeIndex);

  const vessel = bodies.find((b) => b.name === selectedBody);
  const central = planner?.central_body ?? "Sun";
  const frame = vessel ? tnwFrameForVessel(bodies, vessel.name, central) : null;

  if (!vessel || !frame || !planner) return null;

  const editing =
    selectedNodeIndex !== null && planner.nodes[selectedNodeIndex]
      ? planner.nodes[selectedNodeIndex]
      : null;

  const values = {
    prograde: editing?.prograde ?? planner.draft_prograde,
    normal: editing?.normal ?? planner.draft_normal,
    radial: editing?.radial ?? planner.draft_radial,
  };

  const pos = new THREE.Vector3(
    toScene(vessel.position[0]),
    toScene(vessel.position[1]),
    toScene(vessel.position[2]),
  );

  const axisVec = (axis: Axis) => {
    const dir =
      axis === "prograde" ? frame.tHat : axis === "normal" ? frame.nHat : frame.rHat;
    return dir.clone().normalize();
  };

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
    <group position={pos}>
      {AXES.map(({ key, color }) => {
        const val = values[key];
        const dir = axisVec(key);
        const len = Math.max(Math.abs(val) / 500, 0.05) * Math.sign(val || 1);
        const end = dir.clone().multiplyScalar(len * 0.15);
        return (
          <group key={key}>
            <AxisLine end={end} color={color} />
            <DvDragHandle
              position={end}
              color={color}
              axis={dir}
              origin={pos}
              value={val}
              onChange={(v) => setAxisValue(key, v)}
            />
          </group>
        );
      })}
      <mesh>
        <octahedronGeometry args={[0.025, 0]} />
        <meshBasicMaterial color={KSP_COLORS.maneuver} wireframe transparent opacity={0.9} />
      </mesh>
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
      <lineBasicMaterial color={color} transparent opacity={0.85} />
    </line>
  );
}

function DvDragHandle({
  position,
  color,
  axis,
  origin,
  value,
  onChange,
}: {
  position: THREE.Vector3;
  color: string;
  axis: THREE.Vector3;
  origin: THREE.Vector3;
  value: number;
  onChange: (v: number) => void;
}) {
  const { camera } = useThree();
  const dragging = useRef(false);
  const startValue = useRef(value);
  const [active, setActive] = useState(false);

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
    const delta = axisDragDelta(axis, origin, e.movementX, e.movementY, camera);
    onChange(startValue.current + delta);
  };

  const onUp = (e: ThreeEvent<PointerEvent>) => {
    if (!dragging.current) return;
    dragging.current = false;
    setActive(false);
    (e.target as HTMLElement).releasePointerCapture(e.pointerId);
  };

  return (
    <mesh
      position={position}
      onPointerDown={onDown}
      onPointerMove={onMove}
      onPointerUp={onUp}
      onPointerOver={() => setActive(true)}
      onPointerOut={() => !dragging.current && setActive(false)}
    >
      <sphereGeometry args={[active ? 0.022 : 0.016, 16, 16]} />
      <meshBasicMaterial color={color} transparent opacity={active ? 1 : 0.85} />
    </mesh>
  );
}
