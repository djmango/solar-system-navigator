use bevy::light::GlobalAmbientLight;
use bevy::prelude::*;

use crate::components::{
    CelestialBody, FixedBody, Mass, OrbitTrail, Position, Probe, SelectedBody, SoiRadius,
    Starfield, Velocity, VisualRadius,
};
use crate::planner::{compute_soi_radius_for_body, on_simulation_reset};
use crate::resources::{
    ActiveScenario, BodyTextureCache, EditorState, PhysicsConstants, RoutePlanner, SimulationClock,
    WorldAssets,
};
use crate::scenario::{BodyDef, Scenario, load_scenario, scenario_asset_path};

const SELECTED_SCALE: f32 = 1.2;

pub fn spawn_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut texture_cache: ResMut<BodyTextureCache>,
    stars: Query<Entity, With<Starfield>>,
    world_assets: Option<Res<WorldAssets>>,
    mut active: ResMut<ActiveScenario>,
    mut physics: ResMut<PhysicsConstants>,
    mut editor: ResMut<EditorState>,
    mut clock: ResMut<SimulationClock>,
    mut planner: ResMut<RoutePlanner>,
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

    if stars.is_empty() {
        spawn_lighting(&mut commands);
        spawn_starfield(&mut commands, &mut meshes, &mut materials);
    }

    let scenario = active.template.clone();
    active.name = scenario.name.clone();
    physics.g = scenario.g;
    physics.softening = scenario.softening;
    spawn_bodies(
        &mut commands,
        &asset_server,
        &mut texture_cache,
        &mut materials,
        &sphere_mesh,
        &scenario,
    );
    sync_editor_from_scenario(&mut editor, &scenario);
    planner.central_body = scenario
        .bodies
        .iter()
        .find(|b| b.fixed)
        .map(|b| b.name.clone());
    on_simulation_reset(&mut clock, &mut planner);
}

pub fn reload_scenario(
    mut events: MessageReader<crate::resources::ReloadScenario>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut texture_cache: ResMut<BodyTextureCache>,
    world_assets: Res<WorldAssets>,
    mut active: ResMut<ActiveScenario>,
    mut physics: ResMut<PhysicsConstants>,
    mut editor: ResMut<EditorState>,
    mut clock: ResMut<SimulationClock>,
    mut planner: ResMut<RoutePlanner>,
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
        &asset_server,
        &mut texture_cache,
        &mut materials,
        &world_assets.sphere_mesh,
        &active.template,
    );
    sync_editor_from_scenario(&mut editor, &active.template);
    planner.central_body = active
        .template
        .bodies
        .iter()
        .find(|b| b.fixed)
        .map(|b| b.name.clone());
    on_simulation_reset(&mut clock, &mut planner);
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

fn spawn_starfield(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let star_mesh = meshes.add(Sphere::new(1.0));
    let star_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.97, 1.0),
        emissive: LinearRgba::new(1.8, 1.9, 2.2, 1.0),
        unlit: true,
        ..default()
    });

    let mut rng_state: u32 = 0xC0FFEE_u32;
    for _ in 0..600 {
        rng_state = rng_state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        let u = (rng_state as f32) / u32::MAX as f32;
        rng_state = rng_state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        let v = (rng_state as f32) / u32::MAX as f32;
        rng_state = rng_state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        let w = (rng_state as f32) / u32::MAX as f32;

        let theta = u * std::f32::consts::TAU;
        let phi = (v * 2.0 - 1.0).acos();
        let radius = 6_000.0 + w * 4_000.0;
        let position = Vec3::new(
            radius * phi.sin() * theta.cos(),
            radius * phi.cos(),
            radius * phi.sin() * theta.sin(),
        );
        let scale = 1.5 + w * 4.0;

        commands.spawn((
            Mesh3d(star_mesh.clone()),
            MeshMaterial3d(star_material.clone()),
            Transform::from_translation(position).with_scale(Vec3::splat(scale)),
            Starfield,
        ));
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
    asset_server: &AssetServer,
    texture_cache: &mut BodyTextureCache,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    sphere_mesh: &Handle<Mesh>,
    scenario: &Scenario,
) {
    let primary = scenario.bodies.iter().find(|b| b.fixed);
    let (primary_mass, primary_position) = primary
        .map(|p| (p.mass, p.position_vec3()))
        .unwrap_or((1.0, Vec3::ZERO));

    for body in &scenario.bodies {
        if let Err(err) = validate_body(body) {
            warn!("Skipping body '{}': {err}", body.name);
            continue;
        }
        let soi = compute_soi_radius_for_body(body, scenario, primary_mass, primary_position);
        spawn_body(
            commands,
            asset_server,
            texture_cache,
            materials,
            sphere_mesh,
            body,
            soi,
        );
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

fn texture_asset_exists(relative: &str) -> bool {
    std::path::Path::new("assets").join(relative).is_file()
}

fn texture_handle(
    asset_server: &AssetServer,
    cache: &mut BodyTextureCache,
    path: &str,
) -> Handle<Image> {
    cache
        .handles
        .entry(path.to_string())
        .or_insert_with(|| asset_server.load(path.to_string()))
        .clone()
}

fn spawn_body(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_cache: &mut BodyTextureCache,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    sphere_mesh: &Handle<Mesh>,
    def: &BodyDef,
    soi_radius: f32,
) {
    let position = def.position_vec3();
    let base = def.color();
    let atmosphere = def
        .atmosphere_color()
        .unwrap_or_else(|| base.mix(&Color::srgb(0.6, 0.75, 1.0), 0.35));
    let emissive_strength = if def.fixed {
        def.emissive * 4.0
    } else {
        def.emissive * 0.35
    };
    let emissive_color: LinearRgba = if def.fixed {
        LinearRgba::from(base) * emissive_strength
    } else {
        LinearRgba::from(atmosphere) * emissive_strength + LinearRgba::from(base) * 0.15
    };

    let base_texture = def.texture.as_ref().filter(|path| texture_asset_exists(path)).map(
        |path| {
            if !texture_cache.handles.contains_key(path) {
                info!("Loading texture: {path}");
            }
            texture_handle(asset_server, texture_cache, path)
        },
    );

    let mut material = StandardMaterial {
        base_color: Color::WHITE,
        emissive: emissive_color,
        emissive_exposure_weight: if def.fixed { 2.5 } else { 1.2 },
        metallic: if def.probe { 0.35 } else { 0.08 },
        perceptual_roughness: if def.probe { 0.35 } else { 0.62 },
        reflectance: if def.fixed { 0.6 } else { 0.45 },
        ..default()
    };
    if let Some(tex) = base_texture {
        material.base_color_texture = Some(tex);
        if def.fixed {
            material.unlit = true;
        }
    } else {
        material.base_color = base;
        if def.texture.is_some() {
            warn!(
                "Texture missing for '{}' (run ./scripts/fetch_textures.sh) — using color fallback",
                def.name
            );
        }
    }

    let material = materials.add(material);

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
        SoiRadius(soi_radius),
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
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut texture_cache: ResMut<BodyTextureCache>,
    world_assets: Res<WorldAssets>,
    active: Res<ActiveScenario>,
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
        emissive: 1.2,
        atmosphere: None,
        texture: None,
        soi_radius: None,
        fixed: false,
        probe: true,
    };
    let primary = active.template.bodies.iter().find(|b| b.fixed);
    let (primary_mass, primary_position) = primary
        .map(|p| (p.mass, p.position_vec3()))
        .unwrap_or((1.0, Vec3::ZERO));
    let soi =
        compute_soi_radius_for_body(&probe_def, &active.template, primary_mass, primary_position);
    spawn_body(
        &mut commands,
        &asset_server,
        &mut texture_cache,
        &mut materials,
        &world_assets.sphere_mesh,
        &probe_def,
        soi,
    );
}
