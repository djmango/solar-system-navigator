use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

use crate::astro::AU;
use crate::components::{CelestialBody, Position, Velocity, VisualRadius};
use crate::components::{FixedBody, Mass};
use crate::orbit::{self, RelativeState};
use crate::resources::{
    CameraInputState, EditorState, MapViewMode, PhysicsConstants, RoutePlanner,
};

#[derive(Component)]
pub struct OrbitCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub min_radius: f32,
    pub max_radius: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            radius: 2.5 * AU,
            yaw: 0.7,
            pitch: 0.4,
            min_radius: 5.0e7,
            max_radius: 80.0 * AU,
        }
    }
}

pub fn spawn_camera(mut commands: Commands) {
    let orbit = OrbitCamera::default();
    commands.spawn((Camera3d::default(), orbit_camera_transform(&orbit), orbit));
}

pub fn orbit_camera_transform_public(orbit: &OrbitCamera) -> Transform {
    orbit_camera_transform(orbit)
}

fn orbit_camera_transform(orbit: &OrbitCamera) -> Transform {
    let pitch = orbit.pitch.clamp(0.08, std::f32::consts::FRAC_PI_2 - 0.08);
    let horizontal = orbit.radius * pitch.cos();
    let offset = Vec3::new(
        horizontal * orbit.yaw.sin(),
        orbit.radius * pitch.sin(),
        horizontal * orbit.yaw.cos(),
    );
    Transform::from_translation(orbit.focus + offset).looking_at(orbit.focus, Vec3::Y)
}

pub fn demo_orbit_camera(
    time: Res<Time>,
    recorder: Option<Res<crate::resources::DemoRecorder>>,
    mut cameras: Query<&mut OrbitCamera, With<Camera3d>>,
) {
    if recorder.is_none() {
        return;
    }
    let Ok(mut orbit) = cameras.single_mut() else {
        return;
    };
    orbit.yaw += time.delta_secs() * 0.2;
    orbit.pitch = 0.35 + (time.elapsed_secs() * 0.05).sin() * 0.08;
}

pub fn select_body(editor: &mut EditorState, name: &str, frame: bool) {
    let changed = editor.selected_name.as_deref() != Some(name);
    editor.selected_name = Some(name.to_string());
    editor.selection_changed = true;
    editor.velocity_dirty = false;
    if frame || changed {
        editor.frame_camera = true;
    }
}

/// Camera distance that frames a body and its approximate orbit.
pub fn frame_distance_for_body(mu: f32, rel: RelativeState, visual_radius: f32) -> f32 {
    let r = rel.position.length().max(visual_radius);
    if let Some(elements) = orbit::elements_from_state(mu, rel)
        && elements.semi_major_axis.is_finite()
        && elements.semi_major_axis > 0.0
    {
        let a = elements.semi_major_axis;
        return (a * 2.8)
            .max(r * 2.2)
            .max(visual_radius * 20.0)
            .clamp(5.0e7, 60.0 * AU);
    }
    (r * 3.0).max(visual_radius * 25.0).clamp(5.0e7, 60.0 * AU)
}

pub fn focus_camera_on_selection(
    map_mode: Res<MapViewMode>,
    mut editor: ResMut<EditorState>,
    physics: Res<PhysicsConstants>,
    planner: Res<RoutePlanner>,
    bodies: Query<(
        &CelestialBody,
        &Mass,
        &Position,
        &Velocity,
        &VisualRadius,
        Option<&FixedBody>,
    )>,
    mut cameras: Query<&mut OrbitCamera, With<Camera3d>>,
    time: Res<Time>,
) {
    if map_mode.active || !editor.follow_selection {
        return;
    }
    let Ok(mut orbit) = cameras.single_mut() else {
        return;
    };
    let Some(name) = &editor.selected_name else {
        return;
    };
    let Some((_, _, position, velocity, visual, _)) =
        bodies.iter().find(|(b, _, _, _, _, _)| &b.name == name)
    else {
        return;
    };

    let central_name = planner.central_body.clone().or_else(|| {
        bodies
            .iter()
            .find(|(_, _, _, _, _, fixed)| fixed.is_some())
            .map(|(c, _, _, _, _, _)| c.name.clone())
    });
    let (central_pos, central_vel, central_mass) = central_name
        .as_ref()
        .and_then(|n| {
            bodies
                .iter()
                .find(|(b, _, _, _, _, _)| &b.name == n)
                .map(|(_, m, p, v, _, _)| (p.0, v.0, m.0))
        })
        .unwrap_or((Vec3::ZERO, Vec3::ZERO, crate::astro::M_SUN));
    let mu = physics.g * central_mass;

    let target = position.0;
    let t = (time.delta_secs() * 3.5).clamp(0.0, 1.0);
    orbit.focus = orbit.focus.lerp(target, t);

    if editor.frame_camera {
        let rel = RelativeState::new(position.0 - central_pos, velocity.0 - central_vel);
        let desired = if editor.target_frame_radius > 0.0 {
            editor.target_frame_radius
        } else {
            frame_distance_for_body(mu, rel, visual.0)
        };
        orbit.radius = orbit.radius.lerp(desired, t);
        if (orbit.radius - desired).abs() < desired * 0.04 {
            editor.frame_camera = false;
            editor.target_frame_radius = 0.0;
        }
    }
}

pub fn orbit_camera_system(
    map_mode: Res<MapViewMode>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_input: ResMut<CameraInputState>,
    mut cameras: Query<(&mut Transform, &mut OrbitCamera), With<Camera3d>>,
) {
    if map_mode.active {
        return;
    }

    let Ok((mut transform, mut orbit)) = cameras.single_mut() else {
        return;
    };

    let pan_scale = (orbit.radius * 0.0015).max(1.0e6);
    let orbiting = mouse_button.pressed(MouseButton::Left);
    let panning =
        mouse_button.pressed(MouseButton::Right) || mouse_button.pressed(MouseButton::Middle);

    for event in mouse_motion.read() {
        let delta_len = event.delta.length();
        if orbiting {
            camera_input.left_drag_pixels += delta_len;
            orbit.yaw -= event.delta.x * 0.004;
            orbit.pitch += event.delta.y * 0.004;
            orbit.pitch = orbit.pitch.clamp(0.08, std::f32::consts::FRAC_PI_2 - 0.08);
        } else if panning {
            let right = *transform.right();
            let up = *transform.up();
            orbit.focus +=
                right * -event.delta.x * pan_scale * 0.001 + up * event.delta.y * pan_scale * 0.001;
        }
    }

    if mouse_button.just_released(MouseButton::Left) && camera_input.left_drag_pixels < 4.0 {
        // Small movement — treated as click; pick handled in interaction.rs on same frame.
    }

    for event in mouse_wheel.read() {
        let scroll = event.y;
        let factor = 1.0 - scroll * 0.12;
        orbit.radius *= factor;
    }

    if keyboard.pressed(KeyCode::KeyW) {
        orbit.radius *= 0.97;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        orbit.radius *= 1.03;
    }
    orbit.radius = orbit.radius.clamp(orbit.min_radius, orbit.max_radius);

    *transform = orbit_camera_transform(&orbit);
}
