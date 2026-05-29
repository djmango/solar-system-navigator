use bevy::prelude::*;

use crate::components::{
    CelestialBody, FixedBody, Mass, OrbitTrail, Position, TruthOrbit, Velocity,
};
use crate::resources::{PhysicsConstants, SimulationControl, SimulationDiagnostics};
use crate::truth::velocity_verlet_step;

#[derive(Debug, Clone)]
pub struct BodyState {
    pub name: String,
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
        &CelestialBody,
        &Mass,
        &mut Position,
        &mut Velocity,
        &mut Transform,
        &mut OrbitTrail,
        &mut TruthOrbit,
        Option<&FixedBody>,
    )>,
) {
    if simulation_control.paused && !simulation_control.step_once {
        return;
    }

    let dt_base = time.delta_secs() * simulation_control.speed;
    let steps = simulation_control.ticks_per_frame.max(1);
    simulation_control.step_once = false;

    for (_, _, _, _, _, _, mut truth, _) in bodies.iter_mut() {
        truth.ensure_live_if_simulating();
    }

    for _ in 0..steps {
        let snapshot: Vec<BodyState> = bodies
            .iter()
            .map(|(body, mass, pos, vel, _, _, _, fixed)| BodyState {
                name: body.name.clone(),
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

        let new_state = velocity_verlet_step(&snapshot, physics.g, physics.softening, dt_base);

        for (body, _, mut pos, mut vel, mut transform, mut trail, mut truth, fixed) in
            bodies.iter_mut()
        {
            let Some(s) = new_state.iter().find(|s| s.name == body.name) else {
                continue;
            };
            if fixed.is_some() {
                continue;
            }
            pos.0 = s.position;
            vel.0 = s.velocity;
            transform.translation = pos.0;
            trail.push(pos.0);
            if truth.live_only {
                truth.push_live(pos.0);
            }
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
            name: "Sun".to_string(),
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
            name: "Probe".to_string(),
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
                name: "Sun".to_string(),
                position: Vec3::ZERO,
                velocity: Vec3::ZERO,
                mass: 1000.0,
                fixed: true,
            },
            BodyState {
                name: "Probe".to_string(),
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
