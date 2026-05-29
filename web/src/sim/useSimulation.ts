import { useEffect, useRef } from "react";
import init, { WasmSimulation } from "../sim/pkg/sim_wasm";
import { fetchScenarioToml } from "../lib/scenarios";
import { useSimStore } from "../store/simStore";
import type { BodySnapshot, PlannerState, SimDiagnostics } from "../lib/units";
import { bumpOrbitPaths, sceneBodiesRef } from "./sceneRefs";
import { clearTextureCache } from "../scene/textureCache";

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

  clearTextureCache();
  const sim = WasmSimulation.from_scenario_toml(toml);
  wasmSim = sim;
  sim.set_paused(paused);
  sim.set_speed(speed);
  sim.set_soi_auto(soiAuto);
  sim.set_target_body(entry.defaultTarget);
  syncFromWasm(sim, true);
  bumpOrbitPaths();
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
  const paused = useSimStore((s) => s.paused);
  const speed = useSimStore((s) => s.speed);
  const selectedBody = useSimStore((s) => s.selectedBody);
  const rafRef = useRef<number>(0);
  const lastRef = useRef<number>(0);
  const lastHudSync = useRef<number>(0);

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
        syncFromWasm(sim, true);
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
  }, [setReady, setError, setScenarioName, setScenarioIndex]);

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
        // Always update ref for Three.js useFrame (no React cost).
        sceneBodiesRef.current = wasmSim.body_snapshots() as BodySnapshot[];
        // Throttle React/HUD updates to ~12 Hz.
        if (now - lastHudSync.current > 80) {
          lastHudSync.current = now;
          syncFromWasm(wasmSim, false);
        }
      }
    };
    rafRef.current = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(rafRef.current);
  }, []);
}

function syncFromWasm(sim: WasmSimulation, forceBodies: boolean) {
  const store = useSimStore.getState();
  store.setSimTime(sim.sim_time());
  if (forceBodies) {
    const bodies = sim.body_snapshots() as BodySnapshot[];
    sceneBodiesRef.current = bodies;
    store.setBodies(bodies);
  }
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
  sceneBodiesRef.current = wasmSim.body_snapshots() as BodySnapshot[];
  const store = useSimStore.getState();
  store.setSimTime(wasmSim.sim_time());
  store.setBodies(sceneBodiesRef.current);
  store.setDiagnostics(wasmSim.diagnostics() as SimDiagnostics);
  store.setPlanner(wasmSim.planner_state() as PlannerState);
  bumpOrbitPaths();
}
