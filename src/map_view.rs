//! KSP-style map mode: orthographic system view and predicted orbit gizmos.

use bevy::prelude::*;

use crate::astro::AU;
use crate::camera::OrbitCamera;
use crate::components::{
    CelestialBody, FixedBody, Mass as BodyMass, Position, Probe, SoiRadius, Velocity,
};
use crate::orbit;
use crate::planner::{build_soi_snapshots, relative_target_state};
use crate::resources::{MapViewMode, PhysicsConstants, RoutePlanner, SimulationClock};

const MAP_RADIUS: f32 = 3.0 * AU;
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
    planner: Res<RoutePlanner>,
    bodies: Query<(&CelestialBody, &Position)>,
    mut cameras: Query<(&mut Transform, &mut OrbitCamera), With<Camera3d>>,
) {
    if !map_mode.active {
        return;
    }

    let Ok((mut transform, mut orbit)) = cameras.single_mut() else {
        return;
    };

    let focus = planner
        .central_body
        .as_ref()
        .and_then(|name| {
            bodies
                .iter()
                .find(|(c, _)| &c.name == name)
                .map(|(_, p)| p.0)
        })
        .or_else(|| bodies.iter().next().map(|(_, p)| p.0))
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
        &SoiRadius,
        Option<&FixedBody>,
        Option<&Probe>,
    )>,
) {
    if !planner.show_previews && !map_mode.active {
        return;
    }

    let snapshots = build_soi_snapshots(&bodies);

    let central_name = planner.central_body.clone().or_else(|| {
        snapshots
            .iter()
            .find(|b| b.is_primary)
            .map(|b| b.name.clone())
    });

    let Some(central_name) = central_name else {
        return;
    };

    let central = match snapshots.iter().find(|b| b.name == central_name) {
        Some(c) => c,
        None => return,
    };
    let mu = physics.g * central.mass;

    let target_name = planner.target_body.clone().or_else(|| {
        snapshots
            .iter()
            .find(|b| !b.is_primary)
            .map(|b| b.name.clone())
    });

    let Some(target_name) = target_name else {
        return;
    };

    let is_probe = bodies
        .iter()
        .any(|(c, _, _, _, _, _, probe)| c.name == target_name && probe.is_some());

    let Some(rel) = relative_target_state(&snapshots, &central_name, &target_name) else {
        return;
    };

    let color_current = if is_probe {
        Color::srgba(0.3, 0.95, 1.0, 0.85)
    } else {
        Color::srgba(0.5, 0.85, 1.0, 0.7)
    };

    let current_path = orbit::sample_orbit_path(mu, rel, 128);
    draw_path_inertial(&mut gizmos, &current_path, central.position, color_current);

    let node_params: Vec<(f32, f32, f32, f32)> = planner
        .nodes
        .iter()
        .map(|n| (n.time, n.prograde, n.normal, n.radial))
        .collect();

    if !node_params.is_empty() {
        let horizon = planner.preview_horizon.max(50.0);
        let step = planner.preview_step.max(0.05);
        let predicted =
            orbit::propagate_with_nodes(mu, rel, &node_params, clock.time, horizon, step);
        draw_path_inertial(
            &mut gizmos,
            &predicted,
            central.position,
            Color::srgba(1.0, 0.55, 0.15, 0.9),
        );

        for node in &planner.nodes {
            let time_offset = (node.time - clock.time).max(0.0);
            if let Some(pos) = position_at_time(&predicted, time_offset, step) {
                let world = central.position + pos;
                let marker = 5.0e8 + node.delta_v_magnitude() * 2.0e7;
                gizmos.sphere(world, marker, Color::srgba(1.0, 0.9, 0.2, 0.95));
            }
        }
    }

    if map_mode.active {
        draw_soi_gizmos(&mut gizmos, &snapshots);
        if let Some(xfer) = planner.last_hohmann {
            draw_hohmann_target_orbit(&mut gizmos, mu, central.position, xfer.r2);
        }
    }
}

fn draw_soi_gizmos(gizmos: &mut Gizmos, bodies: &[crate::soi::SoiBodySnapshot]) {
    for body in bodies {
        if body.is_primary || !body.soi_radius.is_finite() || body.soi_radius <= 0.0 {
            continue;
        }
        let color = Color::srgba(0.4, 0.75, 1.0, 0.22);
        gizmos.sphere(body.position, body.soi_radius, color);
    }
}

fn draw_hohmann_target_orbit(gizmos: &mut Gizmos, mu: f32, origin: Vec3, radius: f32) {
    let state = orbit::RelativeState::new(
        Vec3::new(radius, 0.0, 0.0),
        Vec3::new(0.0, 0.0, (mu / radius).sqrt()),
    );
    let path = orbit::sample_orbit_path(mu, state, 96);
    draw_path_inertial(gizmos, &path, origin, Color::srgba(0.85, 0.35, 1.0, 0.55));
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
