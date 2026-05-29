import { ChevronDown, ChevronUp, Plus, Rocket, Trash2, Zap } from "lucide-react";
import { getWasmSim, simAction } from "@/sim/useSimulation";
import { useSimStore } from "@/store/simStore";
import { formatTime } from "@/lib/units";
import { KSP_COLORS } from "@/lib/ksp";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

export function ManeuverPlanner() {
  const planner = useSimStore((s) => s.planner);
  const simTime = useSimStore((s) => s.simTime);
  const selectedNodeIndex = useSimStore((s) => s.selectedNodeIndex);
  const setSelectedNodeIndex = useSimStore((s) => s.setSelectedNodeIndex);
  const lastHohmann = useSimStore((s) => s.lastHohmann);
  const setLastHohmann = useSimStore((s) => s.setLastHohmann);

  if (!planner) return null;

  const editing =
    selectedNodeIndex !== null && planner.nodes[selectedNodeIndex]
      ? planner.nodes[selectedNodeIndex]
      : null;

  const prograde = editing?.prograde ?? planner.draft_prograde;
  const normal = editing?.normal ?? planner.draft_normal;
  const radial = editing?.radial ?? planner.draft_radial;
  const totalDv = Math.hypot(prograde, normal, radial);
  const burnUt = editing?.time ?? simTime + planner.default_burn_offset;

  const setDv = (p: number, n: number, r: number) => {
    if (selectedNodeIndex !== null) {
      simAction(() => getWasmSim()?.update_maneuver_node(selectedNodeIndex, p, n, r));
    } else {
      simAction(() => getWasmSim()?.set_draft_delta_v(p, n, r));
    }
  };

  return (
    <Card className="pointer-events-auto absolute bottom-3 right-3 top-auto flex max-h-[calc(100%-5.5rem)] w-[min(100%-1.5rem,22rem)] flex-col border-amber-900/30 sm:bottom-4 sm:right-4 sm:top-24 sm:max-h-[calc(100%-7rem)]">
      <CardHeader className="border-amber-900/20 bg-amber-950/20">
        <CardTitle className="flex items-center gap-2 text-amber-200/90">
          <Rocket className="h-3.5 w-3.5" />
          Maneuver node
        </CardTitle>
      </CardHeader>

      <CardContent className="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto py-3">
        <div className="flex items-center justify-between text-[10px] text-slate-500">
          <span>Node UT</span>
          <span className="font-mono text-slate-300">{formatTime(burnUt)}</span>
        </div>
        <div className="flex items-center justify-between text-[10px] text-slate-500">
          <span>Time to burn</span>
          <span className="font-mono text-amber-300/90">{formatTime(Math.max(0, burnUt - simTime))}</span>
        </div>
        <p className="text-[10px] leading-relaxed text-slate-500">
          Click an orbit to place a node · drag colored handles on the vessel
        </p>

        <DvRow
          label="Prograde"
          color={KSP_COLORS.prograde}
          value={prograde}
          onChange={(v) => setDv(v, normal, radial)}
          onNudge={(d) => setDv(prograde + d, normal, radial)}
        />
        <DvRow
          label="Normal"
          color={KSP_COLORS.normal}
          value={normal}
          onChange={(v) => setDv(prograde, v, radial)}
          onNudge={(d) => setDv(prograde, normal + d, radial)}
        />
        <DvRow
          label="Radial"
          color={KSP_COLORS.radial}
          value={radial}
          onChange={(v) => setDv(prograde, normal, v)}
          onNudge={(d) => setDv(prograde, normal, radial + d)}
        />

        <div className="rounded-md border border-slate-700/60 bg-slate-900/60 px-2 py-1.5">
          <div className="text-[10px] uppercase tracking-wide text-slate-500">Total Δv</div>
          <div className="font-mono text-lg text-amber-300">{totalDv.toFixed(1)} m/s</div>
        </div>

        <div className="flex flex-wrap gap-1.5">
          <Button variant="maneuver" size="sm" className="flex-1" title="Add node (B)" onClick={addNode}>
            <Plus className="h-3 w-3" />
            Create node
          </Button>
          <Button
            variant="secondary"
            size="sm"
            title="Hohmann (H)"
            onClick={() =>
              simAction(() => {
                const xfer = getWasmSim()?.compute_hohmann();
                if (xfer) setLastHohmann(xfer as never);
                getWasmSim()?.apply_hohmann_departure_draft();
              })
            }
          >
            <Zap className="h-3 w-3" />
          </Button>
          <Button variant="destructive" size="sm" title="Clear (C)" onClick={clearNodes}>
            <Trash2 className="h-3 w-3" />
          </Button>
        </div>

        {lastHohmann && (
          <div className="rounded-md border border-amber-800/40 bg-amber-950/30 px-2 py-1.5 text-[10px] text-amber-200/80">
            Hohmann · Δv₁ {lastHohmann.dv_departure.toFixed(0)} · Δv₂{" "}
            {lastHohmann.dv_arrival.toFixed(0)} m/s · {formatTime(lastHohmann.transfer_time)}
            <Button
              variant="ghost"
              size="sm"
              className="mt-1 h-6 w-full text-amber-300"
              onClick={() => simAction(() => getWasmSim()?.add_hohmann_maneuver_pair())}
            >
              Shift+H · Add burn pair
            </Button>
          </div>
        )}

        {planner.nodes.length > 0 && (
          <div className="space-y-1 border-t border-slate-700/50 pt-2">
            <div className="text-[10px] font-semibold uppercase tracking-wider text-slate-500">
              Planned nodes
            </div>
            <ul className="max-h-32 space-y-1 overflow-y-auto">
              {planner.nodes.map((n, i) => {
                const dv = Math.hypot(n.prograde, n.normal, n.radial);
                const selected = selectedNodeIndex === i;
                return (
                  <li key={`${n.time}-${i}`}>
                    <button
                      type="button"
                      className={`flex w-full items-center justify-between rounded-md border px-2 py-1 text-left text-[10px] transition-colors ${
                        selected
                          ? "border-amber-600/60 bg-amber-950/40 text-amber-100"
                          : "border-slate-700/50 bg-slate-900/40 text-slate-400 hover:bg-slate-800/60"
                      } ${n.executed ? "opacity-45 line-through" : ""}`}
                      onClick={() => {
                        setSelectedNodeIndex(i);
                        simAction(() =>
                          getWasmSim()?.set_draft_delta_v(n.prograde, n.normal, n.radial),
                        );
                      }}
                    >
                      <span className="font-mono">MN{i + 1}</span>
                      <span>{formatTime(n.time)}</span>
                      <Badge variant="maneuver">{dv.toFixed(0)} m/s</Badge>
                    </button>
                  </li>
                );
              })}
            </ul>
            {selectedNodeIndex !== null && (
              <Button
                variant="destructive"
                size="sm"
                className="w-full"
                onClick={() => {
                  simAction(() => getWasmSim()?.remove_maneuver_node(selectedNodeIndex));
                  setSelectedNodeIndex(null);
                }}
              >
                Delete selected node
              </Button>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function addNode() {
  simAction(() => getWasmSim()?.add_maneuver_node());
  useSimStore.getState().setSelectedNodeIndex(null);
}

function clearNodes() {
  simAction(() => getWasmSim()?.clear_maneuver_nodes());
  useSimStore.getState().setSelectedNodeIndex(null);
}

function DvRow({
  label,
  color,
  value,
  onChange,
  onNudge,
}: {
  label: string;
  color: string;
  value: number;
  onChange: (v: number) => void;
  onNudge: (delta: number) => void;
}) {
  return (
    <div className="rounded-md border border-slate-700/50 bg-slate-900/50 p-2">
      <div className="mb-1 flex items-center gap-2">
        <span className="h-2 w-2 rounded-full" style={{ backgroundColor: color }} />
        <span className="text-[11px] font-medium text-slate-300">{label}</span>
      </div>
      <div className="flex items-center gap-1">
        <Button variant="secondary" size="icon" className="h-7 w-7 shrink-0" onClick={() => onNudge(-50)}>
          <ChevronDown className="h-3 w-3" />
        </Button>
        <input
          type="number"
          step={10}
          value={Math.round(value)}
          onChange={(e) => onChange(Number(e.target.value))}
          className="h-8 min-w-0 flex-1 rounded-md border border-slate-600 bg-slate-950 px-2 text-center font-mono text-sm text-slate-100 focus:border-sky-600 focus:outline-none focus:ring-1 focus:ring-sky-600/40"
        />
        <Button variant="secondary" size="icon" className="h-7 w-7 shrink-0" onClick={() => onNudge(50)}>
          <ChevronUp className="h-3 w-3" />
        </Button>
      </div>
      <div className="mt-0.5 text-right text-[10px] text-slate-500">m/s</div>
    </div>
  );
}
