import { useEffect, useMemo, useState } from "react";
import type { ThreeEvent } from "@react-three/fiber";
import * as THREE from "three";
import { useSimStore } from "@/store/simStore";
import { toScene } from "@/lib/units";
import type { BodySnapshot } from "@/lib/units";

function textureUrl(body: BodySnapshot): string | null {
  if (!body.texture) return null;
  return `/assets/${body.texture}`;
}

function BodyMesh({ body }: { body: BodySnapshot }) {
  const selectedBody = useSimStore((s) => s.selectedBody);
  const focusBody = useSimStore((s) => s.focusBody);
  const setSelectedBody = useSimStore((s) => s.setSelectedBody);

  const [x, y, z] = body.position.map(toScene) as [number, number, number];
  const radius = Math.max(toScene(body.display_radius), 0.003);
  const hitRadius = Math.max(radius * 4, 0.06);
  const color = new THREE.Color(body.color[0], body.color[1], body.color[2]);
  const selected = body.name === selectedBody;
  const url = textureUrl(body);

  const [map, setMap] = useState<THREE.Texture | null>(null);
  useEffect(() => {
    if (!url) {
      setMap(null);
      return;
    }
    let cancelled = false;
    const loader = new THREE.TextureLoader();
    loader.load(
      url,
      (tex) => {
        if (cancelled) {
          tex.dispose();
          return;
        }
        tex.colorSpace = THREE.SRGBColorSpace;
        setMap(tex);
      },
      undefined,
      () => {
        if (!cancelled) setMap(null);
      },
    );
    return () => {
      cancelled = true;
    };
  }, [url]);

  const onSelect = (e: ThreeEvent<MouseEvent>) => {
    e.stopPropagation();
    setSelectedBody(body.name);
  };

  const onFocus = (e: ThreeEvent<MouseEvent>) => {
    e.stopPropagation();
    focusBody(body.name);
  };

  return (
    <group position={[x, y, z]}>
      {/* Invisible pick target — planets are tiny at solar-system scale */}
      <mesh
        onClick={onSelect}
        onDoubleClick={onFocus}
        renderOrder={10}
      >
        <sphereGeometry args={[hitRadius, 16, 16]} />
        <meshBasicMaterial transparent opacity={0} depthWrite={false} />
      </mesh>

      <mesh>
        <sphereGeometry args={[radius, 48, 48]} />
        <meshStandardMaterial
          map={map ?? undefined}
          color={map ? "#ffffff" : color}
          emissive={body.fixed ? color : "#000000"}
          emissiveIntensity={body.fixed ? body.emissive * 0.4 : 0}
          roughness={map ? 1 : 0.92}
          metalness={0}
        />
      </mesh>

      {body.atmosphere && (
        <mesh scale={1.06}>
          <sphereGeometry args={[radius, 32, 32]} />
          <meshBasicMaterial
            color={new THREE.Color(body.atmosphere[0], body.atmosphere[1], body.atmosphere[2])}
            transparent
            opacity={0.2}
            depthWrite={false}
          />
        </mesh>
      )}

      {selected && (
        <mesh rotation={[Math.PI / 2, 0, 0]}>
          <ringGeometry args={[radius * 1.35, radius * 1.5, 64]} />
          <meshBasicMaterial color="#6eb5ff" transparent opacity={0.85} side={THREE.DoubleSide} />
        </mesh>
      )}

      {body.soi_radius > 0 && body.soi_radius < Infinity && (
        <SoiRing radius={toScene(body.soi_radius)} />
      )}
    </group>
  );
}

export function Bodies() {
  const bodies = useSimStore((s) => s.bodies);
  return (
    <group>
      {bodies.map((body) => (
        <BodyMesh key={body.name} body={body} />
      ))}
    </group>
  );
}

function SoiRing({ radius }: { radius: number }) {
  const geometry = useMemo(() => {
    const pts: THREE.Vector3[] = [];
    for (let i = 0; i <= 128; i++) {
      const a = (i / 128) * Math.PI * 2;
      pts.push(new THREE.Vector3(Math.cos(a) * radius, 0, Math.sin(a) * radius));
    }
    return new THREE.BufferGeometry().setFromPoints(pts);
  }, [radius]);

  return (
    <line geometry={geometry}>
      <lineBasicMaterial color="#64748b" transparent opacity={0.28} />
    </line>
  );
}
