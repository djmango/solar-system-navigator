use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

mod ui;
mod physics;
mod components;
mod resources;

use components::{CentralBody, OrbitingBody, Position, Mass};
use resources::SimulationControl;
use physics::orbital_physics;
use ui::{spawn_ui, ui_system};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<SimulationControl>()
        .add_systems(Startup, setup)
        .add_systems(Update, (orbital_physics, ui_system))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Camera (only one needed)
    commands.spawn(Camera2dBundle::default());

    // Spawn central body (sun)
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(25.0).into()).into(),
            material: materials.add(ColorMaterial::from(Color::YELLOW)),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        CentralBody,
        Mass(1000.0),
        Position(Vec2::ZERO),
    ));

    // Spawn orbiting body (planet)
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(10.0).into()).into(),
            material: materials.add(ColorMaterial::from(Color::BLUE)),
            transform: Transform::from_xyz(200.0, 0.0, 0.0),
            ..default()
        },
        OrbitingBody {
            velocity: Vec2::new(0.0, 2.0),
        },
        Mass(1.0),
        Position(Vec2::new(200.0, 0.0)),
    ));

    // Spawn UI
    spawn_ui(&mut commands);
}
