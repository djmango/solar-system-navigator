import { useEffect, useRef } from "react";
import init, { WasmSimulation } from "../sim/pkg/sim_wasm";
import { fetchScenarioToml } from "../lib/scenarios";
import { useSimStore } from "../store/simStore";
import type { BodySnapshot, PlannerState, SimDiagnostics } from "../lib/units";

let wasmSim: WasmSimulation | null = null;

export function getWasmSim(): WasmSimulation | null {
  return wasmSim;
}

export async function reloadScenarioByIndex(index: number): Promise<void> {
  const { toml, entry } = await fetchScenarioToml(index);
  const store = useSimStore.getState();
  const paused = store.paused;
  const speed = store.speed;
  const soiAuto = store.soiAuto;

  const sim = WasmSimulation.from_scenario_toml(toml);
  wasmSim = sim;
  sim.set_paused(paused);
  sim.set_speed(speed);
  sim.set_soi_auto(soiAuto);
  sim.set_target_body(entry.defaultTarget);
  syncFromWasm(sim);
  store.setScenarioName(sim.scenario_name());
  store.setScenarioIndex(index);
  store.setSelectedBody(entry.defaultTarget);
  store.setSelectedNodeIndex(null);
  store.setLastHohmann(null);
  store.focusBody(entry.defaultTarget);
}

export function useSimulationLoop() {
  const setReady = useSimStore((s) => s.setReady);
  const setError = useSimStore((s) => s.setError);
  const setScenarioName = useSimStore((s) => s.setScenarioName);
  const setScenarioIndex = useSimStore((s) => s.setScenarioIndex);
  const setSimTime = useSimStore((s) => s.setSimTime);
  const setBodies = useSimStore((s) => s.setBodies);
  const setDiagnostics = useSimStore((s) => s.setDiagnostics);
  const setPlanner = useSimStore((s) => s.setPlanner);
  const paused = useSimStore((s) => s.paused);
  const speed = useSimStore((s) => s.speed);
  const selectedBody = useSimStore((s) => s.selectedBody);
  const rafRef = useRef<number>(0);
  const lastRef = useRef<number>(0);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        await init();
        const sim = new WasmSimulation();
        wasmSim = sim;
        if (cancelled) return;
        setScenarioName(sim.scenario_name());
        setScenarioIndex(0);
        syncFromWasm(sim);
        sim.set_target_body("Earth");
        sim.set_soi_auto(true);
        useSimStore.getState().focusBody("Earth");
        setReady(true);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    })();
    return () => {
      cancelled = true;
      wasmSim = null;
    };
  }, [setReady, setError, setScenarioName, setScenarioIndex, setBodies, setSimTime, setDiagnostics, setPlanner]);

  useEffect(() => {
    if (!wasmSim || !selectedBody) return;
    wasmSim.set_target_body(selectedBody);
  }, [selectedBody]);

  useEffect(() => {
    if (!wasmSim) return;
    wasmSim.set_paused(paused);
  }, [paused]);

  useEffect(() => {
    if (!wasmSim) return;
    wasmSim.set_speed(speed);
  }, [speed]);

  useEffect(() => {
    const tick = (now: number) => {
      rafRef.current = requestAnimationFrame(tick);
      if (!wasmSim) return;
      const dt = lastRef.current ? (now - lastRef.current) / 1000 : 0;
      lastRef.current = now;
      if (dt > 0 && dt < 0.5) {
        wasmSim.step(dt);
        syncFromWasm(wasmSim);
      }
    };
    rafRef.current = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(rafRef.current);
  }, [setBodies, setSimTime, setDiagnostics, setPlanner]);
}

function syncFromWasm(sim: WasmSimulation) {
  const store = useSimStore.getState();
  store.setSimTime(sim.sim_time());
  store.setBodies(sim.body_snapshots() as BodySnapshot[]);
  store.setDiagnostics(sim.diagnostics() as SimDiagnostics);
  const planner = sim.planner_state() as PlannerState;
  store.setPlanner(planner);
  if (planner.last_hohmann) {
    store.setLastHohmann(planner.last_hohmann);
  }
}

export function simAction(action: () => void) {
  if (!wasmSim) return;
  action();
  const store = useSimStore.getState();
  store.setSimTime(wasmSim.sim_time());
  store.setBodies(wasmSim.body_snapshots() as BodySnapshot[]);
  store.setDiagnostics(wasmSim.diagnostics() as SimDiagnostics);
  store.setPlanner(wasmSim.planner_state() as PlannerState);
}
