use bevy::prelude::*;

#[derive(Component)]
pub struct CentralBody;

#[derive(Component)]
pub struct OrbitingBody {
    pub velocity: Vec2,
}

#[derive(Component)]
pub struct Position(pub Vec2);

#[derive(Component)]
pub struct Mass(pub f32); 
