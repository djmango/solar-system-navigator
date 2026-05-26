#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod camera;
mod components;
mod demo;
mod input;
mod maneuver;
mod map_view;
mod orbit;
mod physics;
mod planner;
mod soi;
mod transfer;
mod resources;
mod scenario;
mod spawn;
mod ui;

use bevy::prelude::*;

use camera::{demo_orbit_camera, focus_camera_on_selection, orbit_camera_system, spawn_camera};
use demo::{demo_auto_exit, demo_scenario_cycler, demo_simulation_tuning};
use input::keyboard_controls;
use physics::orbital_physics;
use maneuver::execute_maneuver_burns;
use map_view::{draw_orbit_previews, map_mode_camera, toggle_map_mode};
use planner::{
    advance_simulation_clock, sync_route_planner_targets, update_soi_central_body,
};
use resources::{
    ActiveScenario, BodyTextureCache, DemoRecorder, MapViewMode, PhysicsConstants, ReloadScenario,
    RoutePlanner, ScenarioCatalog, SimulationClock, SimulationControl, SimulationDiagnostics,
    SpawnProbe,
};
use scenario::{load_scenario, scenario_asset_path};
use spawn::{
    apply_editor_velocity, draw_orbit_trails, reload_scenario, spawn_probe, spawn_world,
    sync_editor_from_selection, update_selection_visuals,
};
use ui::{spawn_ui, ui_system, update_hud_text};

fn main() {
    let initial_path = scenario_asset_path("scenarios/default.toml");
    let template = match load_scenario(&initial_path) {
        Ok(t) => t,
        Err(err) => {
            eprintln!("Fatal: could not load default scenario at {initial_path}: {err}");
            std::process::exit(1);
        }
    };

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
    .init_resource::<SimulationClock>()
    .init_resource::<RoutePlanner>()
    .init_resource::<MapViewMode>()
    .init_resource::<BodyTextureCache>()
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
        .add_systems(Startup, (spawn_camera, spawn_world, spawn_ui).chain())
        .add_systems(
            Update,
            (
                demo_orbit_camera,
                focus_camera_on_selection,
                orbit_camera_system,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                sync_route_planner_targets,
                update_soi_central_body,
                sync_editor_from_selection,
                apply_editor_velocity,
                execute_maneuver_burns,
                orbital_physics,
                advance_simulation_clock,
                update_selection_visuals,
                draw_orbit_trails,
                draw_orbit_previews,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                toggle_map_mode,
                map_mode_camera,
            ),
        )
        .add_systems(
            Update,
            (
                ui_system,
                update_hud_text,
                keyboard_controls,
                demo_simulation_tuning,
                demo_scenario_cycler,
                demo_auto_exit,
            ),
        )
        .add_systems(Update, (reload_scenario, spawn_probe))
        .run();
}
