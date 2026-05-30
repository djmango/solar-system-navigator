use std::cell::RefCell;

use serde::{Deserialize, Serialize};
use sim_core::simulation::{BodySnapshot, RoutePlannerState, SimDiagnostics, Simulation};
use sim_core::transfer::HohmannTransfer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

struct UiState {
    paused: bool,
    speed: f64,
    target_body: String,
    show_orbits: bool,
    soi_auto: bool,
}

impl UiState {
    fn from_scalars(
        paused: bool,
        speed: f64,
        target_body: &str,
        show_orbits: bool,
        soi_auto: bool,
    ) -> Self {
        Self {
            paused,
            speed: if speed.is_finite() { speed } else { 5000.0 },
            target_body: target_body.to_string(),
            show_orbits,
            soi_auto,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
enum SimCommand {
    AddNode,
    AddNodeAtWorld {
        x: f64,
        y: f64,
        z: f64,
    },
    ClearNodes,
    UpdateNode {
        index: u32,
        prograde: f64,
        normal: f64,
        radial: f64,
    },
    RemoveNode {
        index: u32,
    },
    SetDraftDv {
        prograde: f64,
        normal: f64,
        radial: f64,
    },
    Reset,
    StepOnce,
    ComputeHohmann,
    ApplyHohmannDeparture,
    AddHohmannPair,
}

#[derive(Serialize)]
struct OrbitPathEntry {
    name: String,
    flat: Vec<f64>,
}

#[derive(Serialize)]
struct SyncPacket {
    scenario_name: String,
    sim_time: f64,
    bodies: Vec<BodySnapshot>,
    diagnostics: SimDiagnostics,
    planner: RoutePlannerState,
    orbit_paths: Vec<OrbitPathEntry>,
    maneuver_preview: Vec<f64>,
    maneuver_markers: Vec<f64>,
    error: Option<String>,
}

#[wasm_bindgen]
pub struct WasmSimulation {
    inner: RefCell<Simulation>,
}

impl WasmSimulation {
    fn apply_ui(sim: &mut Simulation, ui: &UiState) {
        sim.paused = ui.paused;
        sim.speed = ui.speed;
        if !ui.target_body.is_empty() {
            sim.planner.target_body = Some(ui.target_body.clone());
        }
        sim.set_soi_auto(ui.soi_auto);
        let node_count = sim.planner.nodes.len();
        sim.set_show_previews(ui.show_orbits || node_count > 0);
    }

    fn orbit_paths_for(sim: &Simulation, show_orbits: bool) -> Vec<OrbitPathEntry> {
        if !show_orbits {
            return Vec::new();
        }
        sim.body_snapshots()
            .into_iter()
            .map(|body| {
                let flat = sim.truth_path_flat(&body.name);
                let data = if flat.len() >= 6 {
                    flat
                } else {
                    sim.orbit_preview_flat(&body.name, 96)
                };
                OrbitPathEntry {
                    name: body.name,
                    flat: data,
                }
            })
            .collect()
    }

    fn maneuver_preview_for(sim: &Simulation) -> Vec<f64> {
        // The preview is an N-body integration over a 1-year horizon. It is only
        // meaningful relative to planned burns — without nodes it merely retraces the
        // orbit line, so skip it to avoid integrating thousands of steps every sync.
        if !sim.planner.show_previews || sim.planner.nodes.is_empty() {
            return Vec::new();
        }
        let timed = sim.predict_maneuver_path_timed();
        if timed.len() >= 8 {
            timed
                .chunks(4)
                .flat_map(|chunk| [chunk[1], chunk[2], chunk[3]])
                .collect()
        } else {
            sim.predict_maneuver_path()
        }
    }

    /// `want_scene` gates orbit-path / marker rebuilds; `want_preview` separately gates
    /// the much heavier N-body maneuver-preview integration. The RAF loop applies these
    /// on throttled cadences (and preview slower than scene), so skipping them on most
    /// frames removes the bulk of per-frame CPU + JSON cost.
    fn sync_packet(
        sim: &Simulation,
        show_orbits: bool,
        want_scene: bool,
        want_preview: bool,
    ) -> SyncPacket {
        let orbit_paths = if want_scene {
            Self::orbit_paths_for(sim, show_orbits)
        } else {
            Vec::new()
        };
        let maneuver_markers = if want_scene {
            sim.maneuver_node_markers_flat()
        } else {
            Vec::new()
        };
        let maneuver_preview = if want_preview {
            Self::maneuver_preview_for(sim)
        } else {
            Vec::new()
        };
        SyncPacket {
            scenario_name: sim.scenario.name.clone(),
            sim_time: sim.sim_time,
            bodies: sim.body_snapshots(),
            diagnostics: sim.diagnostics.clone(),
            planner: sim.planner.clone(),
            orbit_paths,
            maneuver_preview,
            maneuver_markers,
            error: None,
        }
    }

    fn run_command(sim: &mut Simulation, command: SimCommand) -> Result<(), String> {
        match command {
            SimCommand::AddNode => {
                sim.add_maneuver_node();
                Ok(())
            }
            SimCommand::AddNodeAtWorld { x, y, z } => {
                sim.add_maneuver_node_at_world_position(x, y, z).map(|_| ())
            }
            SimCommand::ClearNodes => {
                sim.clear_maneuver_nodes();
                Ok(())
            }
            SimCommand::UpdateNode {
                index,
                prograde,
                normal,
                radial,
            } => sim.update_maneuver_node(index as usize, prograde, normal, radial),
            SimCommand::RemoveNode { index } => sim.remove_maneuver_node(index as usize),
            SimCommand::SetDraftDv {
                prograde,
                normal,
                radial,
            } => {
                sim.planner.draft_prograde = prograde;
                sim.planner.draft_normal = normal;
                sim.planner.draft_radial = radial;
                Ok(())
            }
            SimCommand::Reset => {
                sim.reset();
                Ok(())
            }
            SimCommand::StepOnce => {
                sim.step_once = true;
                sim.step(1.0 / 60.0);
                Ok(())
            }
            SimCommand::ComputeHohmann => match sim.compute_hohmann() {
                Some(xfer) => {
                    sim.planner.last_hohmann = Some(xfer);
                    Ok(())
                }
                None => Err("Could not compute Hohmann transfer".into()),
            },
            SimCommand::ApplyHohmannDeparture => {
                let xfer: HohmannTransfer = sim
                    .planner
                    .last_hohmann
                    .ok_or_else(|| "No Hohmann computed".to_string())?;
                sim.apply_hohmann_departure_draft(&xfer);
                Ok(())
            }
            SimCommand::AddHohmannPair => {
                let xfer = sim
                    .planner
                    .last_hohmann
                    .ok_or_else(|| "No Hohmann computed".to_string())?;
                sim.add_hohmann_maneuver_pair(&xfer);
                Ok(())
            }
        }
    }

    fn sanitize_json_numbers(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Number(n) => {
                if let Some(f) = n.as_f64()
                    && !f.is_finite()
                {
                    *value = serde_json::json!(0.0);
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    Self::sanitize_json_numbers(item);
                }
            }
            serde_json::Value::Object(map) => {
                for item in map.values_mut() {
                    Self::sanitize_json_numbers(item);
                }
            }
            _ => {}
        }
    }

    fn to_json(packet: SyncPacket) -> String {
        match serde_json::to_value(&packet) {
            Ok(mut value) => {
                Self::sanitize_json_numbers(&mut value);
                serde_json::to_string(&value)
                    .unwrap_or_else(|err| format!("{{\"error\":\"serialization failed: {err}\"}}"))
            }
            Err(err) => format!("{{\"error\":\"serialization failed: {err}\"}}"),
        }
    }
}

#[wasm_bindgen]
impl WasmSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmSimulation, JsValue> {
        Simulation::from_default_scenario()
            .map(|inner| Self {
                inner: RefCell::new(inner),
            })
            .map_err(|e| JsValue::from_str(&e))
    }

    pub fn from_scenario_toml(toml: &str) -> Result<WasmSimulation, JsValue> {
        Simulation::from_scenario_toml(toml)
            .map(|inner| Self {
                inner: RefCell::new(inner),
            })
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Apply UI state, advance physics, return one JSON sync packet. **Only RAF entry.**
    /// Pass `want_scene = false` on most frames to skip orbit-path / preview rebuilds.
    #[allow(clippy::too_many_arguments)]
    pub fn drive_frame(
        &self,
        real_dt: f64,
        paused: bool,
        speed: f64,
        target_body: &str,
        show_orbits: bool,
        soi_auto: bool,
        want_scene: bool,
        want_preview: bool,
    ) -> String {
        let ui = UiState::from_scalars(paused, speed, target_body, show_orbits, soi_auto);
        let mut sim = self.inner.borrow_mut();
        Self::apply_ui(&mut sim, &ui);
        if real_dt > 0.0 && real_dt < 0.5 && !ui.paused {
            sim.step(real_dt);
        }
        Self::to_json(Self::sync_packet(
            &sim,
            ui.show_orbits,
            want_scene,
            want_preview,
        ))
    }

    /// Apply UI + user commands, return one JSON sync packet. **Only interaction entry.**
    pub fn dispatch(
        &self,
        paused: bool,
        speed: f64,
        target_body: &str,
        show_orbits: bool,
        soi_auto: bool,
        commands_json: &str,
    ) -> String {
        let commands: Vec<SimCommand> = match serde_json::from_str(commands_json) {
            Ok(commands) => commands,
            Err(err) => {
                return Self::to_json(SyncPacket {
                    scenario_name: String::new(),
                    sim_time: 0.0,
                    bodies: Vec::new(),
                    diagnostics: SimDiagnostics {
                        kinetic_energy: 0.0,
                        potential_energy: 0.0,
                        total_energy: 0.0,
                        body_count: 0,
                    },
                    planner: RoutePlannerState::default(),
                    orbit_paths: Vec::new(),
                    maneuver_preview: Vec::new(),
                    maneuver_markers: Vec::new(),
                    error: Some(format!("bad commands json: {err}")),
                });
            }
        };

        let ui = UiState::from_scalars(paused, speed, target_body, show_orbits, soi_auto);
        let mut sim = self.inner.borrow_mut();
        Self::apply_ui(&mut sim, &ui);
        for command in commands {
            if let Err(err) = Self::run_command(&mut sim, command) {
                let packet = Self::sync_packet(&sim, ui.show_orbits, true, true);
                return Self::to_json(SyncPacket {
                    error: Some(err),
                    ..packet
                });
            }
        }
        Self::to_json(Self::sync_packet(&sim, ui.show_orbits, true, true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_add_node_command() {
        let cmd: SimCommand = serde_json::from_str(r#"{"cmd":"add_node"}"#).unwrap();
        assert!(matches!(cmd, SimCommand::AddNode));
    }

    #[test]
    fn parse_command_batch() {
        let cmds: Vec<SimCommand> =
            serde_json::from_str(r#"[{"cmd":"reset"},{"cmd":"step_once"}]"#).unwrap();
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn empty_target_body_skipped_in_apply_ui() {
        let mut sim = Simulation::from_default_scenario().unwrap();
        sim.planner.target_body = Some("Scout".into());
        let ui = UiState::from_scalars(false, 5000.0, "", true, true);
        WasmSimulation::apply_ui(&mut sim, &ui);
        assert_eq!(sim.planner.target_body.as_deref(), Some("Scout"));
    }
}
