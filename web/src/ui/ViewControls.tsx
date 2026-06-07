import { Eye, EyeOff, HelpCircle, Orbit, Target } from "lucide-react";
import { useSimStore } from "@/store/simStore";
import { reloadScenarioByIndex } from "@/sim/useSimulation";
import { SCENARIO_CATALOG } from "@/lib/scenarios";
import { vesselBodies } from "@/lib/vessels";
import { KSP_SHORTCUTS } from "@/lib/ksp";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { clearSelectedScratchNode } from "@/sim/maneuverNodeUx";

export function ViewControls() {
  const showOrbits = useSimStore((s) => s.showOrbits);
  const setShowOrbits = useSimStore((s) => s.setShowOrbits);
  const followSelection = useSimStore((s) => s.followSelection);
  const setFollowSelection = useSimStore((s) => s.setFollowSelection);
  const soiAuto = useSimStore((s) => s.soiAuto);
  const setSoiAuto = useSimStore((s) => s.setSoiAuto);
  const showShortcuts = useSimStore((s) => s.showShortcuts);
  const setShowShortcuts = useSimStore((s) => s.setShowShortcuts);
  const bodies = useSimStore((s) => s.bodies);
  const maneuverVessel = useSimStore((s) => s.maneuverVessel);
  const selectManeuverVessel = useSimStore((s) => s.selectManeuverVessel);
  const scenarioIndex = useSimStore((s) => s.scenarioIndex);
  const vessels = vesselBodies(bodies);

  return (
    <>
      <Card className="pointer-events-auto absolute left-3 top-[5.5rem] w-44 border-slate-600/50 sm:left-4">
        <CardHeader className="py-1.5">
          <CardTitle>View</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2 pb-2">
          <ToggleRow
            icon={showOrbits ? Eye : EyeOff}
            label="Orbit paths (V)"
            active={showOrbits}
            onClick={() => setShowOrbits(!showOrbits)}
          />
          <ToggleRow
            icon={Target}
            label="Follow (G)"
            active={followSelection}
            onClick={() => setFollowSelection(!followSelection)}
          />
          <ToggleRow
            icon={Orbit}
            label="SOI auto (O)"
            active={soiAuto}
            onClick={() => setSoiAuto(!soiAuto)}
          />

          <label className="block text-[10px] text-slate-500">
            Scenario (1–3)
            <select
              className="mt-0.5 w-full rounded-md border border-slate-600 bg-slate-950 px-2 py-1 text-xs text-slate-200"
              value={scenarioIndex}
              onChange={(e) => void reloadScenarioByIndex(Number(e.target.value))}
            >
              {SCENARIO_CATALOG.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.label}
                </option>
              ))}
            </select>
          </label>

          <label className="block text-[10px] text-slate-500">
            Vessel
            <select
              className="mt-0.5 w-full rounded-md border border-slate-600 bg-slate-950 px-2 py-1 text-xs text-slate-200"
              value={maneuverVessel ?? vessels[0]?.name ?? ""}
              onChange={(e) => {
                clearSelectedScratchNode();
                selectManeuverVessel(e.target.value);
              }}
            >
              {vessels.map((b) => (
                <option key={b.name} value={b.name}>
                  {b.name}
                </option>
              ))}
            </select>
          </label>

          <Button
            variant="ghost"
            size="sm"
            className="w-full justify-start gap-2 text-slate-400"
            onClick={() => setShowShortcuts(!showShortcuts)}
          >
            <HelpCircle className="h-3.5 w-3.5" />
            Shortcuts (?)
          </Button>
        </CardContent>
      </Card>

      {showShortcuts && <ShortcutsPanel onClose={() => setShowShortcuts(false)} />}
    </>
  );
}

function ToggleRow({
  icon: Icon,
  label,
  active,
  onClick,
}: {
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`flex w-full items-center gap-2 rounded-md border px-2 py-1.5 text-left text-[11px] transition-colors ${
        active
          ? "border-sky-700/50 bg-sky-950/40 text-sky-200"
          : "border-slate-700/50 text-slate-400 hover:bg-slate-800/50"
      }`}
    >
      <Icon className="h-3.5 w-3.5 shrink-0" />
      {label}
    </button>
  );
}

function ShortcutsPanel({ onClose }: { onClose: () => void }) {
  return (
    <Card className="pointer-events-auto absolute left-1/2 top-1/2 z-30 w-[min(90vw,24rem)] -translate-x-1/2 -translate-y-1/2 border-slate-600/60">
      <CardHeader className="flex flex-row items-center justify-between py-2">
        <CardTitle>KSP shortcuts</CardTitle>
        <Button variant="ghost" size="sm" onClick={onClose}>
          Close
        </Button>
      </CardHeader>
      <CardContent className="max-h-64 overflow-y-auto pb-3">
        <ul className="space-y-1.5">
          {KSP_SHORTCUTS.map(({ keys, action }) => (
            <li key={keys} className="flex justify-between gap-3 text-[11px]">
              <kbd className="rounded border border-slate-600 bg-slate-900 px-1.5 py-0.5 font-mono text-sky-300">
                {keys}
              </kbd>
              <span className="text-right text-slate-400">{action}</span>
            </li>
          ))}
        </ul>
      </CardContent>
    </Card>
  );
}
