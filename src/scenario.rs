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
    pub bodies: Vec<BodyDef>,
}

fn default_g() -> f32 {
    1.0
}

fn default_softening() -> f32 {
    10.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct BodyDef {
    pub name: String,
    pub mass: f32,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub radius: f32,
    pub color: [f32; 3],
    #[serde(default = "default_emissive")]
    pub emissive: f32,
    #[serde(default)]
    pub atmosphere: Option<[f32; 3]>,
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
