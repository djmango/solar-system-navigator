//! Astronomical constants and helpers (SI: meters, kilograms, seconds).

pub const G: f64 = 6.674_30e-11;
pub const AU: f64 = 149_597_870_700.0;
pub const DAY: f64 = 86_400.0;
pub const YEAR: f64 = 365.25 * DAY;

pub const M_SUN: f64 = 1.988_47e30;
pub const M_EARTH: f64 = 5.972_19e24;
pub const M_MARS: f64 = 6.417_1e23;

pub const DEFAULT_SOFTENING: f64 = 1.0e8;
pub const DEFAULT_TIME_WARP: f64 = 5_000.0;

pub fn circular_orbital_speed(central_mass: f64, radius: f64) -> f64 {
    (G * central_mass / radius).sqrt()
}

pub fn format_length(meters: f64) -> String {
    if meters.is_finite() && meters >= AU {
        format!("{:.3} AU", meters / AU)
    } else if meters >= 1.0e9 {
        format!("{:.2} Gm", meters / 1.0e9)
    } else if meters >= 1.0e6 {
        format!("{:.1} Mm", meters / 1.0e6)
    } else if meters >= 1.0e3 {
        format!("{:.1} km", meters / 1.0e3)
    } else {
        format!("{:.0} m", meters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn earth_circular_speed_is_about_30_km_s() {
        let v = circular_orbital_speed(M_SUN, AU);
        assert_relative_eq!(v, 29_780.0, epsilon = 200.0);
    }
}
