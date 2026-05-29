use glam::DVec3;
use serde::Deserialize;

use crate::astro::{DEFAULT_SOFTENING, G};

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct Scenario {
    pub name: String,
    #[serde(default = "default_g")]
    pub g: f64,
    #[serde(default = "default_softening")]
    pub softening: f64,
    #[serde(default = "default_soi_scale")]
    pub soi_scale: f64,
    #[serde(default = "default_visual_exaggeration")]
    pub visual_exaggeration: f64,
    pub bodies: Vec<BodyDef>,
}

fn default_soi_scale() -> f64 {
    1.0
}
fn default_g() -> f64 {
    G
}
fn default_softening() -> f64 {
    DEFAULT_SOFTENING
}
fn default_visual_exaggeration() -> f64 {
    1.0
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct BodyDef {
    pub name: String,
    pub mass: f64,
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub radius: f64,
    #[serde(default)]
    pub visual_radius: Option<f64>,
    pub color: [f64; 3],
    #[serde(default = "default_emissive")]
    pub emissive: f64,
    #[serde(default)]
    pub atmosphere: Option<[f64; 3]>,
    #[serde(default)]
    pub texture: Option<String>,
    #[serde(default)]
    pub soi_radius: Option<f64>,
    #[serde(default)]
    pub fixed: bool,
    #[serde(default)]
    pub probe: bool,
}

fn default_emissive() -> f64 {
    0.8
}

impl BodyDef {
    pub fn position_vec3(&self) -> DVec3 {
        DVec3::from_array(self.position)
    }

    pub fn velocity_vec3(&self) -> DVec3 {
        DVec3::from_array(self.velocity)
    }

    pub fn display_radius(&self, visual_exaggeration: f64) -> f64 {
        self.visual_radius
            .unwrap_or(self.radius * visual_exaggeration)
    }
}

pub fn load_scenario_from_str(contents: &str) -> Result<Scenario, String> {
    toml::from_str(contents).map_err(|e| format!("Failed to parse scenario TOML: {e}"))
}

pub fn validate_circular_speeds(scenario: &Scenario) -> Result<(), String> {
    let primary = scenario
        .bodies
        .iter()
        .find(|b| b.fixed)
        .ok_or_else(|| "scenario has no fixed primary body".to_string())?;
    let primary_pos = primary.position_vec3();

    for body in &scenario.bodies {
        if body.fixed || body.probe {
            continue;
        }
        let r = (body.position_vec3() - primary_pos).length();
        if r < 1.0 {
            continue;
        }
        let expected = (scenario.g * primary.mass / r).sqrt();
        let actual = body.velocity_vec3().length();
        let rel_err = (actual - expected).abs() / expected;
        if rel_err > 0.02 {
            return Err(format!(
                "{}: speed {actual:.0} m/s differs from circular {expected:.0} m/s by {:.1}%",
                body.name,
                rel_err * 100.0
            ));
        }
    }
    Ok(())
}

pub const DEFAULT_SCENARIO: &str = include_str!("../../../assets/scenarios/default.toml");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scenario_loads() {
        let scenario = load_scenario_from_str(DEFAULT_SCENARIO).expect("parse");
        assert!(scenario.bodies.len() >= 5);
        validate_circular_speeds(&scenario).expect("circular speeds");
    }
}
