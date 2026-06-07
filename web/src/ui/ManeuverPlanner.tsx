import { ChevronDown, ChevronUp, Plus, Rocket, Trash2, Zap } from "lucide-react";
import { useEffect, useState } from "react";
import { simDispatch } from "@/sim/useSimulation";
import { useSimStore } from "@/store/simStore";
import { formatTime } from "@/lib/units";
import { KSP_COLORS } from "@/lib/ksp";
import { isVessel } from "@/lib/vessels";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

export function ManeuverPlanner() {
  const planner = useSimStore((s) => s.planner);
  const simTime = useSimStore((s) => s.simTime);
  const targetPeriod = useSimStore((s) => s.targetPeriod);
  const selectedNodeIndex = useSimStore((s) => s.selectedNodeIndex);
  const setSelectedNodeIndex = useSimStore((s) => s.setSelectedNodeIndex);
  const lastHohmann = useSimStore((s) => s.lastHohmann);
  const maneuverVessel = useSimStore((s) => s.maneuverVessel);
  const bodies = useSimStore((s) => s.bodies);
  const actionNotice = useSimStore((s) => s.actionNotice);
  const setActionNotice = useSimStore((s) => s.setActionNotice);

  if (!planner) return null;

  const target = bodies.find((b) => b.name === maneuverVessel);
  const canPlan = target ? isVessel(target) : false;
  const vesselHint = bodies.find((b) => isVessel(b))?.name ?? "vessel";

  // An executed node is finalized — don't drive the editor from it (edits would
  // be rejected by the sim); fall back to the draft for the next node.
  const editing =
    selectedNodeIndex !== null &&
    planner.nodes[selectedNodeIndex] &&
    !planner.nodes[selectedNodeIndex].executed
      ? planner.nodes[selectedNodeIndex]
      : null;

  const prograde = editing?.prograde ?? planner.draft_prograde;
  const normal = editing?.normal ?? planner.draft_normal;
  const radial = editing?.radial ?? planner.draft_radial;
  const totalDv = Math.hypot(prograde, normal, radial);
  const burnUt = editing?.time ?? simTime + planner.default_burn_offset;

  const setDv = (p: number, n: number, r: number) => {
    if (selectedNodeIndex !== null) {
      simDispatch(
        { cmd: "update_node", index: selectedNodeIndex, prograde: p, normal: n, radial: r },
        { sync: "planner" },
      );
    } else {
      simDispatch({ cmd: "set_draft_dv", prograde: p, normal: n, radial: r }, { sync: "planner" });
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
        {editing && selectedNodeIndex !== null && (
          <NodeTimeSlider
            lead={Math.max(0, burnUt - simTime)}
            maxLead={Math.max(targetPeriod * 2, burnUt - simTime, 3600)}
            onCommit={(lead) =>
              simDispatch(
                { cmd: "set_node_time", index: selectedNodeIndex, time: simTime + lead },
                { sync: "planner" },
              )
            }
          />
        )}
        <p className="text-[10px] leading-relaxed text-slate-500">
          {canPlan
            ? `Click ${target?.name ?? vesselHint}'s orbit to place a node · drag handles for Δv`
            : `Select a vessel (${vesselHint}, …) in View → Vessel to plan burns`}
        </p>

        {actionNotice && (
          <div className="flex items-start justify-between gap-2 rounded-md border border-sky-800/50 bg-sky-950/40 px-2 py-1.5 text-[10px] text-sky-200">
            <span>{actionNotice}</span>
            <button
              type="button"
              className="shrink-0 text-sky-400 hover:text-sky-200"
              onClick={() => setActionNotice(null)}
            >
              ×
            </button>
          </div>
        )}

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
          <Button
            variant="maneuver"
            size="sm"
            className="flex-1"
            title="Add node (B)"
            disabled={!canPlan}
            onClick={addNode}
          >
            <Plus className="h-3 w-3" />
            Create node
          </Button>
          <Button
            variant="secondary"
            size="sm"
            title="Hohmann (H)"
            onClick={() =>
              simDispatch([
                { cmd: "compute_hohmann" },
                { cmd: "apply_hohmann_departure" },
              ])
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
              onClick={() =>
                simDispatch(
                  { cmd: "add_hohmann_pair" },
                  {
                    onComplete: () => {
                      const n = useSimStore.getState().planner?.nodes.length ?? 0;
                      if (n >= 2) useSimStore.getState().setSelectedNodeIndex(n - 2);
                    },
                  },
                )
              }
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
                        simDispatch({
                          cmd: "set_draft_dv",
                          prograde: n.prograde,
                          normal: n.normal,
                          radial: n.radial,
                        });
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
                  simDispatch({ cmd: "remove_node", index: selectedNodeIndex });
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
  simDispatch(
    { cmd: "add_node" },
    {
      notice: "Maneuver node created",
      onComplete: () => {
        const n = useSimStore.getState().planner?.nodes.length ?? 0;
        if (n > 0) useSimStore.getState().setSelectedNodeIndex(n - 1);
      },
    },
  );
}

function clearNodes() {
  simDispatch({ cmd: "clear_nodes" });
  useSimStore.getState().setSelectedNodeIndex(null);
}

function NodeTimeSlider({
  lead,
  maxLead,
  onCommit,
}: {
  lead: number;
  maxLead: number;
  onCommit: (lead: number) => void;
}) {
  // Local value during drag; commit (WASM round-trip + preview integration) on release.
  const [local, setLocal] = useState(lead);
  useEffect(() => setLocal(lead), [lead]);

  const commit = (v: number) => onCommit(v);

  return (
    <div className="rounded-md border border-slate-700/50 bg-slate-900/50 p-2">
      <div className="mb-1 flex items-center justify-between">
        <span className="text-[11px] font-medium text-slate-300">Slide node along orbit</span>
        <span className="font-mono text-[10px] text-amber-300/90">{formatTime(local)}</span>
      </div>
      <input
        type="range"
        min={0}
        max={Math.max(maxLead, 1)}
        step={Math.max(maxLead / 400, 1)}
        value={Math.min(local, maxLead)}
        onChange={(e) => setLocal(Number(e.target.value))}
        onPointerUp={() => commit(local)}
        onKeyUp={() => commit(local)}
        className="h-2 w-full cursor-pointer accent-amber-400"
      />
      <div className="mt-0.5 flex justify-between text-[9px] text-slate-600">
        <span>now</span>
        <span>{formatTime(maxLead)}</span>
      </div>
    </div>
  );
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
  const [local, setLocal] = useState(String(Math.round(value)));

  useEffect(() => {
    setLocal(String(Math.round(value)));
  }, [value]);

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
          value={local}
          onChange={(e) => setLocal(e.target.value)}
          onBlur={() => onChange(Number(local) || 0)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              onChange(Number(local) || 0);
              (e.target as HTMLInputElement).blur();
            }
          }}
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
