use bevy::prelude::*;

use crate::components::{FixedBody, Mass, OrbitTrail, Position, Velocity};
use crate::resources::{PhysicsConstants, SimulationControl, SimulationDiagnostics};

#[derive(Debug, Clone, Copy)]
pub struct BodyState {
    pub position: Vec3,
    pub velocity: Vec3,
    pub mass: f32,
    pub fixed: bool,
}

/// Gravitational acceleration on `target` from all `bodies` (Plummer softening).
/// Skips the body at `skip_index` so a body does not attract itself.
pub fn gravitational_acceleration(
    target: Vec3,
    skip_index: Option<usize>,
    bodies: &[BodyState],
    g: f32,
    softening: f32,
) -> Vec3 {
    let softening_sq = softening * softening;
    let mut acceleration = Vec3::ZERO;

    for (index, body) in bodies.iter().enumerate() {
        if skip_index == Some(index) {
            continue;
        }
        let delta = body.position - target;
        let dist_sq = delta.length_squared() + softening_sq;
        let dist = dist_sq.sqrt();
        let force_mag = g * body.mass / dist_sq;
        acceleration += delta / dist * force_mag;
    }

    acceleration
}

pub fn total_energy(bodies: &[BodyState], g: f32, softening: f32) -> (f32, f32) {
    let softening_sq = softening * softening;
    let mut kinetic = 0.0f32;
    let mut potential = 0.0f32;

    for (i, a) in bodies.iter().enumerate() {
        if !a.fixed {
            kinetic += 0.5 * a.mass * a.velocity.length_squared();
        }
        for b in bodies.iter().skip(i + 1) {
            let delta = a.position - b.position;
            let dist = (delta.length_squared() + softening_sq).sqrt();
            potential -= g * a.mass * b.mass / dist;
        }
    }

    (kinetic, potential)
}

pub fn orbital_physics(
    time: Res<Time>,
    physics: Res<PhysicsConstants>,
    mut simulation_control: ResMut<SimulationControl>,
    mut diagnostics: ResMut<SimulationDiagnostics>,
    mut bodies: Query<(
        &Mass,
        &mut Position,
        &mut Velocity,
        &mut Transform,
        &mut OrbitTrail,
        Option<&FixedBody>,
    )>,
) {
    if simulation_control.paused && !simulation_control.step_once {
        return;
    }

    let dt_base = time.delta_secs() * simulation_control.speed;
    let steps = simulation_control.ticks_per_frame.max(1);
    simulation_control.step_once = false;

    for _ in 0..steps {
        let snapshot: Vec<BodyState> = bodies
            .iter()
            .map(|(mass, pos, vel, _, _, fixed)| BodyState {
                position: pos.0,
                velocity: vel.0,
                mass: mass.0,
                fixed: fixed.is_some(),
            })
            .collect();

        diagnostics.body_count = snapshot.len() as u32;
        let (ke, pe) = total_energy(&snapshot, physics.g, physics.softening);
        diagnostics.kinetic_energy = ke;
        diagnostics.potential_energy = pe;
        diagnostics.total_energy = ke + pe;

        let mut accelerations = Vec::with_capacity(snapshot.len());
        for (index, state) in snapshot.iter().enumerate() {
            let accel = if state.fixed {
                Vec3::ZERO
            } else {
                gravitational_acceleration(
                    state.position,
                    Some(index),
                    &snapshot,
                    physics.g,
                    physics.softening,
                )
            };
            accelerations.push(accel);
        }

        // Velocity Verlet: recompute acceleration at the new position.
        let mut new_positions: Vec<Vec3> = Vec::with_capacity(snapshot.len());
        let mut new_velocities: Vec<Vec3> = Vec::with_capacity(snapshot.len());

        for (index, state) in snapshot.iter().enumerate() {
            if state.fixed {
                new_positions.push(state.position);
                new_velocities.push(state.velocity);
                continue;
            }
            let half_dt = dt_base * 0.5;
            let v_half = state.velocity + accelerations[index] * half_dt;
            let pos_new = state.position + v_half * dt_base;
            new_positions.push(pos_new);
            new_velocities.push(v_half);
        }

        let new_snapshot: Vec<BodyState> = snapshot
            .iter()
            .enumerate()
            .map(|(i, s)| BodyState {
                position: new_positions[i],
                velocity: new_velocities[i],
                mass: s.mass,
                fixed: s.fixed,
            })
            .collect();

        let mut accelerations_end = Vec::with_capacity(new_snapshot.len());
        for (index, state) in new_snapshot.iter().enumerate() {
            let accel = if state.fixed {
                Vec3::ZERO
            } else {
                gravitational_acceleration(
                    state.position,
                    Some(index),
                    &new_snapshot,
                    physics.g,
                    physics.softening,
                )
            };
            accelerations_end.push(accel);
        }

        for (i, (_, mut pos, mut vel, mut transform, mut trail, fixed)) in
            bodies.iter_mut().enumerate()
        {
            if fixed.is_some() {
                continue;
            }
            let half_dt = dt_base * 0.5;
            vel.0 = new_velocities[i] + accelerations_end[i] * half_dt;
            pos.0 = new_positions[i];
            transform.translation = pos.0;
            trail.push(pos.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn acceleration_points_toward_massive_body() {
        let bodies = [BodyState {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            mass: 1000.0,
            fixed: true,
        }];
        let accel = gravitational_acceleration(Vec3::new(100.0, 0.0, 0.0), None, &bodies, 1.0, 1.0);
        assert!(accel.x < 0.0);
        assert_relative_eq!(accel.y, 0.0, epsilon = 1e-5);
        assert_relative_eq!(accel.z, 0.0, epsilon = 1e-5);
    }

    #[test]
    fn self_interaction_is_zero() {
        let bodies = [BodyState {
            position: Vec3::new(50.0, 0.0, 0.0),
            velocity: Vec3::ZERO,
            mass: 10.0,
            fixed: false,
        }];
        let accel = gravitational_acceleration(bodies[0].position, Some(0), &bodies, 1.0, 0.0);
        assert_relative_eq!(accel.x, 0.0, epsilon = 1e-5);
        assert_relative_eq!(accel.y, 0.0, epsilon = 1e-5);
        assert_relative_eq!(accel.z, 0.0, epsilon = 1e-5);
    }

    #[test]
    fn energy_is_negative_for_bound_pair() {
        let bodies = [
            BodyState {
                position: Vec3::ZERO,
                velocity: Vec3::ZERO,
                mass: 1000.0,
                fixed: true,
            },
            BodyState {
                position: Vec3::new(100.0, 0.0, 0.0),
                velocity: Vec3::new(0.0, 0.0, 3.5),
                mass: 1.0,
                fixed: false,
            },
        ];
        let (ke, pe) = total_energy(&bodies, 1.0, 1.0);
        assert!(ke > 0.0);
        assert!(pe < 0.0);
        assert!(ke + pe < 0.0);
    }
}
