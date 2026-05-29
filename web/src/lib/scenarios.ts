export interface ScenarioEntry {
  id: number;
  label: string;
  path: string;
  defaultTarget: string;
}

export const SCENARIO_CATALOG: ScenarioEntry[] = [
  {
    id: 0,
    label: "Inner System (SI)",
    path: "/assets/scenarios/default.toml",
    defaultTarget: "Earth",
  },
  {
    id: 1,
    label: "Apollo 11",
    path: "/assets/missions/apollo11.toml",
    defaultTarget: "CSM",
  },
  {
    id: 2,
    label: "OSIRIS-REx",
    path: "/assets/missions/osiris_rex.toml",
    defaultTarget: "OSIRIS-REx",
  },
];

export async function fetchScenarioToml(index: number): Promise<{ toml: string; entry: ScenarioEntry }> {
  const entry = SCENARIO_CATALOG[index];
  if (!entry) {
    throw new Error(`Unknown scenario index ${index}`);
  }
  const res = await fetch(entry.path);
  if (!res.ok) {
    throw new Error(`Failed to load ${entry.path}: ${res.status}`);
  }
  const toml = await res.text();
  return { toml, entry };
}
