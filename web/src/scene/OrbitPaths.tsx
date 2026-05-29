import { useEffect, useMemo, useState } from "react";
import * as THREE from "three";
import { getWasmSim } from "@/sim/useSimulation";
import { useSimStore } from "@/store/simStore";
import { KSP_COLORS } from "@/lib/ksp";
import { flatKey, OrbitLine } from "./OrbitLine";
import { bodyDefsRef, bumpOrbitPaths, orbitPathsEpoch } from "@/sim/sceneRefs";
import { toScene } from "@/lib/units";
import type { BodySnapshot } from "@/lib/units";

export function OrbitPaths() {
  const showOrbits = useSimStore((s) => s.showOrbits);
  const selectedBody = useSimStore((s) => s.selectedBody);
  const scenarioIndex = useSimStore((s) => s.scenarioIndex);
  const planner = useSimStore((s) => s.planner);
  const bodyNames = useSimStore((s) => s.bodies.map((b) => b.name).join("|"));
  const [epoch, setEpoch] = useState(orbitPathsEpoch);

  useEffect(() => {
    setEpoch(orbitPathsEpoch);
  }, [scenarioIndex, selectedBody, planner?.nodes.length, showOrbits]);

  useEffect(() => {
    bumpOrbitPaths();
    setEpoch(orbitPathsEpoch);
  }, [scenarioIndex]);

  const paths = useMemo(() => {
    if (!showOrbits) return [];
    const sim = getWasmSim();
    if (!sim) return [];
    void epoch;
    const names = bodyDefsRef.current.length
      ? bodyDefsRef.current.map((b) => b.name)
      : (sim.body_snapshots() as BodySnapshot[]).map((b) => b.name);
    return names.map((name) => {
      const flat = sim.truth_path_flat(name);
      const data = flat.length >= 6 ? flat : sim.orbit_preview_flat(name, 96);
      return {
        name,
        flat: data,
        key: flatKey(data),
        pickable: name === selectedBody,
      };
    });
  }, [showOrbits, selectedBody, epoch, scenarioIndex, bodyNames]);

  return (
    <group>
      {paths.map(({ name, flat, key, pickable }) =>
        flat.length >= 6 ? (
          <OrbitLine
            key={`${name}-${key}`}
            flat={flat}
            color={name === "Sun" ? "#554422" : "#3d4a66"}
            pickable={pickable}
          />
        ) : null,
      )}
    </group>
  );
}

export function ManeuverNodeMarkers() {
  const planner = useSimStore((s) => s.planner);
  const [epoch, setEpoch] = useState(0);

  useEffect(() => {
    setEpoch((e) => e + 1);
  }, [planner?.nodes.length]);

  const markers = useMemo(() => {
    const sim = getWasmSim();
    if (!sim || !planner?.nodes.length) return [];
    void epoch;
    const flat = sim.maneuver_node_markers_flat();
    const out: THREE.Vector3[] = [];
    for (let i = 0; i + 3 < flat.length; i += 4) {
      out.push(new THREE.Vector3(toScene(flat[i + 1]), toScene(flat[i + 2]), toScene(flat[i + 3])));
    }
    return out;
  }, [planner?.nodes.length, epoch]);

  return (
    <group>
      {markers.map((pos, i) => (
        <mesh key={i} position={pos} rotation={[Math.PI / 4, 0, Math.PI / 4]}>
          <octahedronGeometry args={[0.025, 0]} />
          <meshBasicMaterial color={KSP_COLORS.maneuver} wireframe toneMapped={false} />
        </mesh>
      ))}
    </group>
  );
}
