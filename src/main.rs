use bevy::prelude::*;

// Components
#[derive(Component)]
struct CentralBody;

#[derive(Component)]
struct OrbitingBody {
    velocity: Vec2,
}

#[derive(Component)]
struct Position(Vec2);

#[derive(Component)]
struct Mass(f32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, orbital_physics)
        .run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Spawn central body (sun)
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::YELLOW,
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        CentralBody,
        Mass(1000.0),
        Position(Vec2::ZERO),
    ));

    // Spawn orbiting body (planet)
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLUE,
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            transform: Transform::from_xyz(200.0, 0.0, 0.0),
            ..default()
        },
        OrbitingBody {
            velocity: Vec2::new(0.0, 2.0), // Initial velocity for circular-ish orbit
        },
        Mass(1.0),
        Position(Vec2::new(200.0, 0.0)),
    ));
}

fn orbital_physics(
    time: Res<Time>,
    mut query_set: ParamSet<(
        Query<(&Position, &Mass), With<CentralBody>>,
        Query<(&mut Position, &mut OrbitingBody, &Mass, &mut Transform)>,
    )>,
) {
    const G: f32 = 1.0; // Gravitational constant (simplified for simulation)

    // Get central body data first
    let central_query = query_set.p0();
    let central_data = central_query.single();
    let central_pos = central_data.0.0;
    let central_mass = central_data.1.0;

    // Then update orbiting bodies
    for (mut pos, mut orbiting, mass, mut transform) in query_set.p1().iter_mut() {
        let r = central_pos - pos.0;
        let distance = r.length();
        
        // Calculate gravitational force
        let force = G * central_mass * mass.0 / (distance * distance);
        let force_dir = r.normalize();
        let acceleration = force_dir * force / mass.0;
        
        // Update velocity and position using basic euler integration
        orbiting.velocity += acceleration * time.delta_seconds();
        pos.0 += orbiting.velocity * time.delta_seconds();
        
        // Update transform directly
        transform.translation = pos.0.extend(0.0);
    }
}
