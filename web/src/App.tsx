import { TimeWarpBar } from "./ui/TimeWarpBar";
import { VesselHud } from "./ui/VesselHud";
import { ManeuverPlanner } from "./ui/ManeuverPlanner";
import { ViewControls } from "./ui/ViewControls";
import { SolarScene } from "./scene/SolarScene";
import { useSimulationLoop } from "./sim/useSimulation";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { useSimStore } from "./store/simStore";

export default function App() {
  useSimulationLoop();
  useKeyboardShortcuts();
  const ready = useSimStore((s) => s.ready);
  const error = useSimStore((s) => s.error);

  if (error) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-3 p-6 text-center">
        <h1 className="text-lg font-semibold text-amber-300">Failed to load simulation</h1>
        <p className="max-w-md text-sm text-slate-400">{error}</p>
      </div>
    );
  }

  return (
    <div className="relative h-full w-full">
      {!ready && (
        <div className="absolute inset-0 z-20 flex flex-col items-center justify-center gap-4 bg-[#070b14]">
          <div className="h-10 w-10 animate-spin rounded-full border-2 border-slate-700 border-t-sky-400" />
          <p className="text-sm text-slate-400">Loading physics engine…</p>
        </div>
      )}
      <SolarScene />
      {ready && (
        <div className="pointer-events-none absolute inset-0 z-10">
          <TimeWarpBar />
          <ViewControls />
          <VesselHud />
          <ManeuverPlanner />
        </div>
      )}
    </div>
  );
}
