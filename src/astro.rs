//! Astronomical constants and helpers (SI: meters, kilograms, seconds).

#![allow(dead_code)]

/// Newtonian gravitational constant [m³ kg⁻¹ s⁻²]
pub const G: f32 = 6.674_30e-11;

pub const AU: f32 = 149_597_870_700.0;
pub const DAY: f32 = 86_400.0;
pub const YEAR: f32 = 365.25 * DAY;

pub const M_SUN: f32 = 1.988_47e30;
pub const M_EARTH: f32 = 5.972_19e24;
pub const M_MARS: f32 = 6.417_1e23;
pub const M_VENUS: f32 = 4.867_5e24;
pub const M_MERCURY: f32 = 3.301_1e23;
pub const M_MOON: f32 = 7.342e22;

pub const R_SUN: f32 = 6.957e8;
pub const R_EARTH: f32 = 6.371e6;
pub const R_MARS: f32 = 3.390e6;
pub const R_VENUS: f32 = 6.052e6;
pub const R_MERCURY: f32 = 2.439e6;
pub const R_MOON: f32 = 1.737e6;

/// Circular orbital speed around `central_mass` at distance `radius` [m/s].
pub fn circular_orbital_speed(central_mass: f32, radius: f32) -> f32 {
    (G * central_mass / radius).sqrt()
}

/// Default Plummer softening for heliocentric N-body [m].
pub const DEFAULT_SOFTENING: f32 = 1.0e8;

/// Default simulation time warp (simulated seconds per real second).
pub const DEFAULT_TIME_WARP: f32 = 50_000.0;

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
