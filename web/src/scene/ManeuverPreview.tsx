import { useMemo } from "react";
import { useSimStore } from "@/store/simStore";
import { KSP_COLORS } from "@/lib/ksp";
import { OrbitLine } from "./OrbitLine";
import { maneuverPreviewFlatRef } from "@/sim/sceneRefs";

export function ManeuverPreview() {
  const cacheEpoch = useSimStore((s) => s.sceneCacheEpoch);
  const showPreviews = useSimStore((s) => s.planner?.show_previews ?? false);

  const flat = useMemo(() => {
    if (!showPreviews) return [];
    void cacheEpoch;
    return maneuverPreviewFlatRef.current;
  }, [showPreviews, cacheEpoch]);

  if (flat.length < 6) return null;

  return (
    <OrbitLine flat={flat} color={KSP_COLORS.maneuver} opacity={0.85} pickable={false} />
  );
}
