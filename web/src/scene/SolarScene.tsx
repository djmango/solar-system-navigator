import { Html, OrbitControls } from "@react-three/drei";
import { Canvas } from "@react-three/fiber";
import { Suspense, useRef } from "react";
import * as THREE from "three";
import type { OrbitControls as OrbitControlsImpl } from "three-stdlib";
import { Bodies } from "./Bodies";
import { OrbitPaths, ManeuverNodeMarkers } from "./OrbitPaths";
import { ManeuverPreview } from "./ManeuverPreview";
import { ManeuverGizmo } from "./ManeuverGizmo";
import { CameraRig } from "./CameraRig";
import { Starfield } from "./Starfield";

export function SolarScene() {
  const controlsRef = useRef<OrbitControlsImpl>(null);

  return (
    <Canvas
      camera={{ position: [0, 6, 14], fov: 50, near: 0.001, far: 2000 }}
      gl={{
        antialias: true,
        alpha: false,
        powerPreference: "high-performance",
        toneMapping: THREE.NoToneMapping,
      }}
      dpr={[1, 1.5]}
      frameloop="always"
    >
      <color attach="background" args={["#070b14"]} />
      <ambientLight intensity={0.35} />
      <directionalLight position={[100, 40, 60]} intensity={0.9} />
      <Starfield />
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
        minDistance={0.02}
        maxDistance={400}
        rotateSpeed={0.45}
        zoomSpeed={1.0}
        enableDamping
        dampingFactor={0.1}
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
