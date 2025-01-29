use bevy::prelude::*;

#[derive(Resource)]
pub struct SimulationControl {
    pub speed: f32,
    pub ticks_per_frame: u32,
}

impl Default for SimulationControl {
    fn default() -> Self {
        Self {
            speed: 1.0,
            ticks_per_frame: 1,
        }
    }
} 
