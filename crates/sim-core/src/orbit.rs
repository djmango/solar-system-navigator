use glam::{DMat3, DVec3};

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct OrbitElements {
    pub semi_major_axis: f64,
    pub eccentricity: f64,
    pub inclination: f64,
    pub longitude_ascending: f64,
    pub argument_periapsis: f64,
    pub true_anomaly: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct RelativeState {
    pub position: DVec3,
    pub velocity: DVec3,
}

impl RelativeState {
    pub fn new(position: DVec3, velocity: DVec3) -> Self {
        Self { position, velocity }
    }
}

pub fn tnw_frame(state: RelativeState) -> (DVec3, DVec3, DVec3) {
    let r = state.position;
    let r_hat = if r.length_squared() > 1e-8 {
        r.normalize()
    } else {
        DVec3::X
    };
    let h = r.cross(state.velocity);
    let n_hat = if h.length_squared() > 1e-8 {
        h.normalize()
    } else {
        DVec3::Y
    };
    let t_hat = n_hat.cross(r_hat).normalize_or_zero();
    let t_hat = if t_hat.length_squared() > 1e-8 {
        t_hat
    } else {
        DVec3::Z
    };
    (t_hat, n_hat, r_hat)
}

pub fn apply_tnw_delta_v(state: RelativeState, prograde: f64, normal: f64, radial: f64) -> DVec3 {
    let (t, n, r) = tnw_frame(state);
    state.velocity + t * prograde + n * normal + r * radial
}

pub fn elements_from_state(mu: f64, state: RelativeState) -> Option<OrbitElements> {
    let r = state.position;
    let v = state.velocity;
    let r_mag = r.length();
    if r_mag < 1e-6 {
        return None;
    }

    let v_sq = v.length_squared();
    let specific_energy = 0.5 * v_sq - mu / r_mag;
    let h_vec = r.cross(v);
    let h_mag = h_vec.length();
    if h_mag < 1e-8 {
        return None;
    }

    let e_vec = v.cross(h_vec) / mu - r / r_mag;
    let eccentricity = e_vec.length();

    let semi_major_axis = if specific_energy.abs() < 1e-8 {
        f64::INFINITY
    } else {
        -mu / (2.0 * specific_energy)
    };

    let inclination = (h_vec.z / h_mag).acos().clamp(0.0, std::f64::consts::PI);
    let n_vec = DVec3::new(-h_vec.y, h_vec.x, 0.0);
    let longitude_ascending = if n_vec.length_squared() > 1e-8 {
        n_vec.y.atan2(n_vec.x)
    } else {
        0.0
    };

    let argument_periapsis = if eccentricity > 1e-6 {
        let n_hat = if n_vec.length_squared() > 1e-8 {
            n_vec.normalize()
        } else {
            DVec3::X
        };
        let e_hat = e_vec / eccentricity;
        let cos_arg = n_hat.dot(e_hat).clamp(-1.0, 1.0);
        let sin_arg = e_hat.z;
        sin_arg.atan2((1.0 - cos_arg * cos_arg).max(0.0).sqrt())
    } else {
        0.0
    };

    let true_anomaly = if eccentricity > 1e-6 {
        let e_hat = e_vec / eccentricity;
        let cos_nu = e_hat.dot(r / r_mag).clamp(-1.0, 1.0);
        let sin_nu = (r / r_mag).cross(e_hat).dot(h_vec / h_mag);
        sin_nu.atan2(cos_nu)
    } else {
        0.0
    };

    Some(OrbitElements {
        semi_major_axis,
        eccentricity,
        inclination,
        longitude_ascending,
        argument_periapsis,
        true_anomaly,
    })
}

fn rotation_from_elements(elements: &OrbitElements) -> DMat3 {
    let cos_i = elements.inclination.cos();
    let sin_i = elements.inclination.sin();
    let cos_omega = elements.longitude_ascending.cos();
    let sin_omega = elements.longitude_ascending.sin();
    let cos_w = elements.argument_periapsis.cos();
    let sin_w = elements.argument_periapsis.sin();

    let r1 = DMat3::from_cols(
        DVec3::new(cos_omega, sin_omega, 0.0),
        DVec3::new(-sin_omega, cos_omega, 0.0),
        DVec3::Z,
    );
    let r2 = DMat3::from_cols(
        DVec3::new(1.0, 0.0, 0.0),
        DVec3::new(0.0, cos_i, sin_i),
        DVec3::new(0.0, -sin_i, cos_i),
    );
    let r3 = DMat3::from_cols(
        DVec3::new(cos_w, sin_w, 0.0),
        DVec3::new(-sin_w, cos_w, 0.0),
        DVec3::Z,
    );
    r1 * r2 * r3
}

pub fn sample_orbit_path(mu: f64, state: RelativeState, segments: usize) -> Vec<DVec3> {
    let Some(elements) = elements_from_state(mu, state) else {
        return vec![state.position];
    };

    if !elements.semi_major_axis.is_finite() || elements.semi_major_axis <= 0.0 {
        return sample_escape_arc(mu, state, segments);
    }

    if elements.eccentricity >= 1.0 {
        return sample_escape_arc(mu, state, segments);
    }

    let a = elements.semi_major_axis;
    let e = elements.eccentricity.clamp(0.0, 0.999);
    let rotation = rotation_from_elements(&elements);
    let mut points = Vec::with_capacity(segments + 1);

    for i in 0..=segments {
        let nu = (i as f64 / segments as f64) * std::f64::consts::TAU;
        let p = a * (1.0 - e * e);
        let r = p / (1.0 + e * nu.cos());
        let pos_orbital = DVec3::new(r * nu.cos(), r * nu.sin(), 0.0);
        points.push(rotation * pos_orbital);
    }

    points
}

fn sample_escape_arc(mu: f64, state: RelativeState, segments: usize) -> Vec<DVec3> {
    let mut points = vec![state.position];
    let mut pos = state.position;
    let mut vel = state.velocity;
    let dt = 0.4 * pos.length() / vel.length().max(0.1);
    for _ in 0..segments {
        let r = pos.length().max(1.0);
        let accel = -pos / (r * r * r) * mu;
        vel += accel * dt;
        pos += vel * dt;
        points.push(pos);
    }
    points
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn circular_orbit_has_low_eccentricity() {
        let mu = 1000.0_f64;
        let r = 100.0_f64;
        let v = (mu / r).sqrt();
        let state = RelativeState::new(DVec3::new(r, 0.0, 0.0), DVec3::new(0.0, 0.0, v));
        let elements = elements_from_state(mu, state).unwrap();
        assert!(elements.eccentricity < 0.05);
        assert_relative_eq!(elements.semi_major_axis, r, epsilon = 2.0);
    }
}
