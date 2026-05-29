import { useSimStore } from "@/store/simStore";
import { formatEnergy, formatSpeed, formatTime } from "@/lib/units";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

export function VesselHud() {
  const selectedBody = useSimStore((s) => s.selectedBody);
  const bodies = useSimStore((s) => s.bodies);
  const planner = useSimStore((s) => s.planner);
  const simTime = useSimStore((s) => s.simTime);
  const diagnostics = useSimStore((s) => s.diagnostics);

  const body = bodies.find((b) => b.name === selectedBody);
  const speedMag = body
    ? Math.hypot(body.velocity[0], body.velocity[1], body.velocity[2])
    : 0;

  const nextNode = planner?.nodes.find((n) => !n.executed && n.time >= simTime);

  return (
    <div className="pointer-events-none absolute bottom-3 left-3 flex flex-col gap-2 sm:bottom-4 sm:left-4">
      <Card className="w-56 border-slate-600/50 sm:w-64">
        <CardContent className="space-y-1.5 py-2">
          {body ? (
            <>
              <div className="flex items-center justify-between gap-2">
                <span className="text-sm font-semibold text-sky-200">{body.name}</span>
                {body.probe && <Badge variant="maneuver">Vessel</Badge>}
              </div>
              <Row label="Velocity" value={formatSpeed(speedMag)} />
              <Row label="SOI" value={planner?.central_body ?? "—"} />
              {nextNode && (
                <Row
                  label="Next node"
                  value={`${formatTime(nextNode.time - simTime)} · ${Math.hypot(nextNode.prograde, nextNode.normal, nextNode.radial).toFixed(0)} m/s`}
                  accent
                />
              )}
            </>
          ) : (
            <p className="text-xs text-slate-500">Click a body to select</p>
          )}
        </CardContent>
      </Card>

      {diagnostics && (
        <Card className="w-56 border-slate-600/50 sm:w-64">
          <CardContent className="py-2 text-[10px] text-slate-500">
            System energy {formatEnergy(diagnostics.total_energy)}
          </CardContent>
        </Card>
      )}
    </div>
  );
}

function Row({ label, value, accent }: { label: string; value: string; accent?: boolean }) {
  return (
    <div className="flex justify-between gap-2 text-[11px]">
      <span className="text-slate-500">{label}</span>
      <span className={accent ? "font-mono text-amber-300/90" : "font-mono text-slate-300"}>
        {value}
      </span>
    </div>
  );
}
