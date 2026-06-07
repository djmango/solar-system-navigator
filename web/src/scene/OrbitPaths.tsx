import { useMemo, useRef, useState } from "react";
import { useFrame } from "@react-three/fiber";
import type { ThreeEvent } from "@react-three/fiber";
import * as THREE from "three";
import { useSimStore } from "@/store/simStore";
import { KSP_COLORS } from "@/lib/ksp";
import { OrbitLine } from "./OrbitLine";
import { bodyDefsRef, maneuverMarkersFlatRef, orbitPathsCacheRef } from "@/sim/sceneRefs";
import { toScene } from "@/lib/units";

export function OrbitPaths() {
  const showOrbits = useSimStore((s) => s.showOrbits);
  const maneuverVessel = useSimStore((s) => s.maneuverVessel);
  const cacheEpoch = useSimStore((s) => s.sceneCacheEpoch);

  const pickRadius = useMemo(() => {
    const body =
      bodyDefsRef.current.find((b) => b.name === maneuverVessel) ??
      useSimStore.getState().bodies.find((b) => b.name === maneuverVessel);
    if (!body) return 0.028;
    return Math.max(toScene(body.display_radius) * 0.35, 0.008);
  }, [maneuverVessel, cacheEpoch]);

  const paths = useMemo(() => {
    if (!showOrbits) return [];
    void cacheEpoch;
    return orbitPathsCacheRef.current;
  }, [showOrbits, cacheEpoch]);

  return (
    <group>
      {paths.map(({ name, flat, key }) => {
        const pickable = name === maneuverVessel;
        return flat.length >= 6 ? (
          <OrbitLine
            key={`${name}-${key}`}
            flat={flat}
            color={name === "Sun" ? "#554422" : pickable ? "#5a8fd4" : "#3d4a66"}
            opacity={pickable ? 0.75 : 0.45}
            pickable={pickable}
            pickRadius={pickRadius}
          />
        ) : null;
      })}
    </group>
  );
}

// Constant on-screen size for node markers (fraction of camera→marker distance).
const MARKER_SCREEN_SCALE = 0.022;

export function ManeuverNodeMarkers() {
  const cacheEpoch = useSimStore((s) => s.sceneCacheEpoch);
  const planner = useSimStore((s) => s.planner);
  const selectedNodeIndex = useSimStore((s) => s.selectedNodeIndex);
  const setSelectedNodeIndex = useSimStore((s) => s.setSelectedNodeIndex);

  // Markers come from WASM as [t,x,y,z,...] for unexecuted nodes only; map each
  // back to its planner-node index so clicking selects the right node.
  const markers = useMemo(() => {
    void cacheEpoch;
    const flat = maneuverMarkersFlatRef.current;
    const nodes = planner?.nodes ?? [];
    const out: { pos: THREE.Vector3; nodeIndex: number }[] = [];
    for (let i = 0; i + 3 < flat.length; i += 4) {
      const t = flat[i];
      let nodeIndex = -1;
      let bestDt = Infinity;
      nodes.forEach((n, ni) => {
        if (n.executed) return;
        const dt = Math.abs(n.time - t);
        if (dt < bestDt) {
          bestDt = dt;
          nodeIndex = ni;
        }
      });
      out.push({
        pos: new THREE.Vector3(toScene(flat[i + 1]), toScene(flat[i + 2]), toScene(flat[i + 3])),
        nodeIndex,
      });
    }
    return out;
  }, [cacheEpoch, planner]);

  return (
    <group>
      {markers.map(({ pos, nodeIndex }, i) =>
        // The selected node is drawn by the interactive gizmo instead.
        nodeIndex === selectedNodeIndex ? null : (
          <NodeMarker
            key={i}
            pos={pos}
            selected={false}
            onSelect={() => nodeIndex >= 0 && setSelectedNodeIndex(nodeIndex)}
          />
        ),
      )}
    </group>
  );
}

function NodeMarker({
  pos,
  selected,
  onSelect,
}: {
  pos: THREE.Vector3;
  selected: boolean;
  onSelect: () => void;
}) {
  const ref = useRef<THREE.Group>(null);
  const [hover, setHover] = useState(false);

  useFrame(({ camera }) => {
    if (!ref.current) return;
    const dist = camera.position.distanceTo(pos);
    const s = Math.max(dist * MARKER_SCREEN_SCALE, 1e-4) * (selected ? 1.35 : hover ? 1.15 : 1);
    ref.current.scale.setScalar(s);
  });

  const onClick = (e: ThreeEvent<MouseEvent>) => {
    e.stopPropagation();
    onSelect();
  };

  return (
    <group ref={ref} position={pos}>
      <mesh
        rotation={[Math.PI / 4, 0, Math.PI / 4]}
        onClick={onClick}
        onPointerOver={() => setHover(true)}
        onPointerOut={() => setHover(false)}
      >
        <octahedronGeometry args={[1, 0]} />
        <meshBasicMaterial
          color={selected ? "#ffffff" : KSP_COLORS.maneuver}
          wireframe={!selected}
          transparent
          opacity={selected ? 0.9 : 0.8}
          toneMapped={false}
        />
      </mesh>
    </group>
  );
}
