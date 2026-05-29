use glam::DVec3;

#[derive(Debug, Clone)]
pub struct BodyState {
    pub name: String,
    pub position: DVec3,
    pub velocity: DVec3,
    pub mass: f64,
    pub fixed: bool,
}

pub fn gravitational_acceleration(
    target: DVec3,
    skip_index: Option<usize>,
    bodies: &[BodyState],
    g: f64,
    softening: f64,
) -> DVec3 {
    let softening_sq = softening * softening;
    let mut acceleration = DVec3::ZERO;

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

pub fn total_energy(bodies: &[BodyState], g: f64, softening: f64) -> (f64, f64) {
    let softening_sq = softening * softening;
    let mut kinetic = 0.0;
    let mut potential = 0.0;

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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn acceleration_points_toward_massive_body() {
        let bodies = [BodyState {
            name: "Sun".into(),
            position: DVec3::ZERO,
            velocity: DVec3::ZERO,
            mass: 1000.0,
            fixed: true,
        }];
        let accel =
            gravitational_acceleration(DVec3::new(100.0, 0.0, 0.0), None, &bodies, 1.0, 1.0);
        assert!(accel.x < 0.0);
        assert_relative_eq!(accel.y, 0.0, epsilon = 1e-5);
    }

    #[test]
    fn energy_is_negative_for_bound_pair() {
        let bodies = [
            BodyState {
                name: "Sun".into(),
                position: DVec3::ZERO,
                velocity: DVec3::ZERO,
                mass: 1000.0,
                fixed: true,
            },
            BodyState {
                name: "Probe".into(),
                position: DVec3::new(100.0, 0.0, 0.0),
                velocity: DVec3::new(0.0, 0.0, 3.5),
                mass: 1.0,
                fixed: false,
            },
        ];
        let (ke, pe) = total_energy(&bodies, 1.0, 1.0);
        assert!(ke + pe < 0.0);
    }
}
