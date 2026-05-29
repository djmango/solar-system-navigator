use glam::DVec3;

use crate::orbit::RelativeState;
use crate::scenario::{BodyDef, Scenario};
use crate::soi::{self, SoiBodySnapshot};
use crate::transfer::{self, HohmannTransfer};

pub fn compute_soi_radius_for_body(
    def: &BodyDef,
    scenario: &Scenario,
    primary_mass: f64,
    primary_position: DVec3,
) -> f64 {
    if let Some(r) = def.soi_radius {
        return r * scenario.soi_scale;
    }
    if def.fixed {
        return f64::INFINITY;
    }
    let orbital_radius = (def.position_vec3() - primary_position)
        .length()
        .max(def.radius * 2.0);
    soi::hill_soi_radius(orbital_radius, def.mass, primary_mass) * scenario.soi_scale
}

pub fn compute_hohmann_for_target(
    snapshots: &[SoiBodySnapshot],
    central_name: &str,
    target_name: &str,
    physics_g: f64,
    hohmann_target_radius: f64,
) -> Option<HohmannTransfer> {
    let central = soi::find_body(snapshots, central_name)?;
    let target = soi::find_body(snapshots, target_name)?;
    let mu = physics_g * central.mass;
    let rel = soi::relative_state_to_central(target.position, target.velocity, central);
    transfer::hohmann_circular(
        mu,
        transfer::circular_orbit_radius(mu, rel.position, rel.velocity)?,
        hohmann_target_radius,
    )
    .ok()
}

pub fn build_soi_snapshots_from_states(
    bodies: &[(String, DVec3, DVec3, f64, f64, bool)],
) -> Vec<SoiBodySnapshot> {
    bodies
        .iter()
        .map(|(name, pos, vel, mass, soi, is_primary)| SoiBodySnapshot {
            name: name.clone(),
            position: *pos,
            velocity: *vel,
            mass: *mass,
            soi_radius: *soi,
            is_primary: *is_primary,
        })
        .collect()
}

pub fn relative_target_state(
    snapshots: &[SoiBodySnapshot],
    central_name: &str,
    target_name: &str,
) -> Option<RelativeState> {
    let central = soi::find_body(snapshots, central_name)?;
    let target = soi::find_body(snapshots, target_name)?;
    Some(soi::relative_state_to_central(
        target.position,
        target.velocity,
        central,
    ))
}
