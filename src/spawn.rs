use bevy::light::GlobalAmbientLight;
use bevy::prelude::*;

use crate::components::{
    CelestialBody, FixedBody, Mass, OrbitTrail, Position, Probe, SelectedBody, Velocity,
    VisualRadius,
};
use crate::resources::{ActiveScenario, EditorState, PhysicsConstants, WorldAssets};
use crate::scenario::{BodyDef, Scenario, load_scenario, scenario_asset_path};

const SELECTED_SCALE: f32 = 1.2;

pub fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    world_assets: Option<Res<WorldAssets>>,
    mut active: ResMut<ActiveScenario>,
    mut physics: ResMut<PhysicsConstants>,
    mut editor: ResMut<EditorState>,
) {
    let sphere_mesh = if let Some(assets) = world_assets {
        assets.sphere_mesh.clone()
    } else {
        let mesh = meshes.add(Sphere::new(1.0));
        commands.insert_resource(WorldAssets {
            sphere_mesh: mesh.clone(),
        });
        mesh
    };

    let scenario = active.template.clone();
    active.name = scenario.name.clone();
    physics.g = scenario.g;
    physics.softening = scenario.softening;

    spawn_lighting(&mut commands);
    spawn_bodies(&mut commands, &mut materials, &sphere_mesh, &scenario);
    sync_editor_from_scenario(&mut editor, &scenario);
}

pub fn reload_scenario(
    mut events: MessageReader<crate::resources::ReloadScenario>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    world_assets: Res<WorldAssets>,
    mut active: ResMut<ActiveScenario>,
    mut physics: ResMut<PhysicsConstants>,
    mut editor: ResMut<EditorState>,
    bodies: Query<Entity, With<CelestialBody>>,
) {
    if events.read().next().is_none() {
        return;
    }

    let path = scenario_asset_path(&active.file_path);
    let scenario = match load_scenario(&path) {
        Ok(scenario) => scenario,
        Err(err) => {
            warn!("Scenario reload failed: {err}");
            return;
        }
    };

    for entity in &bodies {
        commands.entity(entity).despawn();
    }

    active.template = scenario;
    active.name = active.template.name.clone();
    physics.g = active.template.g;
    physics.softening = active.template.softening;
    spawn_bodies(
        &mut commands,
        &mut materials,
        &world_assets.sphere_mesh,
        &active.template,
    );
    sync_editor_from_scenario(&mut editor, &active.template);
}

fn sync_editor_from_scenario(editor: &mut EditorState, scenario: &Scenario) {
    editor.velocity_dirty = false;
    editor.selection_changed = true;
    if let Some(first) = scenario.bodies.iter().find(|b| !b.fixed && !b.probe) {
        editor.selected_name = Some(first.name.clone());
        editor.velocity_x = first.velocity[0];
        editor.velocity_y = first.velocity[1];
        editor.velocity_z = first.velocity[2];
    } else {
        editor.selected_name = None;
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
    materials: &mut ResMut<Assets<StandardMaterial>>,
    sphere_mesh: &Handle<Mesh>,
    scenario: &Scenario,
) {
    for body in &scenario.bodies {
        if let Err(err) = validate_body(body) {
            warn!("Skipping body '{}': {err}", body.name);
            continue;
        }
        spawn_body(commands, materials, sphere_mesh, body);
    }
}

fn validate_body(body: &BodyDef) -> Result<(), &'static str> {
    if body.mass <= 0.0 {
        return Err("mass must be positive");
    }
    if body.radius <= 0.0 {
        return Err("radius must be positive");
    }
    Ok(())
}

fn spawn_body(
    commands: &mut Commands,
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
        VisualRadius(def.radius),
    ));

    if def.fixed {
        entity.insert(FixedBody);
    }
    if def.probe {
        entity.insert(Probe);
    }
}

pub fn draw_orbit_trails(mut gizmos: Gizmos, bodies: Query<(&OrbitTrail, Option<&Probe>)>) {
    for (trail, probe) in &bodies {
        if trail.points.len() < 2 {
            continue;
        }
        let color = if probe.is_some() {
            Color::srgba(0.7, 0.9, 1.0, 0.65)
        } else {
            Color::srgba(0.85, 0.85, 0.95, 0.45)
        };
        for window in trail.points.windows(2) {
            gizmos.line(window[0], window[1], color);
        }
    }
}

pub fn sync_editor_from_selection(
    mut editor: ResMut<EditorState>,
    bodies: Query<(&CelestialBody, &Velocity, Option<&FixedBody>)>,
) {
    if !editor.selection_changed {
        return;
    }
    editor.selection_changed = false;

    let Some(selected) = &editor.selected_name else {
        return;
    };

    if let Some((_, velocity, _)) = bodies.iter().find(|(b, _, _)| &b.name == selected) {
        editor.velocity_x = velocity.0.x;
        editor.velocity_y = velocity.0.y;
        editor.velocity_z = velocity.0.z;
        editor.velocity_dirty = false;
    }
}

pub fn apply_editor_velocity(
    mut editor: ResMut<EditorState>,
    mut bodies: Query<(
        Entity,
        &CelestialBody,
        &mut Velocity,
        Option<&SelectedBody>,
        Option<&Probe>,
        Option<&FixedBody>,
    )>,
    mut commands: Commands,
) {
    let Some(selected) = &editor.selected_name else {
        return;
    };

    let mut applied = false;
    for (entity, body, mut velocity, selected_marker, probe, fixed) in &mut bodies {
        if fixed.is_some() {
            continue;
        }
        let is_selected = &body.name == selected;
        if is_selected {
            if probe.is_none() && editor.velocity_dirty {
                velocity.0 = Vec3::new(editor.velocity_x, editor.velocity_y, editor.velocity_z);
                applied = true;
            }
            if selected_marker.is_none() {
                commands.entity(entity).insert(SelectedBody);
            }
        } else if selected_marker.is_some() {
            commands.entity(entity).remove::<SelectedBody>();
        }
    }
    if applied {
        editor.velocity_dirty = false;
    }
}

pub fn update_selection_visuals(
    editor: Res<EditorState>,
    mut bodies: Query<(
        &CelestialBody,
        &mut Transform,
        &VisualRadius,
        Option<&SelectedBody>,
    )>,
) {
    for (body, mut transform, visual_radius, _) in &mut bodies {
        let selected = editor.selected_name.as_ref() == Some(&body.name);
        let scale = if selected {
            visual_radius.0 * SELECTED_SCALE
        } else {
            visual_radius.0
        };
        transform.scale = Vec3::splat(scale);
    }
}

pub fn spawn_probe(
    mut events: MessageReader<crate::resources::SpawnProbe>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    world_assets: Res<WorldAssets>,
    editor: Res<EditorState>,
    bodies: Query<(&CelestialBody, &Position, &Velocity)>,
    existing_probes: Query<&CelestialBody, With<Probe>>,
) {
    if events.read().next().is_none() {
        return;
    }
    let Some(anchor_name) = &editor.selected_name else {
        return;
    };
    if existing_probes.iter().count() >= 8 {
        warn!("Probe limit reached (8)");
        return;
    }
    let Some((anchor, position, velocity)) =
        bodies.iter().find(|(body, _, _)| &body.name == anchor_name)
    else {
        return;
    };

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
        &mut materials,
        &world_assets.sphere_mesh,
        &probe_def,
    );
}
