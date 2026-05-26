use bevy::prelude::*;

use crate::scenario::Scenario;
use crate::transfer::HohmannTransfer;

#[derive(Resource, Debug, Clone)]
pub struct PhysicsConstants {
    pub g: f32,
    pub softening: f32,
}

impl Default for PhysicsConstants {
    fn default() -> Self {
        Self {
            g: crate::astro::G,
            softening: crate::astro::DEFAULT_SOFTENING,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct SimulationControl {
    pub speed: f32,
    pub ticks_per_frame: u32,
    pub paused: bool,
    pub step_once: bool,
}

impl Default for SimulationControl {
    fn default() -> Self {
        Self {
            speed: crate::astro::DEFAULT_TIME_WARP,
            ticks_per_frame: 4,
            paused: false,
            step_once: false,
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct SimulationDiagnostics {
    pub kinetic_energy: f32,
    pub potential_energy: f32,
    pub total_energy: f32,
    pub body_count: u32,
}

#[derive(Resource, Debug, Clone)]
pub struct ActiveScenario {
    pub name: String,
    pub file_path: String,
    pub template: Scenario,
}

#[derive(Resource, Debug, Clone)]
pub struct SimulationClock {
    pub time: f32,
}

impl Default for SimulationClock {
    fn default() -> Self {
        Self { time: 0.0 }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct MapViewMode {
    pub active: bool,
    pub yaw: f32,
}

impl Default for MapViewMode {
    fn default() -> Self {
        Self {
            active: false,
            yaw: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ManeuverNode {
    pub time: f32,
    pub prograde: f32,
    pub normal: f32,
    pub radial: f32,
    pub executed: bool,
}

impl ManeuverNode {
    pub fn delta_v_magnitude(&self) -> f32 {
        (self.prograde * self.prograde + self.normal * self.normal + self.radial * self.radial)
            .sqrt()
    }
}

#[derive(Resource, Debug, Clone)]
pub struct RoutePlanner {
    pub nodes: Vec<ManeuverNode>,
    pub central_body: Option<String>,
    pub target_body: Option<String>,
    pub draft_prograde: f32,
    pub draft_normal: f32,
    pub draft_radial: f32,
    pub default_burn_offset: f32,
    pub preview_horizon: f32,
    pub preview_step: f32,
    pub show_previews: bool,
    /// Auto-switch central body by SOI around the target vessel.
    pub soi_auto: bool,
    pub hohmann_target_radius: f32,
    pub last_hohmann: Option<HohmannTransfer>,
}

impl Default for RoutePlanner {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            central_body: None,
            target_body: None,
            draft_prograde: 500.0,
            draft_normal: 0.0,
            draft_radial: 0.0,
            default_burn_offset: crate::astro::DAY,
            preview_horizon: crate::astro::YEAR,
            preview_step: 3600.0,
            show_previews: false,
            soi_auto: true,
            hohmann_target_radius: crate::astro::AU * 1.524,
            last_hohmann: None,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct EditorState {
    pub selected_name: Option<String>,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub velocity_z: f32,
    pub probe_delta_v: f32,
    /// When true, slider edits are pushed to the selected body's velocity.
    pub velocity_dirty: bool,
    /// When true, selection just changed — sync sliders from body once.
    pub selection_changed: bool,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            selected_name: None,
            velocity_x: 0.0,
            velocity_y: 0.0,
            velocity_z: 0.0,
            probe_delta_v: 500.0,
            velocity_dirty: false,
            selection_changed: false,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct ScenarioCatalog {
    pub entries: Vec<ScenarioEntry>,
}

#[derive(Debug, Clone)]
pub struct ScenarioEntry {
    pub label: String,
    pub path: String,
}

impl Default for ScenarioCatalog {
    fn default() -> Self {
        Self {
            entries: vec![
                ScenarioEntry {
                    label: "Inner System (SI)".to_string(),
                    path: "scenarios/default.toml".to_string(),
                },
                ScenarioEntry {
                    label: "Apollo 11 (simplified)".to_string(),
                    path: "missions/apollo11.toml".to_string(),
                },
                ScenarioEntry {
                    label: "OSIRIS-REx (simplified)".to_string(),
                    path: "missions/osiris_rex.toml".to_string(),
                },
            ],
        }
    }
}

#[derive(Resource, Debug)]
pub struct DemoRecorder {
    /// How long the app runs before exiting in demo mode (seconds).
    pub duration_secs: f32,
}

#[derive(Resource)]
pub struct WorldAssets {
    pub sphere_mesh: Handle<Mesh>,
}

#[derive(Resource, Default)]
pub struct BodyTextureCache {
    pub handles: std::collections::HashMap<String, Handle<Image>>,
}

#[derive(Message)]
pub struct ReloadScenario;

#[derive(Message)]
pub struct SpawnProbe;

impl DemoRecorder {
    pub fn from_env() -> Option<Self> {
        if std::env::var("SOLAR_DEMO_RECORD").ok().as_deref() != Some("1") {
            return None;
        }
        let duration_secs = std::env::var("SOLAR_DEMO_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(20.0);
        Some(Self { duration_secs })
    }
}
