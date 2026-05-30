import { useMemo } from "react";
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

export function ManeuverNodeMarkers() {
  const cacheEpoch = useSimStore((s) => s.sceneCacheEpoch);

  const markers = useMemo(() => {
    void cacheEpoch;
    const flat = maneuverMarkersFlatRef.current;
    const out: THREE.Vector3[] = [];
    for (let i = 0; i + 3 < flat.length; i += 4) {
      out.push(new THREE.Vector3(toScene(flat[i + 1]), toScene(flat[i + 2]), toScene(flat[i + 3])));
    }
    return out;
  }, [cacheEpoch]);

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
