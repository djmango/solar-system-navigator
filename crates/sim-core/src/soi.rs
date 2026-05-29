use glam::DVec3;

use crate::orbit::RelativeState;

pub fn hill_soi_radius(orbital_radius: f64, body_mass: f64, primary_mass: f64) -> f64 {
    if primary_mass <= 0.0 || orbital_radius <= 0.0 {
        return 0.0;
    }
    orbital_radius * (body_mass / primary_mass).powf(0.4)
}

#[derive(Debug, Clone)]
pub struct SoiBodySnapshot {
    pub name: String,
    pub position: DVec3,
    pub velocity: DVec3,
    pub mass: f64,
    pub soi_radius: f64,
    pub is_primary: bool,
}

pub fn dominant_soi_body<'a>(
    vessel_position: DVec3,
    bodies: &'a [SoiBodySnapshot],
    primary_name: &'a str,
) -> &'a str {
    let mut best: Option<(&'a str, f64)> = None;

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
    vessel_position: DVec3,
    vessel_velocity: DVec3,
    central: &SoiBodySnapshot,
) -> RelativeState {
    RelativeState::new(
        vessel_position - central.position,
        vessel_velocity - central.velocity,
    )
}

pub fn find_body<'a>(bodies: &'a [SoiBodySnapshot], name: &str) -> Option<&'a SoiBodySnapshot> {
    bodies.iter().find(|b| b.name == name)
}
