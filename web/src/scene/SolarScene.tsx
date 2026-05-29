import { Html, OrbitControls, Stars } from "@react-three/drei";
import { Canvas } from "@react-three/fiber";
import { Suspense, useRef } from "react";
import * as THREE from "three";
import type { OrbitControls as OrbitControlsImpl } from "three-stdlib";
import { toScene } from "@/lib/units";
import { Bodies } from "./Bodies";
import { OrbitPaths, ManeuverNodeMarkers } from "./OrbitPaths";
import { ManeuverPreview } from "./ManeuverPreview";
import { ManeuverGizmo } from "./ManeuverGizmo";
import { CameraRig } from "./CameraRig";
import { useSimStore } from "@/store/simStore";

function SunLight() {
  const bodies = useSimStore((s) => s.bodies);
  const sun = bodies.find((b) => b.name === "Sun");
  const pos = sun?.position ?? [0, 0, 0];
  const sunScene = new THREE.Vector3(toScene(pos[0]), toScene(pos[1]), toScene(pos[2]));
  return (
    <>
      <pointLight position={sunScene} intensity={3} distance={0} decay={0} />
      <directionalLight
        position={sunScene}
        intensity={1.2}
        target={undefined}
      />
    </>
  );
}

export function SolarScene() {
  const controlsRef = useRef<OrbitControlsImpl>(null);

  return (
    <Canvas
      camera={{ position: [0, 6, 14], fov: 50, near: 0.00001, far: 5000 }}
      gl={{ antialias: true, alpha: false }}
      dpr={[1, 2]}
    >
      <color attach="background" args={["#070b14"]} />
      <ambientLight intensity={0.22} />
      <SunLight />
      <directionalLight position={[8, 4, 2]} intensity={0.25} />
      <Stars radius={400} depth={100} count={8000} factor={4} fade speed={0.15} />
      <Suspense fallback={null}>
        <Bodies />
        <OrbitPaths />
        <ManeuverPreview />
        <ManeuverNodeMarkers />
        <ManeuverGizmo />
      </Suspense>
      <OrbitControls
        ref={controlsRef}
        makeDefault
        enablePan
        minDistance={0.005}
        maxDistance={800}
        rotateSpeed={0.45}
        zoomSpeed={1.0}
        enableDamping
        dampingFactor={0.08}
      />
      <CameraRig controlsRef={controlsRef} />
      <Html fullscreen style={{ pointerEvents: "none" }}>
        <div className="absolute bottom-24 left-1/2 -translate-x-1/2 text-[10px] text-slate-500/80">
          Click to select · Double-click to frame · Click orbit to add node · Drag Δv handles
        </div>
      </Html>
    </Canvas>
  );
}
