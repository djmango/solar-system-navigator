//! Hohmann and related impulsive transfer helpers.

use bevy::prelude::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HohmannTransfer {
    pub r1: f32,
    pub r2: f32,
    pub dv_departure: f32,
    pub dv_arrival: f32,
    pub transfer_time: f32,
}

impl HohmannTransfer {
    pub fn total_delta_v(&self) -> f32 {
        self.dv_departure.abs() + self.dv_arrival.abs()
    }
}

/// Circular → circular Hohmann (co-planar). `r2 > r1` raises; `r2 < r1` lowers.
pub fn hohmann_circular(mu: f32, r1: f32, r2: f32) -> Result<HohmannTransfer, &'static str> {
    if mu <= 0.0 || r1 <= 0.0 || r2 <= 0.0 {
        return Err("mu and radii must be positive");
    }
    if (r1 - r2).abs() < 1e-4 {
        return Err("radii must differ");
    }

    let (inner, outer) = if r1 < r2 { (r1, r2) } else { (r2, r1) };
    let a_xfer = 0.5 * (inner + outer);

    let v_inner_circ = (mu / inner).sqrt();
    let v_inner_xfer = (mu * (2.0 / inner - 1.0 / a_xfer)).sqrt();
    let dv_departure = if r1 < r2 {
        v_inner_xfer - v_inner_circ
    } else {
        v_inner_circ - v_inner_xfer
    };

    let v_outer_circ = (mu / outer).sqrt();
    let v_outer_xfer = (mu * (2.0 / outer - 1.0 / a_xfer)).sqrt();
    let dv_arrival = if r1 < r2 {
        v_outer_circ - v_outer_xfer
    } else {
        v_outer_xfer - v_outer_circ
    };

    let transfer_time = std::f32::consts::PI * (a_xfer.powi(3) / mu).sqrt();

    Ok(HohmannTransfer {
        r1,
        r2,
        dv_departure,
        dv_arrival,
        transfer_time,
    })
}

/// Current circular orbit radius from specific orbital energy (μ, r, v).
pub fn circular_orbit_radius(mu: f32, position: Vec3, velocity: Vec3) -> Option<f32> {
    let r = position.length();
    if r < 1e-6 {
        return None;
    }
    let v2 = velocity.length_squared();
    let specific_energy = 0.5 * v2 - mu / r;
    if specific_energy >= -1e-8 {
        return None;
    }
    Some(-mu / (2.0 * specific_energy))
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use bevy::prelude::Vec3;

    #[test]
    fn hohmann_raise_has_positive_dvs() {
        let t = hohmann_circular(1000.0, 100.0, 200.0).unwrap();
        assert!(t.dv_departure > 0.0);
        assert!(t.dv_arrival > 0.0);
        assert!(t.transfer_time > 0.0);
    }

    #[test]
    fn hohmann_symmetric_total_reasonable() {
        let up = hohmann_circular(1000.0, 100.0, 200.0).unwrap();
        let down = hohmann_circular(1000.0, 200.0, 100.0).unwrap();
        assert_relative_eq!(up.total_delta_v(), down.total_delta_v(), epsilon = 0.5);
    }

    #[test]
    fn circular_radius_from_state() {
        let mu = 1000.0;
        let r = 100.0;
        let v = (mu / r as f32).sqrt();
        let est = circular_orbit_radius(mu, Vec3::new(r, 0.0, 0.0), Vec3::new(0.0, 0.0, v)).unwrap();
        assert_relative_eq!(est, r, epsilon = 2.0);
    }
}
