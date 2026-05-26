#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod camera;
mod components;
mod demo;
mod input;
mod physics;
mod resources;
mod scenario;
mod spawn;
mod ui;

use bevy::prelude::*;

use camera::{demo_orbit_camera, orbit_camera_system, spawn_camera};
use demo::{demo_auto_exit, demo_scenario_cycler};
use input::keyboard_controls;
use physics::orbital_physics;
use resources::{
    ActiveScenario, DemoRecorder, PhysicsConstants, ReloadScenario, ScenarioCatalog,
    SimulationControl, SimulationDiagnostics, SpawnProbe,
};
use scenario::{load_scenario, scenario_asset_path};
use spawn::{
    apply_editor_selection, draw_orbit_trails, reload_scenario, spawn_probe, spawn_world,
};
use ui::{spawn_ui, ui_system, update_hud_text};

fn main() {
    let initial_path = scenario_asset_path("scenarios/default.toml");
    let template = load_scenario(&initial_path).expect("default scenario must load");

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Solar System Navigator".into(),
            resolution: (1280, 720).into(),
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.04)))
    .init_resource::<SimulationControl>()
    .init_resource::<PhysicsConstants>()
    .init_resource::<SimulationDiagnostics>()
    .init_resource::<ScenarioCatalog>()
    .insert_resource(ActiveScenario {
        name: template.name.clone(),
        file_path: "scenarios/default.toml".to_string(),
        template,
    })
    .insert_resource(resources::EditorState::default());

    if let Some(recorder) = DemoRecorder::from_env() {
        app.insert_resource(recorder);
    }

    app.add_message::<ReloadScenario>()
        .add_message::<SpawnProbe>()
        .add_systems(Startup, (spawn_camera, spawn_world, spawn_ui))
        .add_systems(
            Update,
            (
                orbital_physics,
                apply_editor_selection,
                draw_orbit_trails,
                orbit_camera_system,
                demo_orbit_camera,
                ui_system,
                update_hud_text,
                keyboard_controls,
                demo_scenario_cycler,
                demo_auto_exit,
            ),
        )
        .add_systems(Update, (reload_scenario, spawn_probe))
        .run();
}
