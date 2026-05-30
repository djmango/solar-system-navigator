import { create } from "zustand";
import type {
  BodySnapshot,
  HohmannTransfer,
  PlannerState,
  SimDiagnostics,
} from "../lib/units";
import { vesselBodies } from "../lib/vessels";

export interface SimStore {
  ready: boolean;
  error: string | null;
  scenarioName: string;
  scenarioIndex: number;
  simTime: number;
  paused: boolean;
  speed: number;
  bodies: BodySnapshot[];
  /** Camera focus + selection ring (any body). */
  selectedBody: string | null;
  /** Active vessel for maneuvers / WASM target (probe bodies only). */
  maneuverVessel: string | null;
  selectedNodeIndex: number | null;
  showOrbits: boolean;
  followSelection: boolean;
  soiAuto: boolean;
  frameRequest: number;
  showShortcuts: boolean;
  diagnostics: SimDiagnostics | null;
  planner: PlannerState | null;
  lastHohmann: HohmannTransfer | null;
  actionNotice: string | null;
  sceneFault: string | null;
  sceneCacheEpoch: number;
  /** Bumps to remount the Three.js tree after recoverable faults. */
  sceneRemountKey: number;
  setReady: (ready: boolean) => void;
  setError: (error: string | null) => void;
  setScenarioName: (name: string) => void;
  setScenarioIndex: (index: number) => void;
  setSimTime: (t: number) => void;
  setPaused: (paused: boolean) => void;
  setSpeed: (speed: number) => void;
  setBodies: (bodies: BodySnapshot[]) => void;
  setSelectedBody: (name: string | null) => void;
  setManeuverVessel: (name: string | null) => void;
  setSelectedNodeIndex: (index: number | null) => void;
  setShowOrbits: (show: boolean) => void;
  setFollowSelection: (follow: boolean) => void;
  setSoiAuto: (auto: boolean) => void;
  setShowShortcuts: (show: boolean) => void;
  setDiagnostics: (d: SimDiagnostics) => void;
  setPlanner: (p: PlannerState) => void;
  setLastHohmann: (h: HohmannTransfer | null) => void;
  setActionNotice: (message: string | null) => void;
  setSceneFault: (message: string | null) => void;
  bumpSceneCacheEpoch: () => void;
  bumpSceneRemountKey: () => void;
  focusBody: (name: string) => void;
  selectManeuverVessel: (name: string) => void;
  cycleSelection: (reverse?: boolean) => void;
}

export const useSimStore = create<SimStore>((set, get) => ({
  ready: false,
  error: null,
  scenarioName: "",
  scenarioIndex: 0,
  simTime: 0,
  paused: false,
  speed: 5000,
  bodies: [],
  selectedBody: null,
  maneuverVessel: null,
  selectedNodeIndex: null,
  showOrbits: true,
  followSelection: true,
  soiAuto: true,
  frameRequest: 0,
  showShortcuts: false,
  diagnostics: null,
  planner: null,
  lastHohmann: null,
  actionNotice: null,
  sceneFault: null,
  sceneCacheEpoch: 0,
  sceneRemountKey: 0,
  setReady: (ready) => set({ ready }),
  setError: (error) => set({ error }),
  setScenarioName: (scenarioName) => set({ scenarioName }),
  setScenarioIndex: (scenarioIndex) => set({ scenarioIndex }),
  setSimTime: (simTime) => set({ simTime }),
  setPaused: (paused) => set({ paused }),
  setSpeed: (speed) => set({ speed }),
  setBodies: (bodies) => set({ bodies }),
  setSelectedBody: (selectedBody) => set({ selectedBody }),
  setManeuverVessel: (maneuverVessel) => set({ maneuverVessel }),
  setSelectedNodeIndex: (selectedNodeIndex) => set({ selectedNodeIndex }),
  setShowOrbits: (showOrbits) => set({ showOrbits }),
  setFollowSelection: (followSelection) => set({ followSelection }),
  setSoiAuto: (soiAuto) => set({ soiAuto }),
  setShowShortcuts: (showShortcuts) => set({ showShortcuts }),
  setDiagnostics: (diagnostics) => set({ diagnostics }),
  setPlanner: (planner) => set({ planner }),
  setLastHohmann: (lastHohmann) => set({ lastHohmann }),
  setActionNotice: (actionNotice) => set({ actionNotice }),
  setSceneFault: (sceneFault) => set({ sceneFault }),
  bumpSceneCacheEpoch: () => set((s) => ({ sceneCacheEpoch: s.sceneCacheEpoch + 1 })),
  bumpSceneRemountKey: () => set((s) => ({ sceneRemountKey: s.sceneRemountKey + 1 })),
  focusBody: (name) =>
    set((s) => ({
      selectedBody: name,
      followSelection: true,
      frameRequest: s.frameRequest + 1,
    })),
  selectManeuverVessel: (name) => {
    set({ maneuverVessel: name });
    get().focusBody(name);
  },
  cycleSelection: (reverse = false) => {
    const { bodies, maneuverVessel } = get();
    const names = vesselBodies(bodies).map((b) => b.name);
    if (names.length === 0) return;
    const idx = maneuverVessel ? names.indexOf(maneuverVessel) : -1;
    const next =
      idx < 0
        ? names[0]
        : names[(idx + (reverse ? -1 : 1) + names.length) % names.length];
    get().selectManeuverVessel(next);
  },
}));
