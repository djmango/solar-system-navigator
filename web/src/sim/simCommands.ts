/** Commands sent to WASM `dispatch` — one JSON round-trip per user action. */
export type SimCommand =
  | { cmd: "add_node" }
  | { cmd: "add_node_at_world"; x: number; y: number; z: number }
  | { cmd: "clear_nodes" }
  | { cmd: "update_node"; index: number; prograde: number; normal: number; radial: number }
  | { cmd: "remove_node"; index: number }
  | { cmd: "set_draft_dv"; prograde: number; normal: number; radial: number }
  | { cmd: "reset" }
  | { cmd: "step_once" }
  | { cmd: "compute_hohmann" }
  | { cmd: "apply_hohmann_departure" }
  | { cmd: "add_hohmann_pair" };
