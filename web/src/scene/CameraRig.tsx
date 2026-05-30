import { useFrame } from "@react-three/fiber";
import { useEffect, useRef } from "react";
import * as THREE from "three";
import type { OrbitControls as OrbitControlsImpl } from "three-stdlib";
import { useSimStore } from "@/store/simStore";
import { toScene } from "@/lib/units";
import { sceneBodiesRef, userCameraControlRef } from "@/sim/sceneRefs";

export function CameraRig({ controlsRef }: { controlsRef: React.RefObject<OrbitControlsImpl | null> }) {
  const selectedBody = useSimStore((s) => s.selectedBody);
  const followSelection = useSimStore((s) => s.followSelection);
  const frameRequest = useSimStore((s) => s.frameRequest);
  const framingRef = useRef(false);
  const frameUntilRef = useRef(0);
  const target = useRef(new THREE.Vector3());
  const desiredCam = useRef(new THREE.Vector3());

  useEffect(() => {
    framingRef.current = true;
    frameUntilRef.current = performance.now() + 1200;
  }, [frameRequest, selectedBody]);

  useFrame(({ camera }, delta) => {
    const controls = controlsRef.current;
    if (!controls) return;

    if (userCameraControlRef.current) {
      controls.update();
      return;
    }

    if (!selectedBody) {
      controls.update();
      return;
    }

    const body = sceneBodiesRef.current.find((b) => b.name === selectedBody);
    if (!body) {
      controls.update();
      return;
    }

    target.current.set(
      toScene(body.position[0]),
      toScene(body.position[1]),
      toScene(body.position[2]),
    );

    const lerp = 1 - Math.exp(-10 * delta);
    if (followSelection) {
      controls.target.lerp(target.current, lerp);
    }

    const shouldFrame = framingRef.current && performance.now() < frameUntilRef.current;
    if (shouldFrame) {
      const dist = Math.max(toScene(body.display_radius) * 12, 0.15);
      const dir = new THREE.Vector3(0.45, 0.35, 1).normalize();
      desiredCam.current.copy(target.current).add(dir.multiplyScalar(dist));
      camera.position.lerp(desiredCam.current, 1 - Math.exp(-8 * delta));
    } else {
      framingRef.current = false;
    }

    controls.update();
  });

  return null;
}
