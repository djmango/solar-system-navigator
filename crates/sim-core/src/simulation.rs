use std::collections::HashMap;

use glam::DVec3;

use crate::astro::{self, AU, DAY, DEFAULT_TIME_WARP, YEAR};
use crate::maneuver::{self, ManeuverNode};
use crate::orbit::{self, RelativeState};
use crate::physics::{BodyState, total_energy};
use crate::planner::{self, compute_soi_radius_for_body};
use crate::scenario::{DEFAULT_SCENARIO, Scenario, load_scenario_from_str};
use crate::soi::{self, SoiBodySnapshot};
use crate::transfer::HohmannTransfer;
use crate::truth::{self, build_truth_paths_for_scenario, predict_target_path_nbody};

#[derive(Debug, Clone, serde::Serialize)]
pub struct BodySnapshot {
    pub name: String,
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub mass: f64,
    pub radius: f64,
    pub display_radius: f64,
    pub color: [f64; 3],
    pub emissive: f64,
    pub atmosphere: Option<[f64; 3]>,
    pub texture: Option<String>,
    pub soi_radius: f64,
    pub fixed: bool,
    pub probe: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SimDiagnostics {
    pub kinetic_energy: f64,
    pub potential_energy: f64,
    pub total_energy: f64,
    pub body_count: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RoutePlannerState {
    pub nodes: Vec<ManeuverNode>,
    pub central_body: Option<String>,
    pub target_body: Option<String>,
    pub draft_prograde: f64,
    pub draft_normal: f64,
    pub draft_radial: f64,
    pub default_burn_offset: f64,
    pub preview_horizon: f64,
    pub preview_step: f64,
    pub show_previews: bool,
    pub soi_auto: bool,
    pub hohmann_target_radius: f64,
    pub last_hohmann: Option<HohmannTransfer>,
}

impl Default for RoutePlannerState {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            central_body: None,
            target_body: None,
            draft_prograde: 500.0,
            draft_normal: 0.0,
            draft_radial: 0.0,
            default_burn_offset: DAY,
            preview_horizon: YEAR,
            preview_step: 3600.0,
            show_previews: true,
            soi_auto: true,
            hohmann_target_radius: AU * 1.524,
            last_hohmann: None,
        }
    }
}

pub struct Simulation {
    pub scenario: Scenario,
    states: Vec<BodyState>,
    pub sim_time: f64,
    pub paused: bool,
    pub speed: f64,
    pub ticks_per_frame: u32,
    pub step_once: bool,
    pub planner: RoutePlannerState,
    pub diagnostics: SimDiagnostics,
    truth_paths: HashMap<String, Vec<DVec3>>,
    soi_radii: HashMap<String, f64>,
}

impl Simulation {
    pub fn from_scenario(scenario: Scenario) -> Result<Self, String> {
        crate::scenario::validate_circular_speeds(&scenario)?;
        let states = truth::states_from_scenario(&scenario);
        let soi_radii = compute_soi_radii(&scenario);
        let truth_paths = build_truth_paths_for_scenario(&scenario);
        let primary = scenario
            .bodies
            .iter()
            .find(|b| b.fixed)
            .map(|b| b.name.clone());
        let default_target = scenario
            .bodies
            .iter()
            .find(|b| b.probe)
            .or_else(|| scenario.bodies.iter().find(|b| !b.fixed))
            .map(|b| b.name.clone());

        let planner = RoutePlannerState {
            central_body: primary,
            target_body: default_target,
            ..RoutePlannerState::default()
        };

        Ok(Self {
            scenario,
            states,
            sim_time: 0.0,
            paused: false,
            speed: DEFAULT_TIME_WARP,
            ticks_per_frame: 4,
            step_once: false,
            planner,
            diagnostics: SimDiagnostics {
                kinetic_energy: 0.0,
                potential_energy: 0.0,
                total_energy: 0.0,
                body_count: 0,
            },
            truth_paths,
            soi_radii,
        })
    }

    pub fn from_scenario_toml(toml: &str) -> Result<Self, String> {
        Self::from_scenario(load_scenario_from_str(toml)?)
    }

    pub fn from_default_scenario() -> Result<Self, String> {
        Self::from_scenario_toml(DEFAULT_SCENARIO)
    }

    pub fn reset(&mut self) {
        self.states = truth::states_from_scenario(&self.scenario);
        self.sim_time = 0.0;
        maneuver::reset_maneuver_execution(&mut self.planner.nodes);
        self.planner.last_hohmann = None;
        self.truth_paths = build_truth_paths_for_scenario(&self.scenario);
    }

    pub fn body_snapshots(&self) -> Vec<BodySnapshot> {
        self.scenario
            .bodies
            .iter()
            .map(|def| {
                let state = self
                    .states
                    .iter()
                    .find(|s| s.name == def.name)
                    .expect("body state");
                BodySnapshot {
                    name: def.name.clone(),
                    position: state.position.to_array(),
                    velocity: state.velocity.to_array(),
                    mass: def.mass,
                    radius: def.radius,
                    display_radius: def.display_radius(self.scenario.visual_exaggeration),
                    color: def.color,
                    emissive: def.emissive,
                    atmosphere: def.atmosphere,
                    texture: def.texture.clone(),
                    soi_radius: self.soi_radii.get(&def.name).copied().unwrap_or(0.0),
                    fixed: def.fixed,
                    probe: def.probe,
                }
            })
            .collect()
    }

    pub fn truth_path(&self, name: &str) -> Option<&[DVec3]> {
        self.truth_paths.get(name).map(|v| v.as_slice())
    }

    pub fn truth_path_flat(&self, name: &str) -> Vec<f64> {
        let Some(path) = self.truth_paths.get(name) else {
            return Vec::new();
        };
        path.iter().flat_map(|p| [p.x, p.y, p.z]).collect()
    }

    pub fn soi_snapshots(&self) -> Vec<SoiBodySnapshot> {
        self.states
            .iter()
            .map(|s| {
                let is_primary = self
                    .scenario
                    .bodies
                    .iter()
                    .find(|b| b.name == s.name)
                    .is_some_and(|b| b.fixed);
                SoiBodySnapshot {
                    name: s.name.clone(),
                    position: s.position,
                    velocity: s.velocity,
                    mass: s.mass,
                    soi_radius: self.soi_radii.get(&s.name).copied().unwrap_or(0.0),
                    is_primary,
                }
            })
            .collect()
    }

    pub fn update_soi_central(&mut self) {
        if !self.planner.soi_auto {
            return;
        }
        let primary_name = self
            .scenario
            .bodies
            .iter()
            .find(|b| b.fixed)
            .map(|b| b.name.as_str())
            .unwrap_or("Sun");
        let snapshots = self.soi_snapshots();
        let Some(target_name) = self.planner.target_body.clone() else {
            return;
        };
        let Some(target) = self.states.iter().find(|s| s.name == target_name) else {
            return;
        };
        let central = soi::dominant_soi_body(target.position, &snapshots, primary_name);
        self.planner.central_body = Some(central.to_string());
    }

    /// Flat buffer: [sim_time, body_count, then per body: px,py,pz,vx,vy,vz]
    pub fn state_buffer(&self) -> Vec<f64> {
        let mut buf = vec![self.sim_time, self.states.len() as f64];
        for s in &self.states {
            buf.extend_from_slice(&[
                s.position.x,
                s.position.y,
                s.position.z,
                s.velocity.x,
                s.velocity.y,
                s.velocity.z,
            ]);
        }
        buf
    }

    pub fn step(&mut self, real_dt: f64) {
        if self.paused && !self.step_once {
            return;
        }
        self.step_once = false;

        let dt_base = real_dt * self.speed;
        let steps = self.ticks_per_frame.max(1);

        for _ in 0..steps {
            self.execute_maneuver_burns();
            self.update_diagnostics();
            self.states = truth::velocity_verlet_step(
                &self.states,
                self.scenario.g,
                self.scenario.softening,
                dt_base,
            );
            self.sim_time += dt_base;
        }

        self.update_soi_central();
    }

    fn update_diagnostics(&mut self) {
        let (ke, pe) = total_energy(&self.states, self.scenario.g, self.scenario.softening);
        self.diagnostics = SimDiagnostics {
            kinetic_energy: ke,
            potential_energy: pe,
            total_energy: ke + pe,
            body_count: self.states.len() as u32,
        };
    }

    fn execute_maneuver_burns(&mut self) {
        if self.planner.nodes.is_empty() {
            return;
        }
        let Some(central_name) = self.planner.central_body.clone() else {
            return;
        };
        let Some(central) = self.states.iter().find(|s| s.name == central_name) else {
            return;
        };
        let central_pos = central.position;
        let central_vel = central.velocity;

        let target_name = self.planner.target_body.clone().or_else(|| {
            self.states
                .iter()
                .find(|s| s.name != central_name)
                .map(|s| s.name.clone())
        });
        let Some(target_name) = target_name else {
            return;
        };
        let Some(target) = self.states.iter().find(|s| s.name == target_name) else {
            return;
        };
        let ship_pos = target.position;
        let mut rel_vel = target.velocity - central_vel;
        let mut any_burn = false;

        for node in &mut self.planner.nodes {
            if node.executed || self.sim_time < node.time {
                continue;
            }
            let rel = RelativeState::new(ship_pos - central_pos, rel_vel);
            rel_vel = orbit::apply_tnw_delta_v(rel, node.prograde, node.normal, node.radial);
            node.executed = true;
            any_burn = true;
        }

        if !any_burn {
            return;
        }

        let inertial = central_vel + rel_vel;
        if let Some(target) = self.states.iter_mut().find(|s| s.name == target_name) {
            target.velocity = inertial;
        }
    }

    pub fn add_maneuver_node(&mut self) {
        let t = self.sim_time + self.planner.default_burn_offset;
        self.add_maneuver_node_at_time(t);
    }

    pub fn add_maneuver_node_at_time(&mut self, time: f64) {
        let time = time.max(self.sim_time + 1.0);
        self.planner.nodes.push(ManeuverNode {
            time,
            prograde: self.planner.draft_prograde,
            normal: self.planner.draft_normal,
            radial: self.planner.draft_radial,
            executed: false,
        });
        self.planner.show_previews = true;
    }

    /// Place a maneuver node at the orbit point nearest to a world-space click (meters, SI).
    pub fn add_maneuver_node_at_world_position(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
    ) -> Result<f64, String> {
        let pos = DVec3::new(x, y, z);
        let time = if !self.planner.nodes.is_empty() {
            self.time_at_nearest_predicted_path(pos)
        } else {
            None
        }
        .or_else(|| self.time_at_nearest_target_orbit(pos))
        .ok_or_else(|| "could not map click to an orbit".to_string())?;
        self.add_maneuver_node_at_time(time);
        Ok(time)
    }

    pub fn predict_maneuver_path_timed(&self) -> Vec<f64> {
        let Some(central) = self.planner.central_body.clone() else {
            return Vec::new();
        };
        let Some(target) = self.planner.target_body.clone() else {
            return Vec::new();
        };
        if !self.planner.show_previews {
            return Vec::new();
        }
        let path = predict_target_path_nbody(
            &self.states,
            &central,
            &target,
            &self.planner.nodes,
            self.sim_time,
            self.planner.preview_horizon,
            self.planner.preview_step,
            self.scenario.g,
            self.scenario.softening,
        );
        let mut out = Vec::with_capacity(path.len() * 4);
        let mut t = self.sim_time;
        for p in path {
            out.extend_from_slice(&[t, p.x, p.y, p.z]);
            t += self.planner.preview_step;
        }
        out
    }

    pub fn target_orbit_timed_flat(&self, segments: usize) -> Vec<f64> {
        let Some(target_name) = self.planner.target_body.clone() else {
            return Vec::new();
        };
        let flat = self.orbit_preview_flat(&target_name, segments);
        if flat.len() < 6 {
            return Vec::new();
        }
        let period = self.target_orbital_period().unwrap_or(crate::astro::DAY);
        let n = flat.len() / 3;
        let mut out = Vec::with_capacity(n * 4);
        for i in 0..n {
            let t = self.sim_time + (i as f64 / n as f64) * period;
            out.extend_from_slice(&[t, flat[3 * i], flat[3 * i + 1], flat[3 * i + 2]]);
        }
        out
    }

    pub fn maneuver_node_markers_flat(&self) -> Vec<f64> {
        let mut out = Vec::new();
        for node in &self.planner.nodes {
            if node.executed {
                continue;
            }
            if let Some(p) = self.target_position_at_ut(node.time) {
                out.extend_from_slice(&[node.time, p.x, p.y, p.z]);
            }
        }
        out
    }

    fn target_orbital_period(&self) -> Option<f64> {
        let central_name = self.planner.central_body.clone()?;
        let target_name = self.planner.target_body.clone()?;
        let central = self.states.iter().find(|s| s.name == central_name)?;
        let target = self.states.iter().find(|s| s.name == target_name)?;
        Some(truth::estimate_orbital_period(
            target.position,
            central.position,
            central.mass,
            self.scenario.g,
        ))
    }

    fn target_position_at_ut(&self, ut: f64) -> Option<DVec3> {
        if ut < self.sim_time {
            return None;
        }
        let central = self.planner.central_body.clone()?;
        let target = self.planner.target_body.clone()?;
        let horizon =
            (ut - self.sim_time + self.planner.preview_step * 2.0).max(self.planner.preview_step);
        let path = predict_target_path_nbody(
            &self.states,
            &central,
            &target,
            &self.planner.nodes,
            self.sim_time,
            horizon,
            self.planner.preview_step,
            self.scenario.g,
            self.scenario.softening,
        );
        if path.is_empty() {
            return None;
        }
        let idx = ((ut - self.sim_time) / self.planner.preview_step).round() as usize;
        path.get(idx.min(path.len() - 1)).copied()
    }

    fn time_at_nearest_predicted_path(&self, world_pos: DVec3) -> Option<f64> {
        let central = self.planner.central_body.clone()?;
        let target = self.planner.target_body.clone()?;
        let path = predict_target_path_nbody(
            &self.states,
            &central,
            &target,
            &self.planner.nodes,
            self.sim_time,
            self.planner.preview_horizon,
            self.planner.preview_step,
            self.scenario.g,
            self.scenario.softening,
        );
        let (idx, _) = closest_point_index(&path, world_pos)?;
        Some(self.sim_time + idx as f64 * self.planner.preview_step)
    }

    fn time_at_nearest_target_orbit(&self, world_pos: DVec3) -> Option<f64> {
        let target_name = self.planner.target_body.clone()?;
        let segments = 128;
        let flat = self.orbit_preview_flat(&target_name, segments);
        if flat.len() < 6 {
            return None;
        }
        let n = flat.len() / 3;
        let mut best_i = 0;
        let mut best_d = f64::MAX;
        for i in 0..n {
            let p = DVec3::new(flat[3 * i], flat[3 * i + 1], flat[3 * i + 2]);
            let d = (p - world_pos).length_squared();
            if d < best_d {
                best_d = d;
                best_i = i;
            }
        }
        let period = self.target_orbital_period()?;
        Some(self.sim_time + (best_i as f64 / n as f64) * period)
    }

    pub fn clear_maneuver_nodes(&mut self) {
        self.planner.nodes.clear();
        self.planner.show_previews = false;
    }

    pub fn update_maneuver_node(
        &mut self,
        index: usize,
        prograde: f64,
        normal: f64,
        radial: f64,
    ) -> Result<(), String> {
        let node = self
            .planner
            .nodes
            .get_mut(index)
            .ok_or_else(|| format!("maneuver node {index} not found"))?;
        if node.executed {
            return Err("cannot edit executed maneuver node".into());
        }
        node.prograde = prograde;
        node.normal = normal;
        node.radial = radial;
        self.planner.show_previews = true;
        Ok(())
    }

    pub fn remove_maneuver_node(&mut self, index: usize) -> Result<(), String> {
        if index >= self.planner.nodes.len() {
            return Err(format!("maneuver node {index} not found"));
        }
        self.planner.nodes.remove(index);
        if self.planner.nodes.is_empty() {
            self.planner.show_previews = false;
        }
        Ok(())
    }

    pub fn set_default_burn_offset(&mut self, offset: f64) {
        self.planner.default_burn_offset = offset.max(1.0);
    }

    pub fn set_show_previews(&mut self, show: bool) {
        self.planner.show_previews = show;
    }

    pub fn set_soi_auto(&mut self, auto: bool) {
        self.planner.soi_auto = auto;
    }

    pub fn compute_hohmann(&mut self) -> Option<HohmannTransfer> {
        let snapshots = self.soi_snapshots();
        let central_name = self.planner.central_body.clone()?;
        let target_name = self.planner.target_body.clone()?;
        let xfer = planner::compute_hohmann_for_target(
            &snapshots,
            &central_name,
            &target_name,
            self.scenario.g,
            self.planner.hohmann_target_radius,
        )?;
        self.planner.last_hohmann = Some(xfer);
        Some(xfer)
    }

    pub fn apply_hohmann_departure_draft(&mut self, xfer: &HohmannTransfer) {
        self.planner.draft_prograde = xfer.dv_departure;
        self.planner.draft_normal = 0.0;
        self.planner.draft_radial = 0.0;
    }

    pub fn add_hohmann_maneuver_pair(&mut self, xfer: &HohmannTransfer) {
        let t0 = self.sim_time + self.planner.default_burn_offset;
        let t1 = t0 + xfer.transfer_time;
        self.planner.nodes.push(ManeuverNode {
            time: t0,
            prograde: xfer.dv_departure,
            normal: 0.0,
            radial: 0.0,
            executed: false,
        });
        self.planner.nodes.push(ManeuverNode {
            time: t1,
            prograde: xfer.dv_arrival,
            normal: 0.0,
            radial: 0.0,
            executed: false,
        });
        self.planner.show_previews = true;
    }

    pub fn predict_maneuver_path(&self) -> Vec<f64> {
        if !self.planner.show_previews || self.planner.nodes.is_empty() {
            return Vec::new();
        }
        let Some(central) = self.planner.central_body.clone() else {
            return Vec::new();
        };
        let Some(target) = self.planner.target_body.clone() else {
            return Vec::new();
        };
        let path = predict_target_path_nbody(
            &self.states,
            &central,
            &target,
            &self.planner.nodes,
            self.sim_time,
            self.planner.preview_horizon,
            self.planner.preview_step,
            self.scenario.g,
            self.scenario.softening,
        );
        path.iter().flat_map(|p| [p.x, p.y, p.z]).collect()
    }

    pub fn relative_target_state(&self) -> Option<RelativeState> {
        let central = self.planner.central_body.as_deref()?;
        let target = self.planner.target_body.as_deref()?;
        planner::relative_target_state(&self.soi_snapshots(), central, target)
    }

    pub fn orbit_preview_flat(&self, body_name: &str, segments: usize) -> Vec<f64> {
        let Some(state) = self.states.iter().find(|s| s.name == body_name) else {
            return Vec::new();
        };
        let central_name = self
            .scenario
            .bodies
            .iter()
            .find(|b| b.fixed)
            .map(|b| b.name.as_str())
            .unwrap_or("Sun");
        let central = self.states.iter().find(|s| s.name == central_name);
        let (pos, vel, mu) = if let Some(c) = central {
            let rel = RelativeState::new(state.position - c.position, state.velocity - c.velocity);
            (rel.position, rel.velocity, self.scenario.g * c.mass)
        } else {
            (
                state.position,
                state.velocity,
                self.scenario.g * astro::M_SUN,
            )
        };
        let rel = RelativeState::new(pos, vel);
        orbit::sample_orbit_path(mu, rel, segments)
            .into_iter()
            .flat_map(|p| {
                if let Some(c) = central {
                    let world = c.position + p;
                    [world.x, world.y, world.z]
                } else {
                    [p.x, p.y, p.z]
                }
            })
            .collect()
    }
}

fn closest_point_index(path: &[DVec3], world_pos: DVec3) -> Option<(usize, f64)> {
    if path.is_empty() {
        return None;
    }
    let mut best_i = 0;
    let mut best_d = f64::MAX;
    for (i, p) in path.iter().enumerate() {
        let d = (*p - world_pos).length_squared();
        if d < best_d {
            best_d = d;
            best_i = i;
        }
    }
    Some((best_i, best_d.sqrt()))
}

fn compute_soi_radii(scenario: &Scenario) -> HashMap<String, f64> {
    let primary = scenario.bodies.iter().find(|b| b.fixed);
    let (primary_mass, primary_pos) = primary
        .map(|p| (p.mass, p.position_vec3()))
        .unwrap_or((astro::M_SUN, DVec3::ZERO));

    scenario
        .bodies
        .iter()
        .map(|def| {
            (
                def.name.clone(),
                compute_soi_radius_for_body(def, scenario, primary_mass, primary_pos),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_simulation_advances() {
        let mut sim = Simulation::from_default_scenario().expect("load");
        let t0 = sim.sim_time;
        sim.step(1.0 / 60.0);
        assert!(sim.sim_time > t0);
    }

    #[test]
    fn state_buffer_has_expected_length() {
        let sim = Simulation::from_default_scenario().expect("load");
        let buf = sim.state_buffer();
        assert_eq!(buf.len(), 2 + sim.states.len() * 6);
    }
}
