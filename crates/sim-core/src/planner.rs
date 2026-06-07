use glam::DVec3;

use crate::orbit::{self, RelativeState};
use crate::scenario::{BodyDef, Scenario};
use crate::soi::{self, SoiBodySnapshot};
use crate::transfer::{self, HohmannTransfer};

#[derive(Debug, Clone)]
pub struct HohmannComputeResult {
    pub transfer: HohmannTransfer,
    pub warning: Option<String>,
}

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

/// Circular Hohmann from the vessel's **current radius** to `hohmann_target_radius`.
/// Returns a descriptive error for escape trajectories; warns when eccentricity is high.
pub fn compute_hohmann_for_target(
    snapshots: &[SoiBodySnapshot],
    central_name: &str,
    target_name: &str,
    physics_g: f64,
    hohmann_target_radius: f64,
) -> Result<HohmannComputeResult, String> {
    let central = soi::find_body(snapshots, central_name)
        .ok_or_else(|| format!("central body '{central_name}' not found"))?;
    let target = soi::find_body(snapshots, target_name)
        .ok_or_else(|| format!("vessel '{target_name}' not found"))?;
    let mu = physics_g * central.mass;
    let rel = soi::relative_state_to_central(target.position, target.velocity, central);
    let r = rel.position.length();
    if r < 1e-3 {
        return Err("vessel is too close to the central body for Hohmann planning".into());
    }

    let elements = orbit::elements_from_state(mu, rel)
        .ok_or_else(|| "could not determine orbit around the central body".to_string())?;

    if elements.eccentricity >= 1.0 || !elements.semi_major_axis.is_finite() {
        return Err(
            "Hohmann requires a closed orbit — vessel is on an escape or hyperbolic trajectory"
                .into(),
        );
    }

    let warning = if elements.eccentricity > 0.05 {
        Some(format!(
            "Eccentric orbit (e={:.2}): Hohmann Δv is approximate (uses current radius {:.0} km)",
            elements.eccentricity,
            r / 1000.0
        ))
    } else {
        None
    };

    let transfer = transfer::hohmann_circular(mu, r, hohmann_target_radius)
        .map_err(|reason| format!("Hohmann transfer invalid: {reason}"))?;

    Ok(HohmannComputeResult { transfer, warning })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::soi::SoiBodySnapshot;
    use glam::DVec3;

    fn snapshot(name: &str, pos: DVec3, vel: DVec3, mass: f64) -> SoiBodySnapshot {
        SoiBodySnapshot {
            name: name.to_string(),
            position: pos,
            velocity: vel,
            mass,
            soi_radius: 1.0e12,
            is_primary: name == "Sun",
        }
    }

    #[test]
    fn hohmann_rejects_escape_trajectory() {
        let mu_body = 1.0e30;
        let escape_v = (2.0_f64 * mu_body / 1.0e11).sqrt();
        let snapshots = vec![
            snapshot("Sun", DVec3::ZERO, DVec3::ZERO, mu_body),
            snapshot(
                "Probe",
                DVec3::new(1.0e11, 0.0, 0.0),
                DVec3::new(0.0, 0.0, escape_v),
                1000.0,
            ),
        ];
        let err = compute_hohmann_for_target(&snapshots, "Sun", "Probe", 1.0, 2.0e11)
            .expect_err("escape");
        assert!(err.contains("escape"));
    }

    #[test]
    fn hohmann_warns_on_eccentric_orbit() {
        let mu_body = 1.0e30;
        let r = 1.0e11_f64;
        let v_circ = (mu_body / r).sqrt();
        let snapshots = vec![
            snapshot("Sun", DVec3::ZERO, DVec3::ZERO, mu_body),
            snapshot(
                "Probe",
                DVec3::new(r, 0.0, 0.0),
                DVec3::new(0.0, 0.0, v_circ * 1.2),
                1000.0,
            ),
        ];
        let result =
            compute_hohmann_for_target(&snapshots, "Sun", "Probe", 1.0, 2.0e11).expect("bound");
        assert!(result.warning.is_some());
        assert!(result.transfer.dv_departure.is_finite());
    }
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
