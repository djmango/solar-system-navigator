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
  const centralBody = useSimStore((s) => s.planner?.central_body ?? null);
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

    // Keep the depth range matched to the current zoom so we can fly from
    // solar-system scale down to a vessel's orbit without clipping or z-fighting.
    if (camera instanceof THREE.PerspectiveCamera) {
      const viewDist = camera.position.distanceTo(controls.target);
      const near = Math.min(Math.max(viewDist * 0.002, 1e-6), 1);
      const far = Math.max(viewDist * 50, 4000);
      if (Math.abs(camera.near - near) > near * 0.1 || camera.far !== far) {
        camera.near = near;
        camera.far = far;
        camera.updateProjectionMatrix();
      }
    }

    if (userCameraControlRef.current || !selectedBody) {
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
      const dist = frameDistance(body, centralBody);
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

/** Frame the body together with its orbit around the current central body, so a
 *  selected vessel shows its trajectory rather than just the dot. */
function frameDistance(
  body: import("@/lib/units").BodySnapshot,
  centralBody: string | null,
): number {
  const central = centralBody
    ? sceneBodiesRef.current.find((b) => b.name === centralBody)
    : null;
  if (central && central.name !== body.name) {
    const orbitalR = Math.hypot(
      toScene(body.position[0] - central.position[0]),
      toScene(body.position[1] - central.position[1]),
      toScene(body.position[2] - central.position[2]),
    );
    const dist = orbitalR * 2.4;
    return Math.min(Math.max(dist, toScene(body.display_radius) * 4, 0.02), 60);
  }
  return Math.max(toScene(body.display_radius) * 12, 0.15);
}
