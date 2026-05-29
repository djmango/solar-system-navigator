import { create } from "zustand";
import type {
  BodySnapshot,
  HohmannTransfer,
  PlannerState,
  SimDiagnostics,
} from "../lib/units";

export interface SimStore {
  ready: boolean;
  error: string | null;
  scenarioName: string;
  scenarioIndex: number;
  simTime: number;
  paused: boolean;
  speed: number;
  bodies: BodySnapshot[];
  selectedBody: string | null;
  selectedNodeIndex: number | null;
  showOrbits: boolean;
  followSelection: boolean;
  soiAuto: boolean;
  /** Incremented to trigger camera re-frame on the focused body. */
  frameRequest: number;
  showShortcuts: boolean;
  diagnostics: SimDiagnostics | null;
  planner: PlannerState | null;
  lastHohmann: HohmannTransfer | null;
  setReady: (ready: boolean) => void;
  setError: (error: string | null) => void;
  setScenarioName: (name: string) => void;
  setScenarioIndex: (index: number) => void;
  setSimTime: (t: number) => void;
  setPaused: (paused: boolean) => void;
  setSpeed: (speed: number) => void;
  setBodies: (bodies: BodySnapshot[]) => void;
  setSelectedBody: (name: string | null) => void;
  setSelectedNodeIndex: (index: number | null) => void;
  setShowOrbits: (show: boolean) => void;
  setFollowSelection: (follow: boolean) => void;
  setSoiAuto: (auto: boolean) => void;
  setShowShortcuts: (show: boolean) => void;
  setDiagnostics: (d: SimDiagnostics) => void;
  setPlanner: (p: PlannerState) => void;
  setLastHohmann: (h: HohmannTransfer | null) => void;
  /** Select body and animate camera to frame it (double-click / F key). */
  focusBody: (name: string) => void;
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
  selectedBody: "Earth",
  selectedNodeIndex: null,
  showOrbits: true,
  followSelection: true,
  soiAuto: true,
  frameRequest: 0,
  showShortcuts: false,
  diagnostics: null,
  planner: null,
  lastHohmann: null,
  setReady: (ready) => set({ ready }),
  setError: (error) => set({ error }),
  setScenarioName: (scenarioName) => set({ scenarioName }),
  setScenarioIndex: (scenarioIndex) => set({ scenarioIndex }),
  setSimTime: (simTime) => set({ simTime }),
  setPaused: (paused) => set({ paused }),
  setSpeed: (speed) => set({ speed }),
  setBodies: (bodies) => set({ bodies }),
  setSelectedBody: (selectedBody) => set({ selectedBody }),
  setSelectedNodeIndex: (selectedNodeIndex) => set({ selectedNodeIndex }),
  setShowOrbits: (showOrbits) => set({ showOrbits }),
  setFollowSelection: (followSelection) => set({ followSelection }),
  setSoiAuto: (soiAuto) => set({ soiAuto }),
  setShowShortcuts: (showShortcuts) => set({ showShortcuts }),
  setDiagnostics: (diagnostics) => set({ diagnostics }),
  setPlanner: (planner) => set({ planner }),
  setLastHohmann: (lastHohmann) => set({ lastHohmann }),
  focusBody: (name) =>
    set((s) => ({
      selectedBody: name,
      followSelection: true,
      frameRequest: s.frameRequest + 1,
    })),
  cycleSelection: (reverse = false) => {
    const { bodies, selectedBody } = get();
    const names = bodies.filter((b) => !b.fixed).map((b) => b.name);
    if (names.length === 0) return;
    const idx = selectedBody ? names.indexOf(selectedBody) : -1;
    const next =
      idx < 0
        ? names[0]
        : names[(idx + (reverse ? -1 : 1) + names.length) % names.length];
    get().focusBody(next);
  },
}));
