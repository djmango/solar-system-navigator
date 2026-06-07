import { useMemo } from "react";
import { useSimStore } from "@/store/simStore";
import { KSP_COLORS } from "@/lib/ksp";
import { OrbitLine } from "./OrbitLine";
import { maneuverPreviewFlatRef, sceneBodiesRef, vesselDisplayScaleRef } from "@/sim/sceneRefs";
import { centralBodyMeters, flatToScaledScenePoints } from "@/lib/intraSoiDisplay";

export function ManeuverPreview() {
  const cacheEpoch = useSimStore((s) => s.sceneCacheEpoch);
  const showPreviews = useSimStore((s) => s.planner?.show_previews ?? false);
  const centralBody = useSimStore((s) => s.planner?.central_body ?? null);

  const points = useMemo(() => {
    if (!showPreviews) return null;
    void cacheEpoch;
    const flat = maneuverPreviewFlatRef.current;
    if (flat.length < 6) return null;
    const displayScale = vesselDisplayScaleRef.current;
    const centralM = centralBodyMeters(sceneBodiesRef.current, centralBody);
    return flatToScaledScenePoints(flat, centralM, displayScale);
  }, [showPreviews, cacheEpoch, centralBody]);

  if (!points || points.length < 2) return null;

  return (
    <OrbitLine points={points} color={KSP_COLORS.maneuver} opacity={0.85} pickable={false} />
  );
}
