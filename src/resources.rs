use bevy::prelude::*;

use crate::scenario::Scenario;

#[derive(Resource, Debug, Clone)]
pub struct PhysicsConstants {
    pub g: f32,
    pub softening: f32,
}

impl Default for PhysicsConstants {
    fn default() -> Self {
        Self {
            g: 1.0,
            softening: 10.0,
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
            speed: 1.0,
            ticks_per_frame: 2,
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

#[derive(Resource, Debug, Clone, Default)]
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
                    label: "Inner System".to_string(),
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
