import { useEffect, useRef } from "react";
import init, { WasmSimulation } from "../sim/pkg/sim_wasm";
import { fetchScenarioToml, SCENARIO_CATALOG } from "../lib/scenarios";
import { useSimStore } from "../store/simStore";
import type { BodySnapshot } from "../lib/units";
import { pickDefaultTarget } from "../lib/vessels";
import { sceneBodiesRef } from "./sceneRefs";
import { applyManeuverLayers, applyRafScene, applySceneCache } from "./sceneCache";
import { clearTextureCache } from "../scene/textureCache";
import type { SimCommand } from "./simCommands";

let wasmSim: WasmSimulation | null = null;
let wasmBusy = false;
let simGeneration = 0;
const wasmQueue: Array<() => void> = [];
let pendingFrame: { dt: number; now: number } | null = null;

function wasmErrorMessage(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "string") return err;
  return String(err);
}

function uiScalars() {
  const s = useSimStore.getState();
  return {
    paused: Boolean(s.paused),
    speed: Number.isFinite(s.speed) ? s.speed : 5000,
    targetBody: String(s.maneuverVessel ?? ""),
    showOrbits: Boolean(s.showOrbits),
    soiAuto: Boolean(s.soiAuto),
  };
}

export interface SyncPacket {
  scenario_name: string;
  sim_time: number;
  bodies: BodySnapshot[];
  diagnostics: import("../lib/units").SimDiagnostics;
  planner: import("../lib/units").PlannerState;
  orbit_paths: { name: string; flat: number[] }[];
  maneuver_preview: number[];
  maneuver_markers: number[];
  error?: string;
}

function runWasmExclusive(fn: () => void) {
  wasmQueue.push(fn);
  drainWasmQueue();
}

function scheduleFrame(dt: number, now: number) {
  pendingFrame = { dt, now };
  drainWasmQueue();
}

function drainWasmQueue() {
  if (wasmBusy) return;

  if (wasmQueue.length > 0) {
    wasmBusy = true;
    try {
      wasmQueue.shift()!();
    } finally {
      wasmBusy = false;
      drainWasmQueue();
    }
    return;
  }

  if (!pendingFrame || !wasmSim) return;

  const { dt, now } = pendingFrame;
  pendingFrame = null;
  wasmBusy = true;
  try {
    runDriveFrame(wasmSim, dt, now);
  } finally {
    wasmBusy = false;
    if (pendingFrame) drainWasmQueue();
  }
}

function parseSync(json: string): SyncPacket {
  try {
    const parsed: unknown = JSON.parse(json);
    if (!parsed || typeof parsed !== "object") {
      throw new Error("invalid sync packet");
    }
    return parsed as SyncPacket;
  } catch (err) {
    throw new Error(`WASM sync parse failed: ${wasmErrorMessage(err)}`);
  }
}

function applyHudFromPacket(packet: SyncPacket) {
  const store = useSimStore.getState();
  store.setSimTime(packet.sim_time);
  sceneBodiesRef.current = packet.bodies;
  store.setBodies(packet.bodies);
  store.setDiagnostics(packet.diagnostics);
  store.setPlanner(packet.planner);
  store.setLastHohmann(packet.planner.last_hohmann ?? null);
}

export type SimSyncMode = "full" | "planner" | "none";

export type SimActionOptions = {
  notice?: string;
  sync?: SimSyncMode;
  onComplete?: () => void;
};

function applySyncPacket(packet: SyncPacket, mode: SimSyncMode) {
  if (packet.error) {
    useSimStore.getState().setActionNotice(packet.error);
  }
  if (mode === "full") {
    if (packet.bodies) applyHudFromPacket(packet);
    applySceneCache({
      orbit_paths: packet.orbit_paths ?? [],
      maneuver_preview: packet.maneuver_preview ?? [],
      maneuver_markers: packet.maneuver_markers ?? [],
    });
  } else if (mode === "planner") {
    if (packet.planner) useSimStore.getState().setPlanner(packet.planner);
    applyManeuverLayers(packet.maneuver_preview ?? [], packet.maneuver_markers ?? []);
  }
}

function wasmDispatch(commands: SimCommand[]): SyncPacket | null {
  const sim = wasmSim;
  if (!sim) return null;
  const ui = uiScalars();
  const json = sim.dispatch(
    ui.paused,
    ui.speed,
    ui.targetBody,
    ui.showOrbits,
    ui.soiAuto,
    JSON.stringify(commands),
  );
  return parseSync(json);
}

function bootstrapSim(sim: WasmSimulation): SyncPacket {
  const ui = uiScalars();
  const json = sim.dispatch(ui.paused, ui.speed, ui.targetBody, ui.showOrbits, ui.soiAuto, "[]");
  return parseSync(json);
}

// HUD + orbit paths are cheap; refresh at ~12.5 Hz. The maneuver preview is a heavy
// N-body integration, so refresh it on a slower cadence to keep frames smooth.
const HUD_SYNC_INTERVAL_MS = 80;
const PREVIEW_SYNC_INTERVAL_MS = 250;

function runDriveFrame(sim: WasmSimulation, dt: number, now: number) {
  const gen = simGeneration;
  const wantScene = now - lastHudSyncRef.current >= HUD_SYNC_INTERVAL_MS;
  const wantPreview = wantScene && now - lastPreviewSyncRef.current >= PREVIEW_SYNC_INTERVAL_MS;
  const ui = uiScalars();
  const json = sim.drive_frame(
    dt,
    ui.paused,
    ui.speed,
    ui.targetBody,
    ui.showOrbits,
    ui.soiAuto,
    wantScene,
    wantPreview,
  );
  if (gen !== simGeneration) return;

  const packet = parseSync(json);
  if (packet.error) {
    useSimStore.getState().setSceneFault(packet.error);
    return;
  }
  if (useSimStore.getState().sceneFault) {
    useSimStore.getState().setSceneFault(null);
  }
  if (!packet.bodies?.length) return;

  sceneBodiesRef.current = packet.bodies;
  if (wantScene) {
    lastHudSyncRef.current = now;
    applyHudFromPacket(packet);
    if (wantPreview) lastPreviewSyncRef.current = now;
    applyRafScene(
      packet.orbit_paths ?? [],
      packet.maneuver_markers ?? [],
      wantPreview ? (packet.maneuver_preview ?? []) : undefined,
    );
  }
}

const lastHudSyncRef = { current: 0 };
const lastPreviewSyncRef = { current: 0 };

function applyScenarioTarget(target: string | null) {
  const store = useSimStore.getState();
  if (target) {
    store.selectManeuverVessel(target);
  } else {
    store.setManeuverVessel(null);
    store.setSelectedBody(null);
    store.setActionNotice("This scenario has no vessel — add a body with probe = true.");
  }
}

/** One WASM call per user action — never overlaps with RAF `drive_frame`. */
export function simDispatch(
  commands: SimCommand | SimCommand[],
  noticeOrOptions?: string | SimActionOptions,
) {
  const list = Array.isArray(commands) ? commands : [commands];
  const options: SimActionOptions =
    typeof noticeOrOptions === "string"
      ? { notice: noticeOrOptions, sync: "full" }
      : { sync: "full", ...noticeOrOptions };

  runWasmExclusive(() => {
    try {
      const packet = wasmDispatch(list);
      if (!packet) return;
      applySyncPacket(packet, options.sync ?? "full");
      if (options.notice && !packet.error) {
        useSimStore.getState().setActionNotice(options.notice);
      }
      options.onComplete?.();
    } catch (err) {
      useSimStore.getState().setSceneFault(wasmErrorMessage(err));
    }
  });
}

export function resetSimUiAfterReset() {
  const store = useSimStore.getState();
  store.setSelectedNodeIndex(null);
  store.setLastHohmann(null);
}

export async function reloadScenarioByIndex(index: number): Promise<void> {
  const gen = ++simGeneration;
  try {
    const { toml, entry } = await fetchScenarioToml(index);
    if (gen !== simGeneration) return;

    await new Promise<void>((resolve) => {
      runWasmExclusive(() => {
        if (gen !== simGeneration) {
          resolve();
          return;
        }
        try {
          clearTextureCache();
          const sim = WasmSimulation.from_scenario_toml(toml);
          wasmSim = sim;

          const packet = bootstrapSim(sim);
          applyHudFromPacket(packet);
          applySceneCache({
            orbit_paths: packet.orbit_paths ?? [],
            maneuver_preview: packet.maneuver_preview ?? [],
            maneuver_markers: packet.maneuver_markers ?? [],
          });

          const target = pickDefaultTarget(sceneBodiesRef.current, entry.defaultTarget);
          applyScenarioTarget(target);

          const store = useSimStore.getState();
          store.setScenarioName(packet.scenario_name);
          store.setScenarioIndex(index);
          store.setSelectedNodeIndex(null);
          store.setLastHohmann(null);
          store.setSceneFault(null);
        } catch (err) {
          useSimStore.getState().setActionNotice(wasmErrorMessage(err));
        } finally {
          resolve();
        }
      });
    });
  } catch (err) {
    useSimStore.getState().setActionNotice(wasmErrorMessage(err));
  }
}

export function useSimulationLoop() {
  const setReady = useSimStore((s) => s.setReady);
  const setError = useSimStore((s) => s.setError);
  const setScenarioName = useSimStore((s) => s.setScenarioName);
  const setScenarioIndex = useSimStore((s) => s.setScenarioIndex);
  const rafRef = useRef<number>(0);
  const lastRef = useRef<number>(0);

  useEffect(() => {
    let cancelled = false;
    const gen = ++simGeneration;
    (async () => {
      try {
        await init();
        if (cancelled || gen !== simGeneration) return;

        const sim = new WasmSimulation();
        wasmSim = sim;

        const packet = bootstrapSim(sim);
        applyHudFromPacket(packet);
        applySceneCache({
          orbit_paths: packet.orbit_paths ?? [],
          maneuver_preview: packet.maneuver_preview ?? [],
          maneuver_markers: packet.maneuver_markers ?? [],
        });

        const defaultTarget = SCENARIO_CATALOG[0]?.defaultTarget;
        const target = pickDefaultTarget(sceneBodiesRef.current, defaultTarget);
        applyScenarioTarget(target);

        setScenarioName(packet.scenario_name);
        setScenarioIndex(0);
        setReady(true);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    })();
    return () => {
      cancelled = true;
      simGeneration += 1;
      cancelAnimationFrame(rafRef.current);
      wasmSim = null;
      pendingFrame = null;
      wasmQueue.length = 0;
    };
  }, [setReady, setError, setScenarioName, setScenarioIndex]);

  useEffect(() => {
    const tick = (now: number) => {
      rafRef.current = requestAnimationFrame(tick);
      if (!wasmSim) return;

      const dt = lastRef.current ? (now - lastRef.current) / 1000 : 0;
      lastRef.current = now;
      if (dt <= 0 || dt >= 0.5) return;

      scheduleFrame(dt, now);
    };
    rafRef.current = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(rafRef.current);
  }, []);
}

export function resetSceneFault() {
  useSimStore.getState().setSceneFault(null);
}

export function retryScene() {
  resetSceneFault();
  useSimStore.getState().bumpSceneRemountKey();
}
