import { Line } from "@react-three/drei";
import { useMemo, useRef, useState } from "react";
import { useFrame, useThree } from "@react-three/fiber";
import type { ThreeEvent } from "@react-three/fiber";
import * as THREE from "three";
import { useSimStore } from "@/store/simStore";
import { toScene } from "@/lib/units";
import { KSP_COLORS } from "@/lib/ksp";
import { axisDragDelta, tnwFrameForVessel } from "@/lib/orbitPick";
import { simDispatch } from "@/sim/useSimulation";
import { sceneBodiesRef } from "@/sim/sceneRefs";
import { isVessel } from "@/lib/vessels";

type Axis = "prograde" | "normal" | "radial";

const AXES: { key: Axis; color: string }[] = [
  { key: "prograde", color: KSP_COLORS.prograde },
  { key: "normal", color: KSP_COLORS.normal },
  { key: "radial", color: KSP_COLORS.radial },
];

const GIZMO_ARROW_LEN = 0.045;
const GIZMO_HANDLE_SCALE = 0.004;

/** KSP-style TNW Δv handles — only on vessels; WASM updated on pointer up. */
export function ManeuverGizmo() {
  const groupRef = useRef<THREE.Group>(null);
  const maneuverVessel = useSimStore((s) => s.maneuverVessel);
  const planner = useSimStore((s) => s.planner);
  const selectedNodeIndex = useSimStore((s) => s.selectedNodeIndex);
  const showGizmo = useSimStore((s) => s.showOrbits);
  const [dragValues, setDragValues] = useState<{
    prograde: number;
    normal: number;
    radial: number;
  } | null>(null);

  const editing =
    selectedNodeIndex !== null && planner?.nodes[selectedNodeIndex]
      ? planner.nodes[selectedNodeIndex]
      : null;

  const baseValues = useMemo(
    () => ({
      prograde: editing?.prograde ?? planner?.draft_prograde ?? 0,
      normal: editing?.normal ?? planner?.draft_normal ?? 0,
      radial: editing?.radial ?? planner?.draft_radial ?? 0,
    }),
    [editing, planner],
  );

  const values = dragValues ?? baseValues;

  useFrame(() => {
    if (!groupRef.current || !maneuverVessel) return;
    const vessel = sceneBodiesRef.current.find((b) => b.name === maneuverVessel);
    if (!vessel) return;
    groupRef.current.position.set(
      toScene(vessel.position[0]),
      toScene(vessel.position[1]),
      toScene(vessel.position[2]),
    );
  });

  if (!showGizmo || !maneuverVessel || !planner) return null;

  const vessel = sceneBodiesRef.current.find((b) => b.name === maneuverVessel);
  if (!vessel || !isVessel(vessel)) return null;

  const central = planner.central_body ?? "Sun";
  const frame = tnwFrameForVessel(sceneBodiesRef.current, vessel.name, central);
  if (!frame) return null;

  const commitValues = (next: { prograde: number; normal: number; radial: number }) => {
    setDragValues(null);
    if (selectedNodeIndex !== null) {
      simDispatch(
        {
          cmd: "update_node",
          index: selectedNodeIndex,
          prograde: next.prograde,
          normal: next.normal,
          radial: next.radial,
        },
        { sync: "planner" },
      );
    } else {
      simDispatch(
        { cmd: "set_draft_dv", prograde: next.prograde, normal: next.normal, radial: next.radial },
        { sync: "planner" },
      );
    }
  };

  const previewValues = (axis: Axis, value: number) => {
    setDragValues({
      prograde: axis === "prograde" ? value : values.prograde,
      normal: axis === "normal" ? value : values.normal,
      radial: axis === "radial" ? value : values.radial,
    });
  };

  return (
    <group ref={groupRef}>
      {AXES.map(({ key, color }) => {
        const val = values[key];
        const dir = (key === "prograde" ? frame.tHat : key === "normal" ? frame.nHat : frame.rHat).clone();
        const sign = Math.sign(val) || 1;
        const end = dir.clone().multiplyScalar(GIZMO_ARROW_LEN * sign * (0.5 + Math.min(Math.abs(val) / 2000, 1)));
        const axisDir = (key === "prograde" ? frame.tHat : key === "normal" ? frame.nHat : frame.rHat)
          .clone()
          .normalize();
        return (
          <group key={key}>
            <AxisLine end={end} color={color} />
            <DvDragHandle
              position={end}
              color={color}
              axis={axisDir}
              value={val}
              onPreview={(v) => previewValues(key, v)}
              onCommit={(v) => commitValues({ ...values, [key]: v })}
            />
          </group>
        );
      })}
    </group>
  );
}

function AxisLine({ end, color }: { end: THREE.Vector3; color: string }) {
  const points = useMemo(
    () =>
      [
        [0, 0, 0],
        [end.x, end.y, end.z],
      ] as [number, number, number][],
    [end.x, end.y, end.z],
  );
  return (
    <Line
      points={points}
      color={color}
      transparent
      opacity={0.7}
      depthWrite={false}
      toneMapped={false}
    />
  );
}

function DvDragHandle({
  position,
  color,
  axis,
  value,
  onPreview,
  onCommit,
}: {
  position: THREE.Vector3;
  color: string;
  axis: THREE.Vector3;
  value: number;
  onPreview: (v: number) => void;
  onCommit: (v: number) => void;
}) {
  const { camera } = useThree();
  const groupRef = useRef<THREE.Group>(null);
  const dragging = useRef(false);
  const startValue = useRef(value);
  const currentValue = useRef(value);
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
    startValue.current = value;
    currentValue.current = value;
    setActive(true);
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
  };

  const onMove = (e: ThreeEvent<PointerEvent>) => {
    if (!dragging.current) return;
    e.stopPropagation();
    const delta = axisDragDelta(axis, origin.current, e.movementX, e.movementY, camera, 2.5);
    const next = startValue.current + delta;
    currentValue.current = next;
    onPreview(next);
  };

  const onUp = (e: ThreeEvent<PointerEvent>) => {
    if (!dragging.current) return;
    dragging.current = false;
    setActive(false);
    (e.target as HTMLElement).releasePointerCapture(e.pointerId);
    onCommit(currentValue.current);
  };

  return (
    <group ref={groupRef} position={position}>
      <mesh
        scale={active ? GIZMO_HANDLE_SCALE * 1.25 : GIZMO_HANDLE_SCALE}
        onPointerDown={onDown}
        onPointerMove={onMove}
        onPointerUp={onUp}
        onPointerOver={() => setActive(true)}
        onPointerOut={() => !dragging.current && setActive(false)}
      >
        <sphereGeometry args={[1, 10, 10]} />
        <meshBasicMaterial color={color} toneMapped={false} transparent opacity={0.95} />
      </mesh>
    </group>
  );
}
