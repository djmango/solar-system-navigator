//! KSP-style map mode and N-body truth orbit visualization.

use bevy::prelude::*;

use crate::astro::AU;
use crate::camera::OrbitCamera;
use crate::components::{
    CelestialBody, FixedBody, Mass as BodyMass, Position, Probe, SoiRadius, TruthOrbit, Velocity,
};
use crate::orbit::{self, RelativeState};
use crate::planner::build_soi_snapshots;
use crate::resources::{
    ActiveScenario, GameUx, ManeuverNode, MapViewMode, PhysicsConstants, PlannerPreview,
    RoutePlanner, SimulationClock,
};
use crate::scenario::BodyDef;
use crate::soi::SoiBodySnapshot;

const MAP_RADIUS: f32 = 3.2 * AU;
const MAP_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.02;

pub fn toggle_map_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut map_mode: ResMut<MapViewMode>,
    mut planner: ResMut<RoutePlanner>,
    mut game_ux: ResMut<GameUx>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        map_mode.active = !map_mode.active;
        planner.show_previews = map_mode.active || !planner.nodes.is_empty();
        if map_mode.active {
            game_ux.show_system_orbits = true;
        }
    }
    if keyboard.just_pressed(KeyCode::Escape) && map_mode.active {
        map_mode.active = false;
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
    game_ux: Res<GameUx>,
    clock: Res<SimulationClock>,
    physics: Res<PhysicsConstants>,
    active: Res<ActiveScenario>,
    preview: Res<PlannerPreview>,
    mut gizmos: Gizmos,
    bodies: Query<(
        &CelestialBody,
        &BodyMass,
        &Position,
        &Velocity,
        &SoiRadius,
        &TruthOrbit,
        Option<&FixedBody>,
        Option<&Probe>,
    )>,
) {
    let show_system = map_mode.active || game_ux.show_system_orbits;
    let show_target = planner.show_previews || map_mode.active;

    if !show_system && !show_target {
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

    if show_system {
        draw_nbody_truth_orbits(
            &mut gizmos,
            &bodies,
            &active.template.bodies,
            planner.target_body.as_deref(),
        );
    }

    if !show_target {
        if map_mode.active {
            draw_soi_gizmos(&mut gizmos, &snapshots);
            if let Some(xfer) = planner.last_hohmann {
                let mu = physics.g * central.mass;
                draw_hohmann_target_orbit(&mut gizmos, mu, central.position, xfer.r2);
            }
        }
        return;
    }

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
        .any(|(c, _, _, _, _, _, _, probe)| c.name == target_name && probe.is_some());

    if let Some(truth) = bodies
        .iter()
        .find(|(c, _, _, _, _, _, _, _)| c.name == target_name)
        .map(|(_, _, _, _, _, t, _, _)| t)
    {
        let color = if is_probe {
            Color::srgba(0.3, 0.95, 1.0, 0.95)
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.95)
        };
        draw_world_path(&mut gizmos, &truth.points, color);
    }

    // Planned trajectory (orange) reflects all pending nodes; the coast path is
    // already drawn above, so this only appears once a node exists.
    let preview_matches = preview.vessel.as_deref() == Some(target_name.as_str());
    if !planner.nodes.is_empty() && preview_matches {
        draw_world_path(
            &mut gizmos,
            &preview.path,
            Color::srgba(1.0, 0.55, 0.15, 0.95),
        );
    }

    if preview_matches {
        let marker_base = central.soi_radius.clamp(4.0e8, 3.0e9);
        for node in &planner.nodes {
            if node.executed || node.time < clock.time {
                continue;
            }
            let Some(pos) = preview.position_at_time(node.time) else {
                continue;
            };
            let selected = planner.selected_node == Some(node.id);
            let marker = if selected {
                marker_base * 1.6
            } else {
                marker_base
            };
            let color = if selected {
                Color::srgba(1.0, 1.0, 0.4, 1.0)
            } else {
                Color::srgba(1.0, 0.8, 0.2, 0.85)
            };
            gizmos.sphere(pos, marker, color);
            if selected {
                draw_node_handles(&mut gizmos, &preview, node, central.position, pos);
            }
        }
    }

    if map_mode.active {
        draw_soi_gizmos(&mut gizmos, &snapshots);
        if let Some(xfer) = planner.last_hohmann {
            let mu = physics.g * central.mass;
            draw_hohmann_target_orbit(&mut gizmos, mu, central.position, xfer.r2);
        }
    }
}

/// Draw prograde / normal / radial direction handles at the selected node so the
/// burn frame is legible in the map view.
fn draw_node_handles(
    gizmos: &mut Gizmos,
    preview: &PlannerPreview,
    node: &ManeuverNode,
    central_pos: Vec3,
    node_pos: Vec3,
) {
    let radial = (node_pos - central_pos).normalize_or_zero();
    if radial == Vec3::ZERO {
        return;
    }

    // Prograde from the local tangent of the predicted path.
    let prograde = if preview.step > 0.0 && preview.path.len() >= 2 {
        let idx = (((node.time - preview.start_time) / preview.step).round() as usize)
            .min(preview.path.len() - 1);
        let next = (idx + 1).min(preview.path.len() - 1);
        let prev = idx.saturating_sub(1);
        (preview.path[next] - preview.path[prev]).normalize_or_zero()
    } else {
        Vec3::ZERO
    };
    let prograde = if prograde == Vec3::ZERO {
        radial.cross(Vec3::Y).normalize_or_zero()
    } else {
        prograde
    };
    let normal = prograde.cross(radial).normalize_or_zero();

    let len = ((node_pos - central_pos).length() * 0.3).clamp(2.0e9, 4.0e10);
    gizmos.line(
        node_pos,
        node_pos + prograde * len,
        Color::srgb(0.3, 1.0, 0.4),
    );
    gizmos.line(
        node_pos,
        node_pos + normal * len,
        Color::srgb(0.8, 0.4, 1.0),
    );
    gizmos.line(
        node_pos,
        node_pos + radial * len,
        Color::srgb(0.3, 0.8, 1.0),
    );

    // Resultant Δv direction in that frame.
    let dv = prograde * node.prograde + normal * node.normal + radial * node.radial;
    if dv.length_squared() > 1.0 {
        gizmos.line(
            node_pos,
            node_pos + dv.normalize() * len * 1.2,
            Color::srgb(1.0, 0.9, 0.5),
        );
    }
}

fn draw_nbody_truth_orbits(
    gizmos: &mut Gizmos,
    bodies: &Query<(
        &CelestialBody,
        &BodyMass,
        &Position,
        &Velocity,
        &SoiRadius,
        &TruthOrbit,
        Option<&FixedBody>,
        Option<&Probe>,
    )>,
    body_defs: &[BodyDef],
    highlight: Option<&str>,
) {
    for (body, _, _, _, _, truth, fixed, _) in bodies {
        if fixed.is_some() || truth.points.len() < 2 {
            continue;
        }
        let base = body_color(body_defs, &body.name);
        let selected = highlight == Some(body.name.as_str());
        let color = if selected {
            Color::srgba(1.0, 1.0, 1.0, 0.95)
        } else {
            base.with_alpha(0.5)
        };
        draw_world_path(gizmos, &truth.points, color);
    }
}

fn body_color(body_defs: &[BodyDef], name: &str) -> Color {
    body_defs
        .iter()
        .find(|b| b.name == name)
        .map(|b| b.color())
        .unwrap_or(Color::srgba(0.7, 0.75, 0.85, 1.0))
}

fn draw_soi_gizmos(gizmos: &mut Gizmos, bodies: &[SoiBodySnapshot]) {
    for body in bodies {
        if body.is_primary || !body.soi_radius.is_finite() || body.soi_radius <= 0.0 {
            continue;
        }
        let color = Color::srgba(0.4, 0.75, 1.0, 0.22);
        gizmos.sphere(body.position, body.soi_radius, color);
    }
}

fn draw_hohmann_target_orbit(gizmos: &mut Gizmos, mu: f32, origin: Vec3, radius: f32) {
    let state = RelativeState::new(
        Vec3::new(radius, 0.0, 0.0),
        Vec3::new(0.0, 0.0, (mu / radius).sqrt()),
    );
    let path = orbit::sample_orbit_path(mu, state, 96);
    draw_path_relative(gizmos, &path, origin, Color::srgba(0.85, 0.35, 1.0, 0.55));
}

fn draw_world_path(gizmos: &mut Gizmos, path: &[Vec3], color: Color) {
    for window in path.windows(2) {
        gizmos.line(window[0], window[1], color);
    }
}

fn draw_path_relative(gizmos: &mut Gizmos, path: &[Vec3], origin: Vec3, color: Color) {
    for window in path.windows(2) {
        gizmos.line(origin + window[0], origin + window[1], color);
    }
}
