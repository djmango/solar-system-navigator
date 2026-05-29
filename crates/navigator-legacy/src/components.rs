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

/// Inertial N-body path — pre-integrated at load, then updated live during simulation.
#[derive(Component)]
pub struct TruthOrbit {
    pub points: Vec<Vec3>,
    pub max_points: usize,
    /// When true, only live sim samples are kept (cleared on first physics step after play).
    pub live_only: bool,
}

impl TruthOrbit {
    pub fn new(max_points: usize) -> Self {
        Self {
            points: Vec::with_capacity(max_points),
            max_points,
            live_only: false,
        }
    }

    pub fn load_path(&mut self, path: Vec<Vec3>) {
        self.live_only = false;
        self.points = path;
        if self.points.len() > self.max_points {
            let skip = self.points.len() - self.max_points;
            self.points.drain(0..skip);
        }
    }

    /// Switch from load-time N-body preview to recording the live simulation.
    pub fn ensure_live_if_simulating(&mut self) {
        if !self.live_only {
            self.begin_live_recording();
        }
    }

    pub fn begin_live_recording(&mut self) {
        self.live_only = true;
        self.points.clear();
    }

    pub fn push_live(&mut self, position: Vec3) {
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

#[derive(Component)]
pub struct Starfield;

/// Patched-conics SOI radius for map/planner [m].
#[derive(Component, Clone, Copy)]
pub struct SoiRadius(pub f32);
