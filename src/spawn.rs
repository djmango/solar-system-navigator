use bevy::light::GlobalAmbientLight;
use bevy::prelude::*;

use crate::components::{
    CelestialBody, FixedBody, Mass, OrbitTrail, Position, Probe, SelectedBody, Velocity,
};
use crate::resources::{ActiveScenario, EditorState, PhysicsConstants};
use crate::scenario::{load_scenario, scenario_asset_path, BodyDef, Scenario};

pub fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut active: ResMut<ActiveScenario>,
    mut physics: ResMut<PhysicsConstants>,
    mut editor: ResMut<EditorState>,
) {
    let scenario = active.template.clone();
    active.name = scenario.name.clone();
    physics.g = scenario.g;
    physics.softening = scenario.softening;

    spawn_lighting(&mut commands);
    spawn_bodies(&mut commands, &mut meshes, &mut materials, &scenario);
    sync_editor_from_scenario(&mut editor, &scenario);
}

pub fn reload_scenario(
    mut events: MessageReader<crate::resources::ReloadScenario>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut active: ResMut<ActiveScenario>,
    mut physics: ResMut<PhysicsConstants>,
    mut editor: ResMut<EditorState>,
    bodies: Query<Entity, With<CelestialBody>>,
) {
    if events.read().next().is_none() {
        return;
    }
    for entity in &bodies {
        commands.entity(entity).despawn();
    }

    let path = scenario_asset_path(&active.file_path);
    match load_scenario(&path) {
        Ok(scenario) => {
            active.template = scenario;
            active.name = active.template.name.clone();
            physics.g = active.template.g;
            physics.softening = active.template.softening;
            spawn_bodies(&mut commands, &mut meshes, &mut materials, &active.template);
            sync_editor_from_scenario(&mut editor, &active.template);
        }
        Err(err) => warn!("{err}"),
    }
}

fn sync_editor_from_scenario(editor: &mut EditorState, scenario: &Scenario) {
    if let Some(first) = scenario.bodies.iter().find(|b| !b.fixed && !b.probe) {
        editor.selected_name = Some(first.name.clone());
        editor.velocity_x = first.velocity[0];
        editor.velocity_y = first.velocity[1];
        editor.velocity_z = first.velocity[2];
    }
}

fn spawn_lighting(commands: &mut Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 14_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(500.0, 900.0, 400.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        ..default()
    });
}

fn spawn_bodies(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    scenario: &Scenario,
) {
    let sphere = meshes.add(Sphere::new(1.0));
    for body in &scenario.bodies {
        spawn_body(commands, meshes, materials, &sphere, body);
    }
}

fn spawn_body(
    commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    sphere_mesh: &Handle<Mesh>,
    def: &BodyDef,
) {
    let position = def.position_vec3();
    let material = materials.add(StandardMaterial {
        base_color: def.color(),
        emissive: def.color().into(),
        emissive_exposure_weight: if def.fixed { 2.0 } else { 0.8 },
        metallic: 0.05,
        perceptual_roughness: 0.75,
        ..default()
    });

    let mut entity = commands.spawn((
        Mesh3d(sphere_mesh.clone()),
        MeshMaterial3d(material),
        Transform::from_translation(position).with_scale(Vec3::splat(def.radius)),
        CelestialBody {
            name: def.name.clone(),
        },
        Mass(def.mass),
        Position(position),
        Velocity(def.velocity_vec3()),
        OrbitTrail::new(300),
    ));

    if def.fixed {
        entity.insert(FixedBody);
    }
    if def.probe {
        entity.insert(Probe);
    }
}

pub fn draw_orbit_trails(
    mut gizmos: Gizmos,
    bodies: Query<(&OrbitTrail, &CelestialBody)>,
) {
    for (trail, body) in &bodies {
        if trail.points.len() < 2 {
            continue;
        }
        let color = if body.name.contains("Probe") || body.name.contains("OSIRIS") {
            Color::srgba(0.7, 0.9, 1.0, 0.65)
        } else {
            Color::srgba(0.85, 0.85, 0.95, 0.45)
        };
        for window in trail.points.windows(2) {
            gizmos.line(window[0], window[1], color);
        }
    }
}

pub fn apply_editor_selection(
    editor: Res<EditorState>,
    mut bodies: Query<
        (
            Entity,
            &CelestialBody,
            &mut Velocity,
            Option<&SelectedBody>,
            Option<&Probe>,
            Option<&FixedBody>,
        ),
    >,
    mut commands: Commands,
) {
    let Some(selected) = &editor.selected_name else {
        return;
    };

    for (entity, body, mut velocity, selected_marker, probe, fixed) in &mut bodies {
        if fixed.is_some() {
            continue;
        }
        let is_selected = &body.name == selected;
        if is_selected {
            if probe.is_none() {
                velocity.0 = Vec3::new(
                    editor.velocity_x,
                    editor.velocity_y,
                    editor.velocity_z,
                );
            }
            if selected_marker.is_none() {
                commands.entity(entity).insert(SelectedBody);
            }
        } else if selected_marker.is_some() {
            commands.entity(entity).remove::<SelectedBody>();
        }
    }
}

pub fn spawn_probe(
    mut events: MessageReader<crate::resources::SpawnProbe>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    editor: Res<EditorState>,
    bodies: Query<(&CelestialBody, &Position, &Velocity)>,
) {
    if events.read().next().is_none() {
        return;
    }
    let Some(anchor_name) = &editor.selected_name else {
        return;
    };
    let Some((anchor, position, velocity)) = bodies
        .iter()
        .find(|(body, _, _)| &body.name == anchor_name)
    else {
        return;
    };

    let sphere = meshes.add(Sphere::new(1.0));
    let probe_def = BodyDef {
        name: format!("{} Probe", anchor.name),
        mass: 0.01,
        position: position.0.to_array(),
        velocity: (velocity.0 + Vec3::new(editor.probe_delta_v, 0.0, 0.0)).to_array(),
        radius: 2.0,
        color: [0.9, 0.95, 1.0],
        fixed: false,
        probe: true,
    };
    spawn_body(
        &mut commands,
        &mut meshes,
        &mut materials,
        &sphere,
        &probe_def,
    );
}
