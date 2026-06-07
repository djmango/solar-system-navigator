//! Interactive maneuver-node editing: time-mapped trajectory preview and
//! click-to-place / click-to-select handling (KSP-style map planning).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{CelestialBody, FixedBody, Mass, Position, Velocity};
use crate::physics::BodyState;
use crate::planner::{add_node_at_time, select_node};
use crate::resources::{
    CameraInputState, GameUx, MapViewMode, PhysicsConstants, PlannerPreview, RoutePlanner,
    SimulationClock,
};
use crate::truth::{TRUTH_INTEGRATION_DT, estimate_orbital_period, predict_target_path_nbody};

/// Pixel radius for selecting an existing node marker.
const NODE_PICK_PX: f32 = 24.0;
/// Pixel radius for placing a node on the trajectory polyline.
const PATH_PICK_PX: f32 = 24.0;
/// Click vs drag threshold in map mode.
const CLICK_PX: f32 = 8.0;

type PlannerBodyQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static CelestialBody,
        &'static Mass,
        &'static Position,
        &'static Velocity,
        Option<&'static FixedBody>,
    ),
>;

/// Recompute the active vessel's time-mapped predicted path each frame so the
/// renderer and the click handler share one source of truth.
pub fn update_planner_preview(
    planner: Res<RoutePlanner>,
    map_mode: Res<MapViewMode>,
    game_ux: Res<GameUx>,
    clock: Res<SimulationClock>,
    physics: Res<PhysicsConstants>,
    bodies: PlannerBodyQuery,
    mut preview: ResMut<PlannerPreview>,
) {
    let show = planner.show_previews || map_mode.active || game_ux.show_system_orbits;
    if !show {
        preview.path.clear();
        preview.vessel = None;
        return;
    }

    let central_name = planner.central_body.clone().or_else(|| {
        bodies
            .iter()
            .find(|(_, _, _, _, fixed)| fixed.is_some())
            .map(|(c, _, _, _, _)| c.name.clone())
    });
    let target_name = planner.target_body.clone().or_else(|| {
        bodies
            .iter()
            .find(|(_, _, _, _, fixed)| fixed.is_none())
            .map(|(c, _, _, _, _)| c.name.clone())
    });

    let (Some(central_name), Some(target_name)) = (central_name, target_name) else {
        preview.path.clear();
        preview.vessel = None;
        return;
    };

    let states: Vec<BodyState> = bodies
        .iter()
        .map(|(c, m, p, v, fixed)| BodyState {
            name: c.name.clone(),
            position: p.0,
            velocity: v.0,
            mass: m.0,
            fixed: fixed.is_some(),
        })
        .collect();

    let central = states.iter().find(|b| b.name == central_name);
    let target = states.iter().find(|b| b.name == target_name);
    let (Some(central), Some(target)) = (central, target) else {
        preview.path.clear();
        preview.vessel = None;
        return;
    };

    let period =
        estimate_orbital_period(target.position, central.position, central.mass, physics.g);
    let step = planner.preview_step.max(TRUTH_INTEGRATION_DT * 0.25);
    // Cap the horizon to a few orbits so the polyline stays dense enough to pick.
    let horizon = planner
        .preview_horizon
        .min(period.max(step * 64.0) * 3.0)
        .max(step * 8.0);

    let path = predict_target_path_nbody(
        &states,
        &central_name,
        &target_name,
        &planner.nodes,
        clock.time,
        horizon,
        step,
        physics.g,
        physics.softening,
    );

    preview.path = path;
    preview.start_time = clock.time;
    preview.step = step;
    preview.vessel = Some(target_name);
    preview.central = Some(central_name);
    preview.period = period;
}

/// Place or select maneuver nodes by clicking the vessel trajectory in map mode.
pub fn node_pick(
    mouse: Res<ButtonInput<MouseButton>>,
    map_mode: Res<MapViewMode>,
    mut camera_input: ResMut<CameraInputState>,
    clock: Res<SimulationClock>,
    preview: Res<PlannerPreview>,
    primary: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    ui_interactions: Query<&Interaction>,
    mut planner: ResMut<RoutePlanner>,
) {
    if !map_mode.active {
        return;
    }

    if mouse.just_pressed(MouseButton::Left) {
        camera_input.left_drag_pixels = 0.0;
    }
    if mouse.pressed(MouseButton::Left) {
        // Accumulated by `map_drag_accumulator` so click vs drag is distinguishable.
    }
    if !mouse.just_released(MouseButton::Left) {
        return;
    }

    let drag = camera_input.left_drag_pixels;
    camera_input.left_drag_pixels = 0.0;
    if drag > CLICK_PX {
        return;
    }

    let pointer_over_ui = ui_interactions
        .iter()
        .any(|i| matches!(i, Interaction::Pressed | Interaction::Hovered));
    if pointer_over_ui {
        return;
    }

    let Ok(window) = primary.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };

    // 1) Prefer selecting an existing node marker near the cursor.
    let mut best_node: Option<(f32, u32)> = None;
    for node in &planner.nodes {
        if node.executed || node.time < clock.time {
            continue;
        }
        let Some(world) = preview.position_at_time(node.time) else {
            continue;
        };
        let Ok(screen) = camera.world_to_viewport(cam_transform, world) else {
            continue;
        };
        let d = screen.distance(cursor);
        if d <= NODE_PICK_PX && best_node.as_ref().is_none_or(|(bd, _)| d < *bd) {
            best_node = Some((d, node.id));
        }
    }
    if let Some((_, id)) = best_node {
        select_node(&mut planner, id);
        return;
    }

    // 2) Otherwise place a new node at the nearest point on the trajectory.
    if preview.path.is_empty() || preview.step <= 0.0 {
        return;
    }
    let mut best_point: Option<(f32, usize)> = None;
    for (idx, world) in preview.path.iter().enumerate() {
        let Ok(screen) = camera.world_to_viewport(cam_transform, *world) else {
            continue;
        };
        let d = screen.distance(cursor);
        if best_point.as_ref().is_none_or(|(bd, _)| d < *bd) {
            best_point = Some((d, idx));
        }
    }
    if let Some((d, idx)) = best_point
        && d <= PATH_PICK_PX
    {
        let time = preview.start_time + idx as f32 * preview.step;
        add_node_at_time(&mut planner, time);
    }
}

/// Accumulate cursor travel while LMB is held in map mode (for click vs drag).
pub fn map_drag_accumulator(
    map_mode: Res<MapViewMode>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<bevy::input::mouse::MouseMotion>,
    mut camera_input: ResMut<CameraInputState>,
) {
    if !map_mode.active || !mouse.pressed(MouseButton::Left) {
        motion.clear();
        return;
    }
    for event in motion.read() {
        camera_input.left_drag_pixels += event.delta.length();
    }
}
