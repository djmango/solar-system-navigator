#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod astro;
mod camera;
mod components;
mod demo;
mod input;
mod interaction;
mod map_view;
mod orbit;
mod physics;
mod planner;
mod resources;
pub mod scenario;
mod soi;
mod spawn;
mod transfer;
mod truth;
mod ui;

use bevy::prelude::*;

use camera::{demo_orbit_camera, focus_camera_on_selection, orbit_camera_system, spawn_camera};
use demo::{demo_auto_exit, demo_scenario_cycler, demo_simulation_tuning};
use input::keyboard_controls;
use interaction::{
    ClickTracker, body_pick_on_click, focus_primary_hotkey, frame_camera_hotkey, update_body_hover,
};
use map_view::{draw_orbit_previews, map_mode_camera, toggle_map_mode};
use physics::orbital_physics;
use planner::{sync_route_planner_targets, update_soi_central_body};
use resources::{
    ActiveScenario, BodyTextureCache, CameraInputState, DemoRecorder, GameUx, HoveredBody,
    MapViewMode, PendingTruthPaths, PhysicsConstants, ReloadScenario, RoutePlanner,
    ScenarioCatalog, SimulationClock, SimulationControl, SimulationDiagnostics, SpawnProbe,
};
use scenario::Scenario;
use spawn::{
    apply_editor_velocity, apply_pending_truth_paths, draw_orbit_trails, reload_scenario,
    spawn_probe, spawn_world, sync_editor_from_selection, update_selection_visuals,
};
use ui::{spawn_ui, ui_system, update_hud_text};

/// Build and run the Bevy app (native desktop or browser WASM).
pub fn run_app(template: Scenario) {
    let mut window = Window {
        title: "Solar System Navigator".into(),
        ..default()
    };
    #[cfg(not(target_arch = "wasm32"))]
    {
        window.resolution = (1280, 720).into();
    }
    #[cfg(target_arch = "wasm32")]
    {
        window.fit_canvas_to_parent = true;
    }

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(window),
        ..default()
    }))
    .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.04)))
    .init_resource::<SimulationControl>()
    .init_resource::<SimulationClock>()
    .init_resource::<RoutePlanner>()
    .init_resource::<MapViewMode>()
    .init_resource::<GameUx>()
    .init_resource::<CameraInputState>()
    .init_resource::<ClickTracker>()
    .init_resource::<HoveredBody>()
    .init_resource::<PendingTruthPaths>()
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
        .add_systems(
            Startup,
            (
                spawn_camera,
                spawn_world,
                spawn_ui,
                apply_pending_truth_paths,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                demo_orbit_camera,
                orbit_camera_system,
                body_pick_on_click,
                focus_camera_on_selection,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (update_body_hover, frame_camera_hotkey, focus_primary_hotkey),
        )
        .add_systems(
            Update,
            (
                sync_route_planner_targets,
                update_soi_central_body,
                sync_editor_from_selection,
                apply_editor_velocity,
                orbital_physics,
                update_selection_visuals,
                draw_orbit_trails,
                draw_orbit_previews,
            )
                .chain(),
        )
        .add_systems(Update, (toggle_map_mode, map_mode_camera))
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
        .add_systems(
            Update,
            (reload_scenario, apply_pending_truth_paths, spawn_probe),
        )
        .run();
}

#[cfg(target_arch = "wasm32")]
use scenario::load_scenario_relative;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_start() {
    console_error_panic_hook::set_once();
    let template = load_scenario_relative("scenarios/default.toml")
        .expect("default scenario must be embedded for WASM");
    run_app(template);
}
