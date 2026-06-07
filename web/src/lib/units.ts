export const AU = 149_597_870_700;
export const DAY = 86_400;

/** Render scale: 1 Three.js unit = 1 AU */
export const METERS_PER_UNIT = AU;

export function toScene(meters: number): number {
  return meters / METERS_PER_UNIT;
}

export function toSceneVec(flat: number[], i: number): [number, number, number] {
  return [toScene(flat[i]), toScene(flat[i + 1]), toScene(flat[i + 2])];
}

export function formatTime(seconds: number): string {
  const days = seconds / DAY;
  if (days >= 365) return `${(days / 365.25).toFixed(2)} yr`;
  if (days >= 1) return `${days.toFixed(1)} d`;
  const hrs = seconds / 3600;
  if (hrs >= 1) return `${hrs.toFixed(1)} h`;
  return `${seconds.toFixed(0)} s`;
}

export function formatSpeed(mps: number): string {
  if (mps >= 1000) return `${(mps / 1000).toFixed(2)} km/s`;
  return `${mps.toFixed(0)} m/s`;
}

export function formatEnergy(joules: number): string {
  const abs = Math.abs(joules);
  if (abs >= 1e30) return `${(joules / 1e30).toFixed(3)}×10³⁰ J`;
  if (abs >= 1e24) return `${(joules / 1e24).toFixed(3)}×10²⁴ J`;
  return `${joules.toExponential(2)} J`;
}

export interface BodySnapshot {
  name: string;
  position: [number, number, number];
  velocity: [number, number, number];
  mass: number;
  radius: number;
  display_radius: number;
  color: [number, number, number];
  emissive: number;
  atmosphere?: [number, number, number];
  texture?: string;
  soi_radius: number;
  fixed: boolean;
  probe: boolean;
}

export interface ManeuverNode {
  time: number;
  prograde: number;
  normal: number;
  radial: number;
  executed: boolean;
}

export interface PlannerState {
  nodes: ManeuverNode[];
  central_body: string | null;
  target_body: string | null;
  draft_prograde: number;
  draft_normal: number;
  draft_radial: number;
  default_burn_offset: number;
  preview_horizon: number;
  preview_step: number;
  show_previews: boolean;
  soi_auto: boolean;
  hohmann_target_radius: number;
  last_hohmann: HohmannTransfer | null;
  hohmann_warning?: string | null;
}

export interface HohmannTransfer {
  r1: number;
  r2: number;
  dv_departure: number;
  dv_arrival: number;
  transfer_time: number;
}

export interface SimDiagnostics {
  kinetic_energy: number;
  potential_energy: number;
  total_energy: number;
  body_count: number;
}
