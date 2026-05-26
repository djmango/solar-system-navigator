//! KSP-style map mode: orthographic system view and predicted orbit gizmos.

use bevy::prelude::*;

use crate::components::{CelestialBody, FixedBody, Mass as BodyMass, Position, Probe, Velocity};
use crate::orbit::{self, RelativeState};
use crate::resources::{MapViewMode, PhysicsConstants, RoutePlanner, SimulationClock};
use crate::camera::OrbitCamera;

const MAP_RADIUS: f32 = 1400.0;
const MAP_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.02;

pub fn toggle_map_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut map_mode: ResMut<MapViewMode>,
    mut planner: ResMut<RoutePlanner>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        map_mode.active = !map_mode.active;
        planner.show_previews = map_mode.active || !planner.nodes.is_empty();
    }
}

pub fn map_mode_camera(
    map_mode: Res<MapViewMode>,
    bodies: Query<(&CelestialBody, &Position, Option<&FixedBody>)>,
    mut cameras: Query<(&mut Transform, &mut OrbitCamera), With<Camera3d>>,
) {
    if !map_mode.active {
        return;
    }

    let Ok((mut transform, mut orbit)) = cameras.single_mut() else {
        return;
    };

    let focus = bodies
        .iter()
        .find(|(_, _, fixed)| fixed.is_some())
        .map(|(_, p, _)| p.0)
        .unwrap_or(Vec3::ZERO);

    orbit.focus = focus;
    orbit.radius = MAP_RADIUS;
    orbit.pitch = MAP_PITCH;
    orbit.yaw = map_mode.yaw;
    *transform = crate::camera::orbit_camera_transform_public(&orbit);
}

pub fn draw_orbit_previews(
    planner: Res<RoutePlanner>,
    map_mode: Res<MapViewMode>,
    clock: Res<SimulationClock>,
    physics: Res<PhysicsConstants>,
    mut gizmos: Gizmos,
    bodies: Query<(
        &CelestialBody,
        &BodyMass,
        &Position,
        &Velocity,
        Option<&FixedBody>,
        Option<&Probe>,
    )>,
) {
    if !planner.show_previews && !map_mode.active {
        return;
    }

    let snapshot: Vec<_> = bodies
        .iter()
        .map(|(c, m, p, v, fixed, probe)| {
            (
                c.name.clone(),
                m.0,
                p.0,
                v.0,
                fixed.is_some(),
                probe.is_some(),
            )
        })
        .collect();

    let central_name = planner
        .central_body
        .clone()
        .or_else(|| snapshot.iter().find(|(_, _, _, _, fixed, _)| *fixed).map(|(n, _, _, _, _, _)| n.clone()));

    let Some(central_name) = central_name else {
        return;
    };

    let Some((_, mu_mass, central_pos, _, _, _)) = snapshot
        .iter()
        .find(|(n, _, _, _, fixed, _)| n == &central_name && *fixed)
    else {
        return;
    };
    let mu = physics.g * mu_mass;

    let target_name = planner.target_body.clone().or_else(|| {
        snapshot
            .iter()
            .find(|(_, _, _, _, fixed, probe)| !*fixed && *probe)
            .or_else(|| snapshot.iter().find(|(_, _, _, _, fixed, _)| !*fixed))
            .map(|(n, _, _, _, _, _)| n.clone())
    });

    let Some(target_name) = target_name else {
        return;
    };

    let Some((_, _, ship_pos, ship_vel, _, is_probe)) = snapshot
        .iter()
        .find(|(n, _, _, _, _, _)| n == &target_name)
    else {
        return;
    };

    let rel = RelativeState::new(*ship_pos - *central_pos, *ship_vel);
    let color_current = if *is_probe {
        Color::srgba(0.3, 0.95, 1.0, 0.85)
    } else {
        Color::srgba(0.5, 0.85, 1.0, 0.7)
    };

    let current_path = orbit::sample_orbit_path(mu, rel, 128);
    draw_path_inertial(&mut gizmos, &current_path, *central_pos, color_current);

    let node_params: Vec<(f32, f32, f32, f32)> = planner
        .nodes
        .iter()
        .map(|n| (n.time, n.prograde, n.normal, n.radial))
        .collect();

    if !node_params.is_empty() {
        let horizon = planner.preview_horizon.max(50.0);
        let step = planner.preview_step.max(0.05);
        let predicted = orbit::propagate_with_nodes(mu, rel, &node_params, clock.time, horizon, step);
        draw_path_inertial(
            &mut gizmos,
            &predicted,
            *central_pos,
            Color::srgba(1.0, 0.55, 0.15, 0.9),
        );

        for node in &planner.nodes {
            let time_offset = (node.time - clock.time).max(0.0);
            if let Some(pos) = position_at_time(&predicted, time_offset, step) {
                let world = *central_pos + pos;
                gizmos.sphere(world, 4.0, Color::srgba(1.0, 0.9, 0.2, 0.95));
                let dv = node.delta_v_magnitude();
                if dv > 0.01 {
                    let (_, n_hat, r_hat) = orbit::tnw_frame(RelativeState::new(
                        pos,
                        Vec3::ZERO,
                    ));
                    let dir = (n_hat * node.normal + r_hat * node.radial).normalize_or_zero();
                    let burn_dir = if dir.length_squared() > 1e-6 {
                        dir
                    } else {
                        orbit::tnw_frame(RelativeState::new(pos, Vec3::X)).0
                    };
                    gizmos.arrow(world, world + burn_dir * (8.0 + dv * 2.0), Color::srgb(1.0, 0.4, 0.1));
                }
            }
        }
    }

    if map_mode.active {
        for (_, _, pos, _, fixed, _) in &snapshot {
            if *fixed {
                gizmos.sphere(*pos, 22.0, Color::srgba(1.0, 0.85, 0.2, 0.25));
            }
        }
    }
}

fn draw_path_inertial(gizmos: &mut Gizmos, path: &[Vec3], origin: Vec3, color: Color) {
    for window in path.windows(2) {
        gizmos.line(origin + window[0], origin + window[1], color);
    }
}

fn position_at_time(path: &[Vec3], offset: f32, step: f32) -> Option<Vec3> {
    if path.is_empty() || step <= 0.0 {
        return None;
    }
    let idx = (offset / step).round() as usize;
    path.get(idx.min(path.len() - 1)).copied()
}
