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

/// Pick the innermost SOI that contains the vessel (smallest distance / SOI ratio).
///
/// `exclude` skips a body (typically the active vessel itself) so a selected
/// craft or planet resolves to its parent SOI instead of its own zero-distance
/// self-frame.
pub fn dominant_soi_body<'a>(
    vessel_position: DVec3,
    bodies: &'a [SoiBodySnapshot],
    primary_name: &'a str,
    exclude: Option<&str>,
) -> &'a str {
    let mut best: Option<(&'a str, f64)> = None;

    for body in bodies {
        if body.is_primary || exclude == Some(body.name.as_str()) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sun() -> SoiBodySnapshot {
        SoiBodySnapshot {
            name: "Sun".into(),
            position: DVec3::ZERO,
            velocity: DVec3::ZERO,
            mass: 1.0e30,
            soi_radius: f64::INFINITY,
            is_primary: true,
        }
    }

    fn earth() -> SoiBodySnapshot {
        SoiBodySnapshot {
            name: "Earth".into(),
            position: DVec3::new(1.5e11, 0.0, 0.0),
            velocity: DVec3::ZERO,
            mass: 6.0e24,
            soi_radius: 9.0e8,
            is_primary: false,
        }
    }

    #[test]
    fn vessel_near_earth_uses_earth_soi() {
        let bodies = vec![sun(), earth()];
        let vessel = DVec3::new(1.5e11 + 1.0e8, 0.0, 0.0);
        assert_eq!(dominant_soi_body(vessel, &bodies, "Sun", None), "Earth");
    }

    #[test]
    fn selected_planet_resolves_to_parent_soi() {
        // Sitting on Earth resolves to the Sun once Earth is excluded as the vessel.
        let bodies = vec![sun(), earth()];
        let vessel = earth().position;
        assert_eq!(
            dominant_soi_body(vessel, &bodies, "Sun", Some("Earth")),
            "Sun"
        );
    }
}
