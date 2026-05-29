//! N-body truth paths — same velocity Verlet integrator as the live simulation.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::orbit::{self, RelativeState};
use crate::physics::{BodyState, gravitational_acceleration};
use crate::resources::ManeuverNode;
use crate::scenario::Scenario;

/// Target samples per body when pre-integrating reference orbits at scenario load.
#[cfg(target_arch = "wasm32")]
pub const TRUTH_PATH_SAMPLES: usize = 256;
#[cfg(not(target_arch = "wasm32"))]
pub const TRUTH_PATH_SAMPLES: usize = 512;
/// Fixed substep for offline / preview integration [s] (sim time).
pub const TRUTH_INTEGRATION_DT: f32 = 600.0;
#[cfg(target_arch = "wasm32")]
const MAX_INTEGRATION_STEPS: usize = 12_000;
#[cfg(not(target_arch = "wasm32"))]
const MAX_INTEGRATION_STEPS: usize = 25_000;

/// One velocity Verlet substep — shared by gameplay physics and truth previews.
pub fn velocity_verlet_step(
    states: &[BodyState],
    g: f32,
    softening: f32,
    dt: f32,
) -> Vec<BodyState> {
    let mut accelerations = Vec::with_capacity(states.len());
    for (index, state) in states.iter().enumerate() {
        let accel = if state.fixed {
            Vec3::ZERO
        } else {
            gravitational_acceleration(state.position, Some(index), states, g, softening)
        };
        accelerations.push(accel);
    }

    let mut new_positions = Vec::with_capacity(states.len());
    let mut new_velocities = Vec::with_capacity(states.len());
    let half_dt = dt * 0.5;

    for (index, state) in states.iter().enumerate() {
        if state.fixed {
            new_positions.push(state.position);
            new_velocities.push(state.velocity);
            continue;
        }
        let v_half = state.velocity + accelerations[index] * half_dt;
        new_positions.push(state.position + v_half * dt);
        new_velocities.push(v_half);
    }

    let mid: Vec<BodyState> = states
        .iter()
        .enumerate()
        .map(|(i, s)| BodyState {
            name: s.name.clone(),
            position: new_positions[i],
            velocity: new_velocities[i],
            mass: s.mass,
            fixed: s.fixed,
        })
        .collect();

    let mut accelerations_end = Vec::with_capacity(mid.len());
    for (index, state) in mid.iter().enumerate() {
        let accel = if state.fixed {
            Vec3::ZERO
        } else {
            gravitational_acceleration(state.position, Some(index), &mid, g, softening)
        };
        accelerations_end.push(accel);
    }

    mid.iter()
        .enumerate()
        .map(|(i, s)| BodyState {
            name: s.name.clone(),
            position: s.position,
            velocity: if s.fixed {
                s.velocity
            } else {
                new_velocities[i] + accelerations_end[i] * half_dt
            },
            mass: s.mass,
            fixed: s.fixed,
        })
        .collect()
}

pub fn states_from_scenario(scenario: &Scenario) -> Vec<BodyState> {
    scenario
        .bodies
        .iter()
        .map(|b| BodyState {
            name: b.name.clone(),
            position: b.position_vec3(),
            velocity: b.velocity_vec3(),
            mass: b.mass,
            fixed: b.fixed,
        })
        .collect()
}

/// Two-body orbital period around the primary [s].
pub fn estimate_orbital_period(
    position: Vec3,
    primary_position: Vec3,
    primary_mass: f32,
    g: f32,
) -> f32 {
    let r = (position - primary_position).length().max(1.0);
    let mu = g * primary_mass;
    if mu <= 0.0 {
        return crate::astro::DAY;
    }
    std::f32::consts::TAU * (r.powi(3) / mu).sqrt()
}

pub fn integration_duration_for_scenario(scenario: &Scenario) -> f32 {
    let primary = scenario.bodies.iter().find(|b| b.fixed);
    let (primary_mass, primary_pos) = primary
        .map(|p| (p.mass, p.position_vec3()))
        .unwrap_or((crate::astro::M_SUN, Vec3::ZERO));

    let mut max_period = crate::astro::DAY;
    for body in &scenario.bodies {
        if body.fixed {
            continue;
        }
        let t =
            estimate_orbital_period(body.position_vec3(), primary_pos, primary_mass, scenario.g);
        max_period = max_period.max(t);
    }
    max_period.min(2.0 * crate::astro::YEAR)
}

/// Integrate the full N-body system and record inertial positions (same physics as gameplay).
pub fn integrate_truth_paths(
    initial: &[BodyState],
    g: f32,
    softening: f32,
    duration: f32,
    dt: f32,
    target_samples: usize,
) -> HashMap<String, Vec<Vec3>> {
    let mut paths: HashMap<String, Vec<Vec3>> = initial
        .iter()
        .map(|b| (b.name.clone(), vec![b.position]))
        .collect();

    let steps = ((duration / dt).ceil() as usize).clamp(1, MAX_INTEGRATION_STEPS);
    let sample_every = (steps / target_samples.max(1)).max(1);

    let mut state: Vec<BodyState> = initial.to_vec();
    for step in 0..steps {
        if step > 0 && step % sample_every == 0 {
            for body in &state {
                if let Some(path) = paths.get_mut(&body.name) {
                    path.push(body.position);
                }
            }
        }
        state = velocity_verlet_step(&state, g, softening, dt);
    }

    for body in &state {
        if let Some(path) = paths.get_mut(&body.name)
            && path.last().copied() != Some(body.position)
        {
            path.push(body.position);
        }
    }

    paths
}

pub fn build_truth_paths_for_scenario(scenario: &Scenario) -> HashMap<String, Vec<Vec3>> {
    let initial = states_from_scenario(scenario);
    let duration = integration_duration_for_scenario(scenario);
    info!(
        "Integrating N-body truth orbits for '{}' ({:.1} days of sim time)…",
        scenario.name,
        duration / crate::astro::DAY
    );
    integrate_truth_paths(
        &initial,
        scenario.g,
        scenario.softening,
        duration,
        TRUTH_INTEGRATION_DT,
        TRUTH_PATH_SAMPLES,
    )
}

/// Predict target path under full N-body gravity with impulsive TNW burns (truth preview).
pub fn predict_target_path_nbody(
    initial: &[BodyState],
    central_name: &str,
    target_name: &str,
    nodes: &[ManeuverNode],
    sim_time: f32,
    horizon: f32,
    dt: f32,
    g: f32,
    softening: f32,
) -> Vec<Vec3> {
    let mut state: Vec<BodyState> = initial.to_vec();
    let mut path = Vec::new();

    let steps = ((horizon / dt).ceil() as usize).clamp(1, MAX_INTEGRATION_STEPS);
    let mut t = sim_time;
    let mut node_idx = 0;

    if let Some(pos) = state
        .iter()
        .find(|b| b.name == target_name)
        .map(|b| b.position)
    {
        path.push(pos);
    }

    for _ in 0..steps {
        while node_idx < nodes.len() && nodes[node_idx].time <= t + dt * 0.5 {
            apply_tnw_burn(&mut state, central_name, target_name, &nodes[node_idx]);
            node_idx += 1;
        }

        state = velocity_verlet_step(&state, g, softening, dt);
        t += dt;

        if let Some(pos) = state
            .iter()
            .find(|b| b.name == target_name)
            .map(|b| b.position)
        {
            path.push(pos);
        }
    }

    path
}

fn apply_tnw_burn(
    states: &mut [BodyState],
    central_name: &str,
    target_name: &str,
    node: &ManeuverNode,
) {
    let Some(central) = states.iter().find(|b| b.name == central_name) else {
        return;
    };
    let central_pos = central.position;
    let central_vel = central.velocity;

    let Some(target) = states.iter_mut().find(|b| b.name == target_name) else {
        return;
    };

    let rel = RelativeState::new(target.position - central_pos, target.velocity - central_vel);
    let new_rel_vel = orbit::apply_tnw_delta_v(rel, node.prograde, node.normal, node.radial);
    target.velocity = central_vel + new_rel_vel;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::load_scenario_relative;

    #[test]
    fn verlet_two_body_stays_bound() {
        let states = vec![
            BodyState {
                name: "Sun".into(),
                position: Vec3::ZERO,
                velocity: Vec3::ZERO,
                mass: 1e30,
                fixed: true,
            },
            BodyState {
                name: "Earth".into(),
                position: Vec3::new(crate::astro::AU, 0.0, 0.0),
                velocity: Vec3::new(0.0, 0.0, 29_780.0),
                mass: 1.0,
                fixed: false,
            },
        ];
        let e0 = crate::physics::total_energy(&states, crate::astro::G, 1.0e6);
        let mut s = states;
        for _ in 0..500 {
            s = velocity_verlet_step(&s, crate::astro::G, 1.0e6, 3600.0);
        }
        let e1 = crate::physics::total_energy(&s, crate::astro::G, 1.0e6);
        let drift = (e1.0 + e1.1 - e0.0 - e0.1).abs() / (e0.0 + e0.1).abs();
        assert!(drift < 0.02, "energy drift {drift}");
    }

    #[test]
    fn default_scenario_truth_paths_have_samples() {
        let scenario =
            load_scenario_relative("scenarios/default.toml").expect("load");
        let paths = build_truth_paths_for_scenario(&scenario);
        assert!(paths.get("Earth").is_some_and(|p| p.len() >= 32));
    }
}
