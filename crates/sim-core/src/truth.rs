use std::collections::HashMap;

use glam::DVec3;

use crate::maneuver::ManeuverNode;
use crate::orbit::{self, RelativeState};
use crate::physics::{BodyState, gravitational_acceleration};
use crate::scenario::Scenario;

pub const TRUTH_PATH_SAMPLES: usize = 256;
pub const TRUTH_INTEGRATION_DT: f64 = 600.0;
const MAX_INTEGRATION_STEPS: usize = 12_000;

pub fn velocity_verlet_step(
    states: &[BodyState],
    g: f64,
    softening: f64,
    dt: f64,
) -> Vec<BodyState> {
    let mut accelerations = Vec::with_capacity(states.len());
    for (index, state) in states.iter().enumerate() {
        let accel = if state.fixed {
            DVec3::ZERO
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
            DVec3::ZERO
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

pub fn estimate_orbital_period(
    position: DVec3,
    primary_position: DVec3,
    primary_mass: f64,
    g: f64,
) -> f64 {
    let r = (position - primary_position).length().max(1.0);
    let mu = g * primary_mass;
    if mu <= 0.0 {
        return crate::astro::DAY;
    }
    std::f64::consts::TAU * (r.powi(3) / mu).sqrt()
}

pub fn integration_duration_for_scenario(scenario: &Scenario) -> f64 {
    let primary = scenario.bodies.iter().find(|b| b.fixed);
    let (primary_mass, primary_pos) = primary
        .map(|p| (p.mass, p.position_vec3()))
        .unwrap_or((crate::astro::M_SUN, DVec3::ZERO));

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

pub fn integrate_truth_paths(
    initial: &[BodyState],
    g: f64,
    softening: f64,
    duration: f64,
    dt: f64,
    target_samples: usize,
) -> HashMap<String, Vec<DVec3>> {
    let mut paths: HashMap<String, Vec<DVec3>> = initial
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

pub fn build_truth_paths_for_scenario(scenario: &Scenario) -> HashMap<String, Vec<DVec3>> {
    let initial = states_from_scenario(scenario);
    let duration = integration_duration_for_scenario(scenario);
    integrate_truth_paths(
        &initial,
        scenario.g,
        scenario.softening,
        duration,
        TRUTH_INTEGRATION_DT,
        TRUTH_PATH_SAMPLES,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn predict_target_path_nbody(
    initial: &[BodyState],
    central_name: &str,
    target_name: &str,
    nodes: &[ManeuverNode],
    sim_time: f64,
    horizon: f64,
    dt: f64,
    g: f64,
    softening: f64,
) -> Vec<DVec3> {
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
    use crate::astro;
    use crate::physics::total_energy;
    use crate::scenario::{DEFAULT_SCENARIO, load_scenario_from_str};

    #[test]
    fn verlet_two_body_stays_bound() {
        let states = vec![
            BodyState {
                name: "Sun".into(),
                position: DVec3::ZERO,
                velocity: DVec3::ZERO,
                mass: 1e30,
                fixed: true,
            },
            BodyState {
                name: "Earth".into(),
                position: DVec3::new(astro::AU, 0.0, 0.0),
                velocity: DVec3::new(0.0, 0.0, 29_780.0),
                mass: 1.0,
                fixed: false,
            },
        ];
        let e0 = total_energy(&states, astro::G, 1.0e6);
        let mut s = states;
        for _ in 0..500 {
            s = velocity_verlet_step(&s, astro::G, 1.0e6, 3600.0);
        }
        let e1 = total_energy(&s, astro::G, 1.0e6);
        let drift = (e1.0 + e1.1 - e0.0 - e0.1).abs() / (e0.0 + e0.1).abs();
        assert!(drift < 0.02, "energy drift {drift}");
    }

    #[test]
    fn default_scenario_truth_paths_have_samples() {
        let scenario = load_scenario_from_str(DEFAULT_SCENARIO).expect("load");
        let paths = build_truth_paths_for_scenario(&scenario);
        assert!(paths.get("Earth").is_some_and(|p| p.len() >= 32));
    }
}
