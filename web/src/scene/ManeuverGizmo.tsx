import { Line } from "@react-three/drei";
import { useMemo, useRef, useState } from "react";
import { useFrame, useThree } from "@react-three/fiber";
import type { ThreeEvent } from "@react-three/fiber";
import * as THREE from "three";
import { useSimStore } from "@/store/simStore";
import { toScene } from "@/lib/units";
import { KSP_COLORS } from "@/lib/ksp";
import { axisDragDelta } from "@/lib/orbitPick";
import { simDispatch } from "@/sim/useSimulation";
import {
  maneuverMarkersFlatRef,
  maneuverPreviewFlatRef,
  sceneBodiesRef,
  vesselDisplayScaleRef,
} from "@/sim/sceneRefs";
import { centralBodyMeters, scalePointAroundCentral } from "@/lib/intraSoiDisplay";

type Axis = "prograde" | "normal" | "radial";

const AXES: { key: Axis; color: string }[] = [
  { key: "prograde", color: KSP_COLORS.prograde },
  { key: "normal", color: KSP_COLORS.normal },
  { key: "radial", color: KSP_COLORS.radial },
];

// Gizmo size is held constant in screen space: arm length ≈ this fraction of the
// camera→node distance, so it never balloons when zoomed in or vanishes far out.
const SCREEN_SCALE = 0.07;
const HANDLE_R = 0.16;
const HANDLE_HIT_R = 0.34;

interface NodeFrame {
  pos: THREE.Vector3; // scene-space node position
  tHat: THREE.Vector3;
  nHat: THREE.Vector3;
  rHat: THREE.Vector3;
}

function nodeWorldMeters(nodeTime: number): THREE.Vector3 | null {
  const flat = maneuverMarkersFlatRef.current;
  let best: THREE.Vector3 | null = null;
  let bestDt = Infinity;
  for (let i = 0; i + 3 < flat.length; i += 4) {
    const dt = Math.abs(flat[i] - nodeTime);
    if (dt < bestDt) {
      bestDt = dt;
      best = new THREE.Vector3(flat[i + 1], flat[i + 2], flat[i + 3]);
    }
  }
  return best;
}

/** Prograde tangent at the node from the predicted (amber) path, in meters. */
function previewTangentMeters(nodeM: THREE.Vector3): THREE.Vector3 | null {
  const flat = maneuverPreviewFlatRef.current;
  const n = Math.floor(flat.length / 3);
  if (n < 2) return null;
  let bestI = 0;
  let bestD = Infinity;
  for (let i = 0; i < n; i++) {
    const dx = flat[i * 3] - nodeM.x;
    const dy = flat[i * 3 + 1] - nodeM.y;
    const dz = flat[i * 3 + 2] - nodeM.z;
    const d = dx * dx + dy * dy + dz * dz;
    if (d < bestD) {
      bestD = d;
      bestI = i;
    }
  }
  const a = Math.max(0, bestI - 1);
  const b = Math.min(n - 1, bestI + 1);
  if (a === b) return null;
  const t = new THREE.Vector3(
    flat[b * 3] - flat[a * 3],
    flat[b * 3 + 1] - flat[a * 3 + 1],
    flat[b * 3 + 2] - flat[a * 3 + 2],
  );
  return t.lengthSq() > 0 ? t.normalize() : null;
}

/** KSP-style TNW Δv handles, anchored on the selected node, constant screen size. */
export function ManeuverGizmo() {
  const groupRef = useRef<THREE.Group>(null);
  const planner = useSimStore((s) => s.planner);
  const selectedNodeIndex = useSimStore((s) => s.selectedNodeIndex);
  const maneuverVessel = useSimStore((s) => s.maneuverVessel);
  const cacheEpoch = useSimStore((s) => s.sceneCacheEpoch);
  const [dragValues, setDragValues] = useState<{
    prograde: number;
    normal: number;
    radial: number;
  } | null>(null);

  const node =
    selectedNodeIndex !== null &&
    planner?.nodes[selectedNodeIndex] &&
    !planner.nodes[selectedNodeIndex].executed
      ? planner.nodes[selectedNodeIndex]
      : null;

  const baseValues = useMemo(
    () => ({
      prograde: node?.prograde ?? 0,
      normal: node?.normal ?? 0,
      radial: node?.radial ?? 0,
    }),
    [node],
  );
  const values = dragValues ?? baseValues;

  // Recompute the burn frame on each sync / selection change. Positions are
  // effectively static while paused (the normal planning case).
  const frame = useMemo<NodeFrame | null>(() => {
    if (!node || !planner) return null;
    void cacheEpoch;
    const nodeM = nodeWorldMeters(node.time);
    if (!nodeM) return null;

    const displayScale = vesselDisplayScaleRef.current;
    const centralM =
      centralBodyMeters(sceneBodiesRef.current, planner.central_body) ?? new THREE.Vector3();
    const nodeDisplayM =
      displayScale > 1.0001
        ? scalePointAroundCentral(nodeM, centralM, displayScale)
        : nodeM;

    const rHat = nodeM.clone().sub(centralM);
    if (rHat.lengthSq() === 0) rHat.set(1, 0, 0);
    rHat.normalize();

    let tHat = previewTangentMeters(nodeM);
    if (!tHat) {
      const vessel = sceneBodiesRef.current.find((b) => b.name === maneuverVessel);
      tHat = vessel
        ? new THREE.Vector3(vessel.velocity[0], vessel.velocity[1], vessel.velocity[2]).normalize()
        : new THREE.Vector3(0, 0, 1);
    }
    const nHat = new THREE.Vector3().crossVectors(rHat, tHat);
    if (nHat.lengthSq() === 0) nHat.set(0, 1, 0);
    nHat.normalize();
    // Re-orthogonalize prograde so the triad is clean.
    const tOrtho = new THREE.Vector3().crossVectors(nHat, rHat).normalize();

    return {
      pos: new THREE.Vector3(toScene(nodeDisplayM.x), toScene(nodeDisplayM.y), toScene(nodeDisplayM.z)),
      tHat: tOrtho,
      nHat,
      rHat,
    };
  }, [node, planner, maneuverVessel, cacheEpoch]);

  useFrame(({ camera }) => {
    const g = groupRef.current;
    if (!g || !frame) return;
    const dist = camera.position.distanceTo(frame.pos);
    g.scale.setScalar(Math.max(dist * SCREEN_SCALE, 1e-4));
  });

  if (!frame || selectedNodeIndex === null) return null;

  const commit = (next: { prograde: number; normal: number; radial: number }) => {
    setDragValues(null);
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
  };

  return (
    <group ref={groupRef} position={frame.pos}>
      {/* Node hub */}
      <mesh>
        <sphereGeometry args={[HANDLE_R * 0.7, 12, 12]} />
        <meshBasicMaterial color={KSP_COLORS.maneuver} toneMapped={false} />
      </mesh>
      {AXES.map(({ key, color }) => {
        const dir = (key === "prograde" ? frame.tHat : key === "normal" ? frame.nHat : frame.rHat).clone();
        return (
          <AxisGizmo
            key={key}
            dir={dir}
            color={color}
            origin={frame.pos}
            value={values[key]}
            onPreview={(v) =>
              setDragValues({
                prograde: key === "prograde" ? v : values.prograde,
                normal: key === "normal" ? v : values.normal,
                radial: key === "radial" ? v : values.radial,
              })
            }
            onCommit={(v) => commit({ ...values, [key]: v })}
          />
        );
      })}
    </group>
  );
}

/** One axis: a bidirectional arm with a + and − drag handle (KSP pro/retro pairs). */
function AxisGizmo({
  dir,
  color,
  origin,
  value,
  onPreview,
  onCommit,
}: {
  dir: THREE.Vector3;
  color: string;
  origin: THREE.Vector3;
  value: number;
  onPreview: (v: number) => void;
  onCommit: (v: number) => void;
}) {
  const linePoints = useMemo(
    () =>
      [
        [-dir.x, -dir.y, -dir.z],
        [dir.x, dir.y, dir.z],
      ] as [number, number, number][],
    [dir.x, dir.y, dir.z],
  );

  return (
    <group>
      <Line points={linePoints} color={color} transparent opacity={0.65} depthWrite={false} toneMapped={false} />
      <DragHandle dir={dir} sign={1} color={color} origin={origin} value={value} onPreview={onPreview} onCommit={onCommit} />
      <DragHandle dir={dir} sign={-1} color={color} origin={origin} value={value} onPreview={onPreview} onCommit={onCommit} />
    </group>
  );
}

function DragHandle({
  dir,
  sign,
  color,
  origin,
  value,
  onPreview,
  onCommit,
}: {
  dir: THREE.Vector3;
  sign: number;
  color: string;
  origin: THREE.Vector3;
  value: number;
  onPreview: (v: number) => void;
  onCommit: (v: number) => void;
}) {
  const { camera } = useThree();
  const dragging = useRef(false);
  const startPointer = useRef(new THREE.Vector2());
  const startValue = useRef(value);
  const current = useRef(value);
  const [hover, setHover] = useState(false);
  // Drag along the world axis in the handle's direction (sign folds into delta).
  const worldAxis = useMemo(() => dir.clone().multiplyScalar(sign), [dir, sign]);

  const onDown = (e: ThreeEvent<PointerEvent>) => {
    e.stopPropagation();
    dragging.current = true;
    startPointer.current.set(e.clientX, e.clientY);
    startValue.current = value;
    current.current = value;
    (e.target as Element).setPointerCapture?.(e.pointerId);
  };
  const onMove = (e: ThreeEvent<PointerEvent>) => {
    if (!dragging.current) return;
    e.stopPropagation();
    const delta = axisDragDelta(
      worldAxis,
      origin,
      e.clientX - startPointer.current.x,
      e.clientY - startPointer.current.y,
      camera,
      1.0,
    );
    current.current = startValue.current + delta;
    onPreview(current.current);
  };
  const onUp = (e: ThreeEvent<PointerEvent>) => {
    if (!dragging.current) return;
    dragging.current = false;
    (e.target as Element).releasePointerCapture?.(e.pointerId);
    onCommit(current.current);
  };

  return (
    <group position={[dir.x * sign, dir.y * sign, dir.z * sign]}>
      <mesh
        scale={HANDLE_HIT_R}
        onPointerDown={onDown}
        onPointerMove={onMove}
        onPointerUp={onUp}
        onPointerCancel={onUp}
        onPointerOver={() => setHover(true)}
        onPointerOut={() => !dragging.current && setHover(false)}
      >
        <sphereGeometry args={[1, 12, 12]} />
        <meshBasicMaterial transparent opacity={0} depthWrite={false} />
      </mesh>
      <mesh scale={hover ? HANDLE_R * 1.3 : HANDLE_R}>
        <sphereGeometry args={[1, 12, 12]} />
        <meshBasicMaterial color={color} toneMapped={false} transparent opacity={sign > 0 ? 0.95 : 0.5} />
      </mesh>
    </group>
  );
}
