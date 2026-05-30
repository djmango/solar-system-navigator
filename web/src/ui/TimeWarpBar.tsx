import { Gauge, Pause, Play, RotateCcw, StepForward } from "lucide-react";
import { resetSimUiAfterReset, simDispatch } from "@/sim/useSimulation";
import { useSimStore } from "@/store/simStore";
import { formatWarp, WARP_LEVELS } from "@/lib/ksp";
import { formatTime } from "@/lib/units";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Slider } from "@/components/ui/slider";

export function TimeWarpBar() {
  const paused = useSimStore((s) => s.paused);
  const setPaused = useSimStore((s) => s.setPaused);
  const speed = useSimStore((s) => s.speed);
  const setSpeed = useSimStore((s) => s.setSpeed);
  const simTime = useSimStore((s) => s.simTime);
  const scenarioName = useSimStore((s) => s.scenarioName);
  const followSelection = useSimStore((s) => s.followSelection);

  const warpIdx = WARP_LEVELS.findIndex((w) => w === speed);
  const sliderVal = warpIdx >= 0 ? warpIdx : 0;

  return (
    <Card className="pointer-events-auto absolute left-3 right-3 top-3 mx-auto max-w-3xl border-slate-600/50 sm:left-4 sm:right-auto sm:w-[32rem]">
      <CardContent className="flex flex-wrap items-center gap-2 py-2">
        <div className="min-w-0 flex-1">
          <div className="truncate text-sm font-semibold text-slate-100">Solar System Navigator</div>
          <div className="truncate text-[10px] text-slate-500">{scenarioName}</div>
        </div>

        <Badge variant="muted" className="font-mono">
          MET {formatTime(simTime)}
        </Badge>

        <Badge variant={paused ? "muted" : "warp"} className="gap-1">
          <Gauge className="h-3 w-3" />
          {formatWarp(paused ? 0 : speed)}
        </Badge>

        {followSelection && (
          <Badge variant="default" className="hidden sm:inline-flex">
            Follow
          </Badge>
        )}

        <div className="flex items-center gap-1">
          <Button
            size="icon"
            variant="secondary"
            title="Pause (Space)"
            onClick={() => setPaused(!paused)}
          >
            {paused ? <Play className="h-3.5 w-3.5" /> : <Pause className="h-3.5 w-3.5" />}
          </Button>
          <Button
            size="icon"
            variant="secondary"
            title="Step (N)"
            onClick={() => simDispatch({ cmd: "step_once" })}
          >
            <StepForward className="h-3.5 w-3.5" />
          </Button>
          <Button
            size="icon"
            variant="secondary"
            title="Reset (R)"
            onClick={() => {
              simDispatch({ cmd: "reset" });
              resetSimUiAfterReset();
            }}
          >
            <RotateCcw className="h-3.5 w-3.5" />
          </Button>
        </div>

        <div className="w-full min-w-[10rem] flex-1 sm:w-auto">
          <Slider
            min={0}
            max={WARP_LEVELS.length - 1}
            step={1}
            value={[sliderVal]}
            onValueChange={([v]) => {
              const w = WARP_LEVELS[v ?? 0];
              setSpeed(w);
              if (w === 0) setPaused(true);
              else setPaused(false);
            }}
          />
        </div>
      </CardContent>
    </Card>
  );
}
