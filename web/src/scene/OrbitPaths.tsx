import { useMemo } from "react";
import * as THREE from "three";
import { getWasmSim } from "@/sim/useSimulation";
import { useSimStore } from "@/store/simStore";
import { toScene } from "@/lib/units";
import { KSP_COLORS } from "@/lib/ksp";
import { ClickableOrbitPath, flatToScenePoints } from "./ClickableOrbitPath";

export function OrbitPaths() {
  const bodies = useSimStore((s) => s.bodies);
  const showOrbits = useSimStore((s) => s.showOrbits);
  const selectedBody = useSimStore((s) => s.selectedBody);
  const simTime = useSimStore((s) => s.simTime);

  const paths = useMemo(() => {
    if (!showOrbits) return [];
    const sim = getWasmSim();
    if (!sim) return [];
    return bodies.map((body) => {
      const flat = sim.truth_path_flat(body.name);
      if (flat.length >= 6) {
        return { name: body.name, points: flatToScenePoints(flat), pickable: body.name === selectedBody };
      }
      const preview = sim.orbit_preview_flat(body.name, 128);
      return { name: body.name, points: flatToScenePoints(preview), pickable: body.name === selectedBody };
    });
  }, [bodies, showOrbits, selectedBody, simTime]);

  return (
    <group>
      {paths.map(({ name, points, pickable }) =>
        points.length >= 2 ? (
          pickable ? (
            <ClickableOrbitPath
              key={name}
              points={points}
              color={name === "Sun" ? "#554422" : "#3d4a66"}
              pickable
              tubeRadius={0.035}
            />
          ) : (
            <ClickableOrbitPath
              key={name}
              points={points}
              color={name === "Sun" ? "#554422" : "#3d4a66"}
              pickable={false}
            />
          )
        ) : null,
      )}
    </group>
  );
}

export function ManeuverNodeMarkers() {
  const simTime = useSimStore((s) => s.simTime);
  const planner = useSimStore((s) => s.planner);

  const markers = useMemo(() => {
    const sim = getWasmSim();
    if (!sim || !planner?.nodes.length) return [];
    const flat = sim.maneuver_node_markers_flat();
    const out: THREE.Vector3[] = [];
    for (let i = 0; i + 3 < flat.length; i += 4) {
      out.push(new THREE.Vector3(toScene(flat[i + 1]), toScene(flat[i + 2]), toScene(flat[i + 3])));
    }
    return out;
  }, [simTime, planner]);

  return (
    <group>
      {markers.map((pos, i) => (
        <mesh key={i} position={pos} rotation={[Math.PI / 4, 0, Math.PI / 4]}>
          <octahedronGeometry args={[0.04, 0]} />
          <meshBasicMaterial color={KSP_COLORS.maneuver} wireframe />
        </mesh>
      ))}
    </group>
  );
}
