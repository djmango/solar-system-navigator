use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

use crate::astro::AU;
use crate::components::{CelestialBody, Position};
use crate::resources::{EditorState, MapViewMode};

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
            min_radius: 1.0e9,
            max_radius: 50.0 * AU,
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

pub fn focus_camera_on_selection(
    map_mode: Res<MapViewMode>,
    editor: Res<EditorState>,
    bodies: Query<(&CelestialBody, &Position)>,
    mut cameras: Query<&mut OrbitCamera, With<Camera3d>>,
    time: Res<Time>,
) {
    if map_mode.active {
        return;
    }
    let Ok(mut orbit) = cameras.single_mut() else {
        return;
    };
    let Some(name) = &editor.selected_name else {
        return;
    };
    let Some((_, position)) = bodies.iter().find(|(b, _)| &b.name == name) else {
        return;
    };
    let target = position.0;
    let t = (time.delta_secs() * 2.0).clamp(0.0, 1.0);
    orbit.focus = orbit.focus.lerp(target, t);
}

pub fn orbit_camera_system(
    map_mode: Res<MapViewMode>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cameras: Query<(&mut Transform, &mut OrbitCamera), With<Camera3d>>,
) {
    if map_mode.active {
        return;
    }

    let Ok((mut transform, mut orbit)) = cameras.single_mut() else {
        return;
    };

    let pan_scale = (orbit.radius * 0.002).max(1.0e6);

    let rotating =
        mouse_button.pressed(MouseButton::Right) || mouse_button.pressed(MouseButton::Middle);

    for event in mouse_motion.read() {
        if rotating && !mouse_button.pressed(MouseButton::Middle) {
            orbit.yaw -= event.delta.x * 0.004;
            orbit.pitch += event.delta.y * 0.004;
            orbit.pitch = orbit.pitch.clamp(0.08, std::f32::consts::FRAC_PI_2 - 0.08);
        } else if mouse_button.pressed(MouseButton::Middle) {
            let right = *transform.right();
            let up = *transform.up();
            orbit.focus +=
                right * -event.delta.x * pan_scale * 0.001 + up * event.delta.y * pan_scale * 0.001;
        }
    }

    for event in mouse_wheel.read() {
        let scroll = event.y;
        orbit.radius *= 1.0 - scroll * 0.08;
    }

    if keyboard.pressed(KeyCode::KeyW) {
        orbit.radius *= 0.98;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        orbit.radius *= 1.02;
    }
    orbit.radius = orbit.radius.clamp(orbit.min_radius, orbit.max_radius);

    *transform = orbit_camera_transform(&orbit);
}
