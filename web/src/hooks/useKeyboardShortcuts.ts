import { useEffect } from "react";
import { reloadScenarioByIndex, resetSimUiAfterReset, simDispatch } from "@/sim/useSimulation";
import { useSimStore } from "@/store/simStore";
import { warpDown, warpUp, WARP_LEVELS } from "@/lib/ksp";
import { isVessel } from "@/lib/vessels";
import { clearSelectedScratchNode, commandsAfterRemovingSelectedScratch } from "@/sim/maneuverNodeUx";

function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || target.isContentEditable;
}

function hasManeuverVessel(store: ReturnType<typeof useSimStore.getState>): boolean {
  const name = store.maneuverVessel;
  if (!name) return false;
  const body = store.bodies.find((b) => b.name === name);
  return body ? isVessel(body) : false;
}

export function useKeyboardShortcuts() {
  const ready = useSimStore((s) => s.ready);

  useEffect(() => {
    if (!ready) return;

    const onKeyDown = (e: KeyboardEvent) => {
      if (isTypingTarget(e.target)) return;

      const store = useSimStore.getState();

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
          clearSelectedScratchNode();
          store.cycleSelection(e.shiftKey);
          break;
        case "KeyG":
          store.setFollowSelection(!store.followSelection);
          break;
        case "KeyF":
          if (store.selectedBody) store.focusBody(store.selectedBody);
          break;
        case "KeyB":
          if (!hasManeuverVessel(store)) {
            store.setActionNotice("Select a vessel in View to plan maneuvers.");
            break;
          }
          simDispatch(
            commandsAfterRemovingSelectedScratch({ cmd: "add_node" }),
            {
              notice: "Draft maneuver node created — drag a handle or enter Δv to keep it",
              onComplete: () => {
                const n = useSimStore.getState().planner?.nodes.length ?? 0;
                if (n > 0) store.setSelectedNodeIndex(n - 1);
              },
            },
          );
          break;
        case "KeyC":
          simDispatch({ cmd: "clear_nodes" });
          store.setSelectedNodeIndex(null);
          break;
        case "KeyV":
          store.setShowOrbits(!store.showOrbits);
          break;
        case "KeyO":
          store.setSoiAuto(!store.soiAuto);
          break;
        case "KeyN":
          simDispatch({ cmd: "step_once" });
          break;
        case "KeyR":
          simDispatch({ cmd: "reset" });
          resetSimUiAfterReset();
          break;
        case "KeyW":
          if (store.selectedNodeIndex === null) {
            store.setActionNotice("Select a maneuver node to warp to it.");
            break;
          }
          simDispatch(
            { cmd: "warp_to_node", index: store.selectedNodeIndex, lead: 30 },
            { notice: "Warped to maneuver node" },
          );
          break;
        case "KeyH":
          if (!hasManeuverVessel(store)) {
            store.setActionNotice("Select a vessel in View for Hohmann planning.");
            break;
          }
          if (e.shiftKey) {
            simDispatch(commandsAfterRemovingSelectedScratch([{ cmd: "compute_hohmann" }, { cmd: "add_hohmann_pair" }]), {
              onComplete: () => {
                const n = useSimStore.getState().planner?.nodes.length ?? 0;
                if (n >= 2) store.setSelectedNodeIndex(n - 2);
              },
            });
          } else {
            simDispatch(
              commandsAfterRemovingSelectedScratch([
                { cmd: "compute_hohmann" },
                { cmd: "apply_hohmann_departure" },
              ]),
            );
          }
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
