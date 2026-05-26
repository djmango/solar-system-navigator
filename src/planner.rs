//! Route planner: sync central body, advance sim clock, UI helpers.

use bevy::prelude::*;

use crate::components::{CelestialBody, FixedBody, Probe};
use crate::maneuver::reset_maneuver_execution;
use crate::resources::{EditorState, ManeuverNode, RoutePlanner, SimulationClock, SimulationControl};

pub fn advance_simulation_clock(
    time: Res<Time>,
    control: Res<SimulationControl>,
    mut clock: ResMut<SimulationClock>,
) {
    if control.paused && !control.step_once {
        return;
    }
    let dt = time.delta_secs() * control.speed * control.ticks_per_frame.max(1) as f32;
    clock.time += dt;
}

pub fn sync_route_planner_targets(
    editor: Res<EditorState>,
    mut planner: ResMut<RoutePlanner>,
    bodies: Query<(&CelestialBody, Option<&FixedBody>, Option<&Probe>)>,
) {
    if let Some(name) = &editor.selected_name {
        planner.target_body = Some(name.clone());
    }

    let fixed: Vec<_> = bodies
        .iter()
        .filter(|(_, fixed, _)| fixed.is_some())
        .map(|(c, _, _)| c.name.clone())
        .collect();

    if planner.central_body.is_none() {
        planner.central_body = fixed.first().cloned();
    }
}

pub fn add_maneuver_node(planner: &mut RoutePlanner, clock: &SimulationClock) {
    let t = clock.time + planner.default_burn_offset;
    planner.nodes.push(ManeuverNode {
        time: t,
        prograde: planner.draft_prograde,
        normal: planner.draft_normal,
        radial: planner.draft_radial,
        executed: false,
    });
    planner.show_previews = true;
}

pub fn clear_maneuver_nodes(planner: &mut RoutePlanner) {
    planner.nodes.clear();
    planner.show_previews = false;
}

pub fn on_simulation_reset(clock: &mut SimulationClock, planner: &mut RoutePlanner) {
    clock.time = 0.0;
    reset_maneuver_execution(planner);
}
