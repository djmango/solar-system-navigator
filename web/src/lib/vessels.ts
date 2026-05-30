import type { BodySnapshot } from "./units";

export function isVessel(body: BodySnapshot): boolean {
  return body.probe;
}

export function vesselBodies(bodies: BodySnapshot[]): BodySnapshot[] {
  return bodies.filter(isVessel);
}

export function firstVesselName(bodies: BodySnapshot[]): string | null {
  return vesselBodies(bodies)[0]?.name ?? null;
}

export function pickDefaultTarget(bodies: BodySnapshot[], preferred?: string): string | null {
  if (preferred && bodies.some((b) => b.name === preferred && b.probe)) {
    return preferred;
  }
  return firstVesselName(bodies);
}
