import { useMemo, useState, useEffect } from "react";
import { getWasmSim } from "@/sim/useSimulation";
import { useSimStore } from "@/store/simStore";
import { KSP_COLORS } from "@/lib/ksp";
import { flatKey, OrbitLine } from "./OrbitLine";

export function ManeuverPreview() {
  const planner = useSimStore((s) => s.planner);
  const [epoch, setEpoch] = useState(0);

  useEffect(() => {
    setEpoch((e) => e + 1);
  }, [planner?.nodes.length, planner?.nodes, planner?.draft_prograde, planner?.draft_normal, planner?.draft_radial]);

  const flat = useMemo(() => {
    const sim = getWasmSim();
    if (!sim || !planner?.show_previews) return [];
    void epoch;
    const timed = sim.predict_maneuver_path_timed();
    if (timed.length >= 8) {
      const xyz: number[] = [];
      for (let i = 0; i + 3 < timed.length; i += 4) {
        xyz.push(timed[i + 1], timed[i + 2], timed[i + 3]);
      }
      return xyz;
    }
    return sim.predict_maneuver_path();
  }, [planner?.show_previews, planner?.nodes.length, epoch]);

  if (flat.length < 6) return null;

  return (
    <OrbitLine
      flat={flat}
      color={KSP_COLORS.maneuver}
      opacity={0.8}
      pickable
    />
  );
}
