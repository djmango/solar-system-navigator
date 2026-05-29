import { useEffect } from "react";
import { getWasmSim, reloadScenarioByIndex, simAction } from "@/sim/useSimulation";
import { SCENARIO_CATALOG } from "@/lib/scenarios";
import { useSimStore } from "@/store/simStore";
import { warpDown, warpUp, WARP_LEVELS } from "@/lib/ksp";

function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || target.isContentEditable;
}

export function useKeyboardShortcuts() {
  const ready = useSimStore((s) => s.ready);

  useEffect(() => {
    if (!ready) return;

    const onKeyDown = (e: KeyboardEvent) => {
      if (isTypingTarget(e.target)) return;

      const store = useSimStore.getState();
      const sim = getWasmSim();

      switch (e.code) {
        case "Space":
          e.preventDefault();
          store.setPaused(!store.paused);
          break;
        case "Comma":
          e.preventDefault();
          store.setSpeed(warpDown(store.speed));
          break;
        case "Period":
          e.preventDefault();
          store.setSpeed(warpUp(store.speed));
          break;
        case "Equal":
        case "NumpadAdd":
          e.preventDefault();
          store.setSpeed(Math.min(store.speed * 1.25, WARP_LEVELS[WARP_LEVELS.length - 1]));
          break;
        case "Minus":
        case "NumpadSubtract":
          e.preventDefault();
          store.setSpeed(Math.max(store.speed / 1.25, 1));
          break;
        case "Tab":
          e.preventDefault();
          store.cycleSelection(e.shiftKey);
          simAction(() => sim?.set_target_body(useSimStore.getState().selectedBody ?? "Earth"));
          break;
        case "KeyG":
          store.setFollowSelection(!store.followSelection);
          break;
        case "KeyF":
          if (store.selectedBody) store.focusBody(store.selectedBody);
          break;
        case "KeyB":
          simAction(() => sim?.add_maneuver_node());
          break;
        case "KeyC":
          simAction(() => sim?.clear_maneuver_nodes());
          store.setSelectedNodeIndex(null);
          break;
        case "KeyV": {
          const next = !store.showOrbits;
          store.setShowOrbits(next);
          simAction(() => {
            const s = getWasmSim();
            if (!s) return;
            const p = s.planner_state() as { nodes: unknown[] };
            s.set_show_previews(next || p.nodes.length > 0);
          });
          break;
        }
        case "KeyO":
          store.setSoiAuto(!store.soiAuto);
          simAction(() => sim?.set_soi_auto(useSimStore.getState().soiAuto));
          break;
        case "KeyN":
          simAction(() => sim?.step_once());
          break;
        case "KeyR":
          simAction(() => sim?.reset());
          store.setSelectedNodeIndex(null);
          break;
        case "KeyH":
          simAction(() => {
            const xfer = sim?.compute_hohmann();
            if (xfer) store.setLastHohmann(xfer as never);
            if (e.shiftKey && xfer) sim?.add_hohmann_maneuver_pair();
            else if (xfer) sim?.apply_hohmann_departure_draft();
          });
          break;
        case "Slash":
          if (e.shiftKey) {
            store.setShowShortcuts(!store.showShortcuts);
          }
          break;
        case "Digit1":
          e.preventDefault();
          void reloadScenarioByIndex(0);
          break;
        case "Digit2":
          e.preventDefault();
          void reloadScenarioByIndex(1);
          break;
        case "Digit3":
          e.preventDefault();
          void reloadScenarioByIndex(2);
          break;
        case "Question":
          store.setShowShortcuts(!store.showShortcuts);
          break;
        default:
          break;
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [ready]);
}
