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
import { SceneErrorBoundary } from "./SceneErrorBoundary";
import { userCameraControlRef } from "@/sim/sceneRefs";
import { resetSceneFault } from "@/sim/useSimulation";
import { useSimStore } from "@/store/simStore";
import { clearOrDeselectSelectedNode } from "@/sim/maneuverNodeUx";

function SceneFallback() {
  return (
    <mesh>
      <boxGeometry args={[0.001, 0.001, 0.001]} />
      <meshBasicMaterial visible={false} />
    </mesh>
  );
}

export function SolarScene() {
  const controlsRef = useRef<OrbitControlsImpl>(null);
  const sceneRemountKey = useSimStore((s) => s.sceneRemountKey);
  const setSceneFault = useSimStore((s) => s.setSceneFault);
  const bumpSceneRemountKey = useSimStore((s) => s.bumpSceneRemountKey);

  return (
    <Canvas
      key={`solar-canvas-${sceneRemountKey}`}
      camera={{ position: [0, 6, 14], fov: 50, near: 0.001, far: 2000 }}
      gl={{
        antialias: true,
        alpha: false,
        powerPreference: "high-performance",
        toneMapping: THREE.NoToneMapping,
      }}
      dpr={[1, 1.5]}
      frameloop="always"
      onPointerMissed={() => {
        clearOrDeselectSelectedNode();
      }}
      onCreated={({ gl }) => {
        const canvas = gl.domElement;
        canvas.addEventListener("webglcontextlost", (e) => {
          e.preventDefault();
          setSceneFault("WebGL context lost — use Retry scene or refresh the page.");
        });
        canvas.addEventListener("webglcontextrestored", () => {
          resetSceneFault();
          bumpSceneRemountKey();
        });
      }}
    >
      <color attach="background" args={["#070b14"]} />
      <ambientLight intensity={0.35} />
      <directionalLight position={[100, 40, 60]} intensity={0.9} />
      <Starfield />
      <SceneErrorBoundary
        resetKey={sceneRemountKey}
        fallback={<SceneFallback />}
        onError={(error) => setSceneFault(error.message)}
      >
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
          minDistance={0.0008}
          maxDistance={400}
          rotateSpeed={0.55}
          zoomSpeed={1.25}
          onStart={() => {
            userCameraControlRef.current = true;
          }}
          onEnd={() => {
            userCameraControlRef.current = false;
          }}
        />
        <CameraRig controlsRef={controlsRef} />
        <Html fullscreen style={{ pointerEvents: "none" }}>
          <div className="absolute bottom-24 left-1/2 -translate-x-1/2 text-[10px] text-slate-500/80">
            Click to select · Double-click to frame · Click orbit to add node · Drag Δv handles
          </div>
        </Html>
      </SceneErrorBoundary>
    </Canvas>
  );
}
