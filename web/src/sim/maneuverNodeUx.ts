import type { ManeuverNode } from "@/lib/units";
import { useSimStore } from "@/store/simStore";
import type { SimCommand } from "./simCommands";
import { simDispatch } from "./useSimulation";

const SCRATCH_DV_EPS = 0.5;

export function isScratchNode(node: ManeuverNode | undefined): boolean {
  return Boolean(
    node &&
      !node.executed &&
      Math.hypot(node.prograde, node.normal, node.radial) <= SCRATCH_DV_EPS,
  );
}

export function selectedScratchNodeIndex(): number | null {
  const store = useSimStore.getState();
  const index = store.selectedNodeIndex;
  if (index === null) return null;
  return isScratchNode(store.planner?.nodes[index]) ? index : null;
}

export function commandsAfterRemovingSelectedScratch(
  commands: SimCommand | SimCommand[],
): SimCommand[] {
  const list = Array.isArray(commands) ? commands : [commands];
  const scratch = selectedScratchNodeIndex();
  return scratch === null ? list : [{ cmd: "remove_node", index: scratch }, ...list];
}

export function clearSelectedScratchNode() {
  const scratch = selectedScratchNodeIndex();
  if (scratch === null) return false;
  simDispatch(
    { cmd: "remove_node", index: scratch },
    {
      sync: "planner",
      onComplete: () => useSimStore.getState().setSelectedNodeIndex(null),
    },
  );
  return true;
}

export function clearOrDeselectSelectedNode() {
  if (!clearSelectedScratchNode()) {
    useSimStore.getState().setSelectedNodeIndex(null);
  }
}

export function selectNodeWithScratchCleanup(nextIndex: number | null) {
  const scratch = selectedScratchNodeIndex();
  if (scratch === null || nextIndex === scratch) {
    useSimStore.getState().setSelectedNodeIndex(nextIndex);
    return;
  }

  const adjustedNext =
    nextIndex !== null && scratch < nextIndex ? Math.max(0, nextIndex - 1) : nextIndex;

  simDispatch(
    { cmd: "remove_node", index: scratch },
    {
      sync: "planner",
      onComplete: () => useSimStore.getState().setSelectedNodeIndex(adjustedNext),
    },
  );
}
