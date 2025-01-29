use bevy::prelude::*;

use crate::components::{CentralBody, OrbitingBody, Position, Mass};
use crate::resources::SimulationControl;

pub fn orbital_physics(
    time: Res<Time>,
    simulation_control: Res<SimulationControl>,
    mut query_set: ParamSet<(
        Query<(&Position, &Mass), With<CentralBody>>,
        Query<(&mut Position, &mut OrbitingBody, &Mass, &mut Transform)>,
    )>,
) {
    const G: f32 = 1.0;

    let dt = time.delta_seconds() * simulation_control.speed;
    
    for _ in 0..simulation_control.ticks_per_frame {
        let central_query = query_set.p0();
        let central_data = central_query.single();
        let central_pos = central_data.0.0;
        let central_mass = central_data.1.0;

        for (mut pos, mut orbiting, mass, mut transform) in query_set.p1().iter_mut() {
            let r = central_pos - pos.0;
            let distance = r.length();
            
            let force = G * central_mass * mass.0 / (distance * distance);
            let force_dir = r.normalize();
            let acceleration = force_dir * force / mass.0;
            
            orbiting.velocity += acceleration * dt;
            pos.0 += orbiting.velocity * dt;
            
            transform.translation = pos.0.extend(0.0);
        }
    }
} 
