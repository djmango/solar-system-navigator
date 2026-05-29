import { useEffect, useRef } from "react";
import type { ThreeEvent } from "@react-three/fiber";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useSimStore } from "@/store/simStore";
import { toScene } from "@/lib/units";
import type { BodySnapshot } from "@/lib/units";
import { loadPlanetTexture } from "./textureCache";
import { bodyDefsRef, sceneBodiesRef } from "@/sim/sceneRefs";

const SPHERE = new THREE.SphereGeometry(1, 32, 32);
const ATMOSPHERE = new THREE.SphereGeometry(1, 20, 20);

function BodyMesh({ def }: { def: BodySnapshot }) {
  const groupRef = useRef<THREE.Group>(null);
  const surfaceRef = useRef<THREE.MeshBasicMaterial>(null);
  const selectedBody = useSimStore((s) => s.selectedBody);
  const focusBody = useSimStore((s) => s.focusBody);
  const setSelectedBody = useSimStore((s) => s.setSelectedBody);

  const radius = Math.max(toScene(def.display_radius), 0.003);
  const color = new THREE.Color(def.color[0], def.color[1], def.color[2]);
  const selected = def.name === selectedBody;
  const url = def.texture ? `/assets/${def.texture}` : null;

  useEffect(() => {
    if (!url) return;
    let alive = true;
    loadPlanetTexture(url)
      .then((map) => {
        if (!alive || !surfaceRef.current) return;
        surfaceRef.current.map = map;
        surfaceRef.current.color.set("#ffffff");
        surfaceRef.current.needsUpdate = true;
      })
      .catch(() => {});
    return () => {
      alive = false;
    };
  }, [url]);

  useFrame(() => {
    const live = sceneBodiesRef.current.find((b) => b.name === def.name);
    if (!live || !groupRef.current) return;
    groupRef.current.position.set(
      toScene(live.position[0]),
      toScene(live.position[1]),
      toScene(live.position[2]),
    );
  });

  const onSelect = (e: ThreeEvent<MouseEvent>) => {
    e.stopPropagation();
    setSelectedBody(def.name);
  };

  const onFocus = (e: ThreeEvent<MouseEvent>) => {
    e.stopPropagation();
    focusBody(def.name);
  };

  return (
    <group ref={groupRef}>
      {/* Pick target — modest size, invisible */}
      <mesh scale={Math.max(radius * 2, 0.04)} onClick={onSelect} onDoubleClick={onFocus}>
        <primitive object={SPHERE} attach="geometry" />
        <meshBasicMaterial visible={false} />
      </mesh>

      <mesh scale={radius}>
        <primitive object={SPHERE} attach="geometry" />
        <meshBasicMaterial ref={surfaceRef} color={color} toneMapped={false} />
      </mesh>

      {def.atmosphere && (
        <mesh scale={radius * 1.05}>
          <primitive object={ATMOSPHERE} attach="geometry" />
          <meshBasicMaterial
            color={new THREE.Color(def.atmosphere[0], def.atmosphere[1], def.atmosphere[2])}
            transparent
            opacity={0.12}
            depthWrite={false}
            toneMapped={false}
          />
        </mesh>
      )}

      {selected && (
        <mesh rotation={[Math.PI / 2, 0, 0]} scale={radius}>
          <ringGeometry args={[1.35, 1.5, 48]} />
          <meshBasicMaterial color="#6eb5ff" transparent opacity={0.7} side={THREE.DoubleSide} />
        </mesh>
      )}
    </group>
  );
}

export function Bodies() {
  const defs = useSimStore((s) => s.bodies);

  useEffect(() => {
    bodyDefsRef.current = defs;
  }, [defs]);

  return (
    <group>
      {defs.map((def) => (
        <BodyMesh key={def.name} def={def} />
      ))}
    </group>
  );
}
