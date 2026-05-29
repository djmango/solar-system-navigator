use glam::DVec3;

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct HohmannTransfer {
    pub r1: f64,
    pub r2: f64,
    pub dv_departure: f64,
    pub dv_arrival: f64,
    pub transfer_time: f64,
}

impl HohmannTransfer {
    pub fn total_delta_v(&self) -> f64 {
        self.dv_departure.abs() + self.dv_arrival.abs()
    }
}

pub fn hohmann_circular(mu: f64, r1: f64, r2: f64) -> Result<HohmannTransfer, &'static str> {
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

    let transfer_time = std::f64::consts::PI * (a_xfer.powi(3) / mu).sqrt();

    Ok(HohmannTransfer {
        r1,
        r2,
        dv_departure,
        dv_arrival,
        transfer_time,
    })
}

pub fn circular_orbit_radius(mu: f64, position: DVec3, velocity: DVec3) -> Option<f64> {
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

    #[test]
    fn hohmann_raise_has_positive_dvs() {
        let t = hohmann_circular(1000.0, 100.0, 200.0).unwrap();
        assert!(t.dv_departure > 0.0);
        assert!(t.dv_arrival > 0.0);
    }
}
