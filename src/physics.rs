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
pub fn gravitational_acceleration(
    target: Vec3,
    bodies: &[BodyState],
    g: f32,
    softening: f32,
) -> Vec3 {
    let softening_sq = softening * softening;
    let mut acceleration = Vec3::ZERO;

    for body in bodies {
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
        for state in &snapshot {
            let accel = if state.fixed {
                Vec3::ZERO
            } else {
                gravitational_acceleration(
                    state.position,
                    &snapshot,
                    physics.g,
                    physics.softening,
                )
            };
            accelerations.push(accel);
        }

        for ((_, mut pos, mut vel, mut transform, mut trail, fixed), accel) in
            bodies.iter_mut().zip(accelerations)
        {
            if fixed.is_some() {
                continue;
            }
            let half_dt = dt_base * 0.5;
            vel.0 += accel * half_dt;
            pos.0 += vel.0 * dt_base;
            vel.0 += accel * half_dt;
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
        let accel = gravitational_acceleration(Vec3::new(100.0, 0.0, 0.0), &bodies, 1.0, 1.0);
        assert!(accel.x < 0.0);
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
