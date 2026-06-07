use std::collections::HashMap;

use glam::DVec3;

use crate::maneuver::ManeuverNode;
use crate::orbit::{self, RelativeState};
use crate::physics::{BodyState, gravitational_acceleration};
use crate::scenario::Scenario;

pub const TRUTH_PATH_SAMPLES: usize = 256;
pub const TRUTH_INTEGRATION_DT: f64 = 600.0;
const MAX_INTEGRATION_STEPS: usize = 12_000;
/// Burn-time tolerance [s] for splitting an integration step at a node.
const MANEUVER_TIME_EPS: f64 = 1.0e-3;

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

    // Preview must not mutate the caller's nodes; clone the still-pending ones.
    let mut preview_nodes: Vec<ManeuverNode> = nodes
        .iter()
        .filter(|n| !n.executed && n.time >= sim_time - MANEUVER_TIME_EPS)
        .cloned()
        .collect();
    preview_nodes.sort_by(|a, b| a.time.total_cmp(&b.time));

    if let Some(pos) = state
        .iter()
        .find(|b| b.name == target_name)
        .map(|b| b.position)
    {
        path.push(pos);
    }

    for _ in 0..steps {
        t = advance_state_with_maneuvers(
            &mut state,
            &mut preview_nodes,
            central_name,
            target_name,
            t,
            dt,
            g,
            softening,
        );

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

/// Advance the N-body state over `dt`, splitting the step exactly at any
/// maneuver-node times that fall inside the interval.
///
/// This is the single authoritative burn path shared by the live simulation
/// and the map preview: nodes are instantaneous burns at universal sim time,
/// applied in TNW from the state at the burn instant, so execution and
/// prediction stay in lockstep even under large time-warp steps. Nodes that
/// fire are flagged `executed`.
#[allow(clippy::too_many_arguments)]
pub fn advance_state_with_maneuvers(
    state: &mut Vec<BodyState>,
    nodes: &mut [ManeuverNode],
    central_name: &str,
    target_name: &str,
    start_time: f64,
    dt: f64,
    g: f64,
    softening: f64,
) -> f64 {
    if dt <= 0.0 {
        return start_time;
    }

    let end_time = start_time + dt;
    let mut t = start_time;

    loop {
        let Some(idx) = next_pending_node(nodes, t, end_time) else {
            let remaining = end_time - t;
            if remaining > MANEUVER_TIME_EPS {
                *state = velocity_verlet_step(state, g, softening, remaining);
            }
            return end_time;
        };

        let burn_time = nodes[idx].time.clamp(t, end_time);
        let coast = burn_time - t;
        if coast > MANEUVER_TIME_EPS {
            *state = velocity_verlet_step(state, g, softening, coast);
        }
        apply_tnw_burn(state, central_name, target_name, &nodes[idx]);
        nodes[idx].executed = true;
        t = burn_time;
    }
}

fn next_pending_node(nodes: &[ManeuverNode], start_time: f64, end_time: f64) -> Option<usize> {
    nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !n.executed && n.time <= end_time + MANEUVER_TIME_EPS)
        .min_by(|(_, a), (_, b)| a.time.max(start_time).total_cmp(&b.time.max(start_time)))
        .map(|(i, _)| i)
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
    if target.fixed {
        return;
    }

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

    fn sun_probe() -> Vec<BodyState> {
        vec![
            BodyState {
                name: "Sun".into(),
                position: DVec3::ZERO,
                velocity: DVec3::ZERO,
                mass: 1.0e30,
                fixed: true,
            },
            BodyState {
                name: "Probe".into(),
                position: DVec3::new(astro::AU, 0.0, 0.0),
                velocity: DVec3::new(0.0, 0.0, 29_780.0),
                mass: 1.0,
                fixed: false,
            },
        ]
    }

    #[test]
    fn burn_executes_at_node_time_inside_large_step() {
        // A single huge step that straddles the node time must still fire the
        // burn (this is the time-warp case the old per-substep check missed).
        let mut state = sun_probe();
        let mut nodes = vec![ManeuverNode {
            time: 5_000.0,
            prograde: 1_000.0,
            normal: 0.0,
            radial: 0.0,
            executed: false,
        }];
        let v0 = state
            .iter()
            .find(|s| s.name == "Probe")
            .unwrap()
            .velocity
            .length();

        advance_state_with_maneuvers(
            &mut state,
            &mut nodes,
            "Sun",
            "Probe",
            0.0,
            50_000.0,
            astro::G,
            1.0e6,
        );

        assert!(nodes[0].executed, "node should have fired inside the step");
        let v1 = state
            .iter()
            .find(|s| s.name == "Probe")
            .unwrap()
            .velocity
            .length();
        assert!(
            v1 > v0 + 500.0,
            "prograde burn should raise speed: {v0} -> {v1}"
        );
    }

    #[test]
    fn future_node_is_not_executed_early() {
        let mut state = sun_probe();
        let mut nodes = vec![ManeuverNode {
            time: 1.0e6,
            prograde: 1_000.0,
            normal: 0.0,
            radial: 0.0,
            executed: false,
        }];
        advance_state_with_maneuvers(
            &mut state,
            &mut nodes,
            "Sun",
            "Probe",
            0.0,
            1_000.0,
            astro::G,
            1.0e6,
        );
        assert!(!nodes[0].executed);
    }

    #[test]
    fn live_and_preview_agree_after_burn() {
        // Stepping the live integrator across a node must land on the same point
        // the map preview predicts for that time — the property that keeps the
        // amber path honest.
        let nodes = vec![ManeuverNode {
            time: 30_000.0,
            prograde: 800.0,
            normal: 120.0,
            radial: -60.0,
            executed: false,
        }];
        let dt = 600.0;
        let horizon = 120_000.0;

        // Preview: dense fixed-step integration with the (immutable) node list.
        let preview = predict_target_path_nbody(
            &sun_probe(),
            "Sun",
            "Probe",
            &nodes,
            0.0,
            horizon,
            dt,
            astro::G,
            1.0e6,
        );

        // Live: same dt cadence, mutating a working copy of the nodes.
        let mut live_state = sun_probe();
        let mut live_nodes = nodes.clone();
        let mut t = 0.0;
        let steps = (horizon / dt) as usize;
        for _ in 0..steps {
            t = advance_state_with_maneuvers(
                &mut live_state,
                &mut live_nodes,
                "Sun",
                "Probe",
                t,
                dt,
                astro::G,
                1.0e6,
            );
        }

        let live_pos = live_state
            .iter()
            .find(|s| s.name == "Probe")
            .unwrap()
            .position;
        let preview_pos = *preview.last().unwrap();
        let err = (live_pos - preview_pos).length();
        let scale = live_pos.length().max(1.0);
        assert!(
            err / scale < 1.0e-6,
            "live vs preview divergence {err} m (scale {scale})"
        );
    }
}
