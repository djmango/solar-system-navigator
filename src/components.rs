use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct CelestialBody {
    pub name: String,
}

/// Sun-like anchor: attracts others but is not integrated.
#[derive(Component)]
pub struct FixedBody;

#[derive(Component)]
pub struct Probe;

#[derive(Component, Debug, Clone, Copy)]
pub struct Mass(pub f32);

#[derive(Component, Debug, Clone, Copy)]
pub struct Position(pub Vec3);

#[derive(Component, Debug, Clone, Copy)]
pub struct Velocity(pub Vec3);

#[derive(Component)]
pub struct OrbitTrail {
    pub points: Vec<Vec3>,
    pub max_points: usize,
}

impl OrbitTrail {
    pub fn new(max_points: usize) -> Self {
        Self {
            points: Vec::with_capacity(max_points),
            max_points,
        }
    }

    pub fn push(&mut self, position: Vec3) {
        if self.points.len() >= self.max_points {
            self.points.remove(0);
        }
        self.points.push(position);
    }
}

#[derive(Component)]
pub struct SelectedBody;

/// Display radius used to restore scale when selection changes.
#[derive(Component, Clone, Copy)]
pub struct VisualRadius(pub f32);
