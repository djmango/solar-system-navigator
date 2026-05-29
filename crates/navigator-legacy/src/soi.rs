//! Sphere-of-influence (patched conics) helpers for map planning.

use bevy::prelude::*;

/// Hill-sphere style SOI radius: r_SOI ≈ a · (m/M)^(2/5)
pub fn hill_soi_radius(orbital_radius: f32, body_mass: f32, primary_mass: f32) -> f32 {
    if primary_mass <= 0.0 || orbital_radius <= 0.0 {
        return 0.0;
    }
    orbital_radius * (body_mass / primary_mass).powf(0.4)
}

#[derive(Debug, Clone)]
pub struct SoiBodySnapshot {
    pub name: String,
    pub position: Vec3,
    pub velocity: Vec3,
    pub mass: f32,
    pub soi_radius: f32,
    pub is_primary: bool,
}

/// Pick the innermost SOI that contains the vessel (smallest distance / SOI ratio).
pub fn dominant_soi_body<'a>(
    vessel_position: Vec3,
    bodies: &'a [SoiBodySnapshot],
    primary_name: &'a str,
) -> &'a str {
    let mut best: Option<(&'a str, f32)> = None;

    for body in bodies {
        if body.is_primary {
            continue;
        }
        let offset = vessel_position - body.position;
        let distance = offset.length();
        if distance >= body.soi_radius || body.soi_radius <= 0.0 {
            continue;
        }
        let score = distance / body.soi_radius;
        if best.is_none_or(|(_, s)| score < s) {
            best = Some((&body.name, score));
        }
    }

    best.map(|(name, _)| name).unwrap_or(primary_name)
}

pub fn relative_state_to_central(
    vessel_position: Vec3,
    vessel_velocity: Vec3,
    central: &SoiBodySnapshot,
) -> crate::orbit::RelativeState {
    crate::orbit::RelativeState::new(
        vessel_position - central.position,
        vessel_velocity - central.velocity,
    )
}

pub fn find_body<'a>(bodies: &'a [SoiBodySnapshot], name: &str) -> Option<&'a SoiBodySnapshot> {
    bodies.iter().find(|b| b.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sun() -> SoiBodySnapshot {
        SoiBodySnapshot {
            name: "Sun".into(),
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            mass: 10_000.0,
            soi_radius: f32::INFINITY,
            is_primary: true,
        }
    }

    fn earth() -> SoiBodySnapshot {
        SoiBodySnapshot {
            name: "Earth".into(),
            position: Vec3::new(300.0, 0.0, 0.0),
            velocity: Vec3::new(0.0, 0.0, 10.0),
            mass: 1.0,
            soi_radius: 45.0,
            is_primary: false,
        }
    }

    #[test]
    fn vessel_near_earth_uses_earth_soi() {
        let bodies = vec![sun(), earth()];
        let vessel = Vec3::new(305.0, 0.0, 0.0);
        assert_eq!(dominant_soi_body(vessel, &bodies, "Sun"), "Earth");
    }

    #[test]
    fn vessel_far_from_planets_uses_sun() {
        let bodies = vec![sun(), earth()];
        let vessel = Vec3::new(2000.0, 0.0, 0.0);
        assert_eq!(dominant_soi_body(vessel, &bodies, "Sun"), "Sun");
    }
}
