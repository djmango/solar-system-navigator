import { useFrame, useThree } from "@react-three/fiber";
import { useEffect, useRef } from "react";
import * as THREE from "three";
import type { OrbitControls as OrbitControlsImpl } from "three-stdlib";
import { useSimStore } from "@/store/simStore";
import { toScene } from "@/lib/units";

export function CameraRig({ controlsRef }: { controlsRef: React.RefObject<OrbitControlsImpl | null> }) {
  const { camera } = useThree();
  const bodies = useSimStore((s) => s.bodies);
  const selectedBody = useSimStore((s) => s.selectedBody);
  const followSelection = useSimStore((s) => s.followSelection);
  const frameRequest = useSimStore((s) => s.frameRequest);
  const framingRef = useRef(false);
  const frameUntilRef = useRef(0);

  useEffect(() => {
    framingRef.current = true;
    frameUntilRef.current = performance.now() + 1200;
  }, [frameRequest, selectedBody]);

  useFrame((_, delta) => {
    const controls = controlsRef.current;
    if (!controls || !selectedBody) return;

    const body = bodies.find((b) => b.name === selectedBody);
    if (!body) return;

    const target = new THREE.Vector3(
      toScene(body.position[0]),
      toScene(body.position[1]),
      toScene(body.position[2]),
    );

    const lerp = 1 - Math.exp(-10 * delta);
    controls.target.lerp(target, followSelection ? lerp : lerp * 0.5);

    const shouldFrame = framingRef.current && performance.now() < frameUntilRef.current;
    if (shouldFrame) {
      const dist = Math.max(toScene(body.display_radius) * 10, 0.12);
      const dir = new THREE.Vector3(0.45, 0.35, 1).normalize();
      const desired = target.clone().add(dir.multiplyScalar(dist));
      camera.position.lerp(desired, 1 - Math.exp(-7 * delta));
    } else {
      framingRef.current = false;
    }

    controls.update();
  });

  return null;
}
