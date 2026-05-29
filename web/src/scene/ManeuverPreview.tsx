import { useMemo } from "react";
import { getWasmSim } from "@/sim/useSimulation";
import { useSimStore } from "@/store/simStore";
import { KSP_COLORS } from "@/lib/ksp";
import { ClickableOrbitPath, flatToScenePoints, timedFlatToScene } from "./ClickableOrbitPath";

export function ManeuverPreview() {
  const simTime = useSimStore((s) => s.simTime);
  const planner = useSimStore((s) => s.planner);

  const points = useMemo(() => {
    const sim = getWasmSim();
    if (!sim) return [];
    const timed = sim.predict_maneuver_path_timed();
    if (timed.length >= 8) return timedFlatToScene(timed);
    const flat = sim.predict_maneuver_path();
    return flatToScenePoints(flat);
  }, [simTime, planner]);

  if (points.length < 2) return null;

  return (
    <ClickableOrbitPath
      points={points}
      color={KSP_COLORS.maneuver}
      opacity={0.85}
      pickable
      tubeRadius={0.045}
    />
  );
}
