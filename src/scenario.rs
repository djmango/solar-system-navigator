use bevy::prelude::*;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Scenario {
    pub name: String,
    #[serde(default = "default_g")]
    pub g: f32,
    #[serde(default = "default_softening")]
    pub softening: f32,
    /// Scales auto-computed Hill SOI radii (1.0 = default).
    #[serde(default = "default_soi_scale")]
    pub soi_scale: f32,
    /// Multiplies `radius` for mesh display when `visual_radius` is omitted.
    #[serde(default = "default_visual_exaggeration")]
    pub visual_exaggeration: f32,
    pub bodies: Vec<BodyDef>,
}

fn default_soi_scale() -> f32 {
    1.0
}

fn default_g() -> f32 {
    crate::astro::G
}

fn default_softening() -> f32 {
    crate::astro::DEFAULT_SOFTENING
}

fn default_visual_exaggeration() -> f32 {
    1.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct BodyDef {
    pub name: String,
    pub mass: f32,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    /// Physical radius [m] (SOI, Hill sphere).
    pub radius: f32,
    /// Mesh display radius [m]; defaults to `radius * scenario.visual_exaggeration`.
    #[serde(default)]
    pub visual_radius: Option<f32>,
    pub color: [f32; 3],
    #[serde(default = "default_emissive")]
    pub emissive: f32,
    #[serde(default)]
    pub atmosphere: Option<[f32; 3]>,
    /// Asset path relative to `assets/` (e.g. `textures/earth.jpg`).
    #[serde(default)]
    pub texture: Option<String>,
    /// Override Hill SOI radius; auto-computed from mass and orbit if omitted.
    #[serde(default)]
    pub soi_radius: Option<f32>,
    #[serde(default)]
    pub fixed: bool,
    #[serde(default)]
    pub probe: bool,
}

fn default_emissive() -> f32 {
    0.8
}

impl BodyDef {
    pub fn position_vec3(&self) -> Vec3 {
        Vec3::from_array(self.position)
    }

    pub fn velocity_vec3(&self) -> Vec3 {
        Vec3::from_array(self.velocity)
    }

    pub fn color(&self) -> Color {
        Color::srgb(self.color[0], self.color[1], self.color[2])
    }

    pub fn atmosphere_color(&self) -> Option<Color> {
        self.atmosphere.map(|[r, g, b]| Color::srgb(r, g, b))
    }

    pub fn display_radius(&self, visual_exaggeration: f32) -> f32 {
        self.visual_radius
            .unwrap_or(self.radius * visual_exaggeration)
    }
}

pub fn load_scenario(path: impl AsRef<Path>) -> Result<Scenario, String> {
    let path = path.as_ref();
    let contents =
        fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    toml::from_str(&contents).map_err(|e| format!("Failed to parse {}: {e}", path.display()))
}

pub fn scenario_asset_path(relative: &str) -> String {
    format!("assets/{relative}")
}

/// Validate circular heliocentric speed for non-fixed bodies around the primary.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scenario_loads_and_has_circular_speeds() {
        let path = scenario_asset_path("scenarios/default.toml");
        let scenario = load_scenario(&path).expect("default scenario should parse");
        assert!(scenario.bodies.len() >= 5);
        validate_circular_speeds(&scenario).expect("inner system circular speeds");
    }

    #[test]
    fn mission_scenarios_load() {
        for rel in ["missions/apollo11.toml", "missions/osiris_rex.toml"] {
            let path = scenario_asset_path(rel);
            let scenario = load_scenario(&path).unwrap_or_else(|e| panic!("{rel}: {e}"));
            assert!(!scenario.bodies.is_empty());
        }
    }
}
