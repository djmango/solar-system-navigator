//! Mouse picking, selection, and game-style focus framing.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::astro::AU;
use crate::camera::{frame_distance_for_body, select_body};
use crate::components::{CelestialBody, Position, VisualRadius};
use crate::orbit::RelativeState;
use crate::resources::{CameraInputState, EditorState, MapViewMode, PhysicsConstants};
const CLICK_DRAG_THRESHOLD_PX: f32 = 8.0;
const DOUBLE_CLICK_SECS: f32 = 0.35;

#[derive(Resource, Default)]
pub struct ClickTracker {
    last_time: f32,
    last_body: Option<String>,
}

pub fn body_pick_on_click(
    mouse: Res<ButtonInput<MouseButton>>,
    mut camera_input: ResMut<CameraInputState>,
    mut editor: ResMut<EditorState>,
    mut click_tracker: ResMut<ClickTracker>,
    time: Res<Time>,
    map_mode: Res<MapViewMode>,
    physics: Res<PhysicsConstants>,
    active: Res<crate::resources::ActiveScenario>,
    primary: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    bodies: Query<(&CelestialBody, &Position, &VisualRadius)>,
    ui_interactions: Query<&Interaction>,
) {
    if map_mode.active {
        return;
    }

    let pointer_over_ui = ui_interactions.iter().any(|i| {
        matches!(
            i,
            Interaction::Pressed | Interaction::Hovered
        )
    });
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

    let mut best: Option<(f32, String, f32)> = None;
    for (body, position, visual) in &bodies {
        let center = position.0;
        let pick_r = visual.0 * 2.5;
        if let Some(t) = ray_sphere_hit(origin, dir, center, pick_r)
            && best.as_ref().is_none_or(|(best_t, _, _)| t < *best_t)
        {
            best = Some((t, body.name.clone(), visual.0));
        }
    }

    let Some((_, name, visual_r)) = best else {
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
        frame_selected_body(&mut editor, &physics, &active, &bodies, &name);
    }
    let _ = visual_r;
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
    active: Res<crate::resources::ActiveScenario>,
    bodies: Query<(&CelestialBody, &Position, &VisualRadius)>,
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
    frame_selected_body(&mut editor, &physics, &active, &bodies, &name);
}

pub fn focus_sun_hotkey(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut editor: ResMut<EditorState>,
    active: Res<crate::resources::ActiveScenario>,
) {
    if !keyboard.just_pressed(KeyCode::Home) {
        return;
    }
    let sun = active
        .template
        .bodies
        .iter()
        .find(|b| b.fixed)
        .map(|b| b.name.clone())
        .unwrap_or_else(|| "Sun".to_string());
    select_body(&mut editor, &sun, true);
    editor.target_frame_radius = 2.5 * AU;
}

fn frame_selected_body(
    editor: &mut EditorState,
    physics: &PhysicsConstants,
    active: &crate::resources::ActiveScenario,
    bodies: &Query<(&CelestialBody, &Position, &VisualRadius)>,
    name: &str,
) {
    let primary = active.template.bodies.iter().find(|b| b.fixed);
    let (primary_mass, primary_pos) = primary
        .map(|p| (p.mass, p.position_vec3()))
        .unwrap_or((crate::astro::M_SUN, Vec3::ZERO));
    let mu = physics.g * primary_mass;

    let Some((_, pos, visual)) = bodies.iter().find(|(b, _, _)| b.name == name) else {
        editor.frame_camera = true;
        editor.target_frame_radius = AU;
        return;
    };

    let rel_vel = active
        .template
        .bodies
        .iter()
        .find(|b| b.name == name)
        .map(|b| b.velocity_vec3())
        .unwrap_or(Vec3::ZERO);
    let primary_vel = primary.map(|p| p.velocity_vec3()).unwrap_or(Vec3::ZERO);
    let rel = RelativeState::new(pos.0 - primary_pos, rel_vel - primary_vel);
    editor.target_frame_radius = frame_distance_for_body(mu, rel, visual.0);
    editor.frame_camera = true;
}
