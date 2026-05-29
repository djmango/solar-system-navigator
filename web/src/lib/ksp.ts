/** KSP-style discrete warp levels (sim seconds per real second). */
export const WARP_LEVELS = [
  0,
  1,
  4,
  10,
  50,
  100,
  1_000,
  5_000,
  10_000,
  50_000,
  100_000,
  500_000,
] as const;

export function nearestWarpIndex(speed: number): number {
  let best = 0;
  let bestDist = Infinity;
  for (let i = 0; i < WARP_LEVELS.length; i++) {
    const d = Math.abs(WARP_LEVELS[i] - speed);
    if (d < bestDist) {
      bestDist = d;
      best = i;
    }
  }
  return best;
}

export function warpUp(current: number): number {
  const i = nearestWarpIndex(current);
  return WARP_LEVELS[Math.min(i + 1, WARP_LEVELS.length - 1)];
}

export function warpDown(current: number): number {
  const i = nearestWarpIndex(current);
  return WARP_LEVELS[Math.max(i - 1, 0)];
}

export function formatWarp(speed: number): string {
  if (speed <= 0) return "Paused";
  if (speed < 1000) return `×${speed}`;
  if (speed < 1_000_000) return `×${(speed / 1000).toFixed(speed % 1000 === 0 ? 0 : 1)}k`;
  return `×${(speed / 1_000_000).toFixed(1)}M`;
}

export const KSP_SHORTCUTS = [
  { keys: "Space", action: "Pause / resume" },
  { keys: ",  .", action: "Decrease / increase warp" },
  { keys: "-  +", action: "Fine-tune warp" },
  { keys: "Tab", action: "Cycle vessel focus" },
  { keys: "G", action: "Toggle camera follow" },
  { keys: "F", action: "Frame selected body" },
  { keys: "B", action: "Add maneuver node" },
  { keys: "C", action: "Clear maneuver nodes" },
  { keys: "H", action: "Hohmann Δv draft (Shift+H = add pair)" },
  { keys: "V", action: "Toggle orbit paths" },
  { keys: "N", action: "Single physics step" },
  { keys: "R", action: "Reset simulation" },
  { keys: "1  2  3", action: "Switch scenario" },
  { keys: "Click orbit", action: "Place maneuver node at UT" },
  { keys: "Drag handles", action: "Adjust Δv on vessel gizmo" },
] as const;

export const KSP_COLORS = {
  prograde: "#4ade80",
  normal: "#c084fc",
  radial: "#60a5fa",
  maneuver: "#fbbf24",
  panel: "rgba(15, 23, 42, 0.88)",
  border: "rgba(71, 85, 105, 0.55)",
} as const;
