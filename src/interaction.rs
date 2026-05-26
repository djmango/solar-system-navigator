//! Mouse picking, selection, and game-style focus framing.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::astro::AU;
use crate::camera::{frame_distance_for_body, select_body};
use crate::components::{CelestialBody, FixedBody, Mass, Position, Velocity, VisualRadius};
use crate::orbit::RelativeState;
use crate::resources::{
    CameraInputState, EditorState, HoveredBody, MapViewMode, PhysicsConstants, RoutePlanner,
};

const CLICK_DRAG_THRESHOLD_PX: f32 = 8.0;
const DOUBLE_CLICK_SECS: f32 = 0.35;

#[derive(Resource, Default)]
pub struct ClickTracker {
    last_time: f32,
    last_body: Option<String>,
}

type BodyQueryItem<'a> = (
    &'a CelestialBody,
    &'a Mass,
    &'a Position,
    &'a Velocity,
    &'a VisualRadius,
    Option<&'a FixedBody>,
);

pub fn update_body_hover(
    map_mode: Res<MapViewMode>,
    mouse: Res<ButtonInput<MouseButton>>,
    camera_input: Res<CameraInputState>,
    primary: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    bodies: Query<BodyQueryItem<'_>>,
    ui_interactions: Query<&Interaction>,
    mut hovered: ResMut<HoveredBody>,
) {
    if map_mode.active || mouse.pressed(MouseButton::Left) {
        hovered.0 = None;
        return;
    }

    let pointer_over_ui = ui_interactions
        .iter()
        .any(|i| matches!(i, Interaction::Pressed | Interaction::Hovered));
    if pointer_over_ui || camera_input.left_drag_pixels > 2.0 {
        hovered.0 = None;
        return;
    }

    let Ok(window) = primary.single() else {
        hovered.0 = None;
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        hovered.0 = None;
        return;
    };
    let Ok((camera, cam_transform)) = cameras.single() else {
        hovered.0 = None;
        return;
    };
    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor) else {
        hovered.0 = None;
        return;
    };

    let origin = ray.origin;
    let dir = ray.direction.normalize();
    let mut best: Option<(f32, String)> = None;
    for (body, _, position, _, visual, _) in &bodies {
        let pick_r = visual.0 * 2.5;
        if let Some(t) = ray_sphere_hit(origin, dir, position.0, pick_r)
            && best.as_ref().is_none_or(|(best_t, _)| t < *best_t)
        {
            best = Some((t, body.name.clone()));
        }
    }
    hovered.0 = best.map(|(_, name)| name);
}

pub fn body_pick_on_click(
    mouse: Res<ButtonInput<MouseButton>>,
    mut camera_input: ResMut<CameraInputState>,
    mut editor: ResMut<EditorState>,
    mut click_tracker: ResMut<ClickTracker>,
    time: Res<Time>,
    map_mode: Res<MapViewMode>,
    physics: Res<PhysicsConstants>,
    planner: Res<RoutePlanner>,
    primary: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    bodies: Query<BodyQueryItem<'_>>,
    ui_interactions: Query<&Interaction>,
) {
    if map_mode.active {
        return;
    }

    let pointer_over_ui = ui_interactions
        .iter()
        .any(|i| matches!(i, Interaction::Pressed | Interaction::Hovered));
    if pointer_over_ui {
        if mouse.just_released(MouseButton::Left) {
            camera_input.left_drag_pixels = 0.0;
        }
        return;
    }

    if !mouse.just_released(MouseButton::Left) {
        return;
    }

    if camera_input.left_drag_pixels > CLICK_DRAG_THRESHOLD_PX {
        camera_input.left_drag_pixels = 0.0;
        return;
    }
    camera_input.left_drag_pixels = 0.0;

    let Ok(window) = primary.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor) else {
        return;
    };

    let origin = ray.origin;
    let dir = ray.direction.normalize();

    let mut best: Option<(f32, String)> = None;
    for (body, _, position, _, visual, _) in &bodies {
        let pick_r = visual.0 * 2.5;
        if let Some(t) = ray_sphere_hit(origin, dir, position.0, pick_r)
            && best.as_ref().is_none_or(|(best_t, _)| t < *best_t)
        {
            best = Some((t, body.name.clone()));
        }
    }

    let Some((_, name)) = best else {
        return;
    };

    let double_click = click_tracker
        .last_body
        .as_ref()
        .is_some_and(|prev| prev == &name)
        && time.elapsed_secs() - click_tracker.last_time < DOUBLE_CLICK_SECS;

    click_tracker.last_time = time.elapsed_secs();
    click_tracker.last_body = Some(name.clone());

    select_body(&mut editor, &name, true);
    editor.target_frame_radius = 0.0;
    if double_click {
        frame_selected_body(&mut editor, &physics, &planner, &bodies, &name);
    }
}

fn ray_sphere_hit(origin: Vec3, dir: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    if radius <= 0.0 {
        return None;
    }
    let oc = origin - center;
    let b = oc.dot(dir);
    let c = oc.dot(oc) - radius * radius;
    let disc = b * b - c;
    if disc < 0.0 {
        return None;
    }
    let sqrt_disc = disc.sqrt();
    let t0 = -b - sqrt_disc;
    let t1 = -b + sqrt_disc;
    if t0 > 0.0 {
        Some(t0)
    } else if t1 > 0.0 {
        Some(t1)
    } else {
        None
    }
}

pub fn frame_camera_hotkey(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut editor: ResMut<EditorState>,
    map_mode: Res<MapViewMode>,
    physics: Res<PhysicsConstants>,
    planner: Res<RoutePlanner>,
    bodies: Query<BodyQueryItem<'_>>,
) {
    if map_mode.active {
        return;
    }
    if !(keyboard.just_pressed(KeyCode::KeyF) || keyboard.just_pressed(KeyCode::NumpadEnter)) {
        return;
    }
    let Some(name) = editor.selected_name.clone() else {
        return;
    };
    frame_selected_body(&mut editor, &physics, &planner, &bodies, &name);
}

/// Focus the scenario primary (fixed) body — Sun in heliocentric presets, Earth in Apollo, etc.
pub fn focus_primary_hotkey(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut editor: ResMut<EditorState>,
    physics: Res<PhysicsConstants>,
    planner: Res<RoutePlanner>,
    bodies: Query<BodyQueryItem<'_>>,
) {
    if !keyboard.just_pressed(KeyCode::Home) {
        return;
    }
    let primary = bodies
        .iter()
        .find(|(_, _, _, _, _, fixed)| fixed.is_some())
        .or_else(|| {
            planner
                .central_body
                .as_ref()
                .and_then(|name| bodies.iter().find(|(b, _, _, _, _, _)| &b.name == name))
        });
    let Some((c, _, pos, vel, visual, _)) = primary else {
        return;
    };
    select_body(&mut editor, &c.name, true);
    editor.target_frame_radius =
        primary_frame_radius(&physics, &planner, &c.name, pos.0, vel.0, visual.0);
    editor.frame_camera = true;
}

fn primary_frame_radius(
    physics: &PhysicsConstants,
    planner: &RoutePlanner,
    primary_name: &str,
    pos: Vec3,
    vel: Vec3,
    visual: f32,
) -> f32 {
    if primary_name == "Sun" || planner.central_body.as_deref() == Some("Sun") {
        return 2.5 * AU;
    }
    let rel = RelativeState::new(pos, vel);
    frame_distance_for_body(physics.g * crate::astro::M_SUN, rel, visual).min(1.0e10)
}

fn central_state(
    planner: &RoutePlanner,
    bodies: &Query<BodyQueryItem<'_>>,
) -> Option<(Vec3, Vec3, f32)> {
    let name = planner.central_body.clone().or_else(|| {
        bodies
            .iter()
            .find(|(_, _, _, _, _, fixed)| fixed.is_some())
            .map(|(c, _, _, _, _, _)| c.name.clone())
    })?;
    let (_, mass, pos, vel, _, _) = bodies.iter().find(|(b, _, _, _, _, _)| b.name == name)?;
    Some((pos.0, vel.0, mass.0))
}

pub fn frame_selected_body(
    editor: &mut EditorState,
    physics: &PhysicsConstants,
    planner: &RoutePlanner,
    bodies: &Query<BodyQueryItem<'_>>,
    name: &str,
) {
    let Some((central_pos, central_vel, central_mass)) = central_state(planner, bodies) else {
        editor.frame_camera = true;
        editor.target_frame_radius = AU;
        return;
    };

    let mu = physics.g * central_mass;

    let Some((_, _, pos, vel, visual, _)) = bodies.iter().find(|(b, _, _, _, _, _)| b.name == name)
    else {
        editor.frame_camera = true;
        editor.target_frame_radius = AU;
        return;
    };

    let rel = RelativeState::new(pos.0 - central_pos, vel.0 - central_vel);
    editor.target_frame_radius = frame_distance_for_body(mu, rel, visual.0);
    editor.frame_camera = true;
}
