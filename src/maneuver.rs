//! Maneuver nodes and burn execution (KSP-style Δv in TNW).

use bevy::prelude::*;

use crate::components::{CelestialBody, FixedBody, Position, Velocity};
use crate::orbit::{self, RelativeState};
use crate::resources::{RoutePlanner, SimulationClock};

pub fn execute_maneuver_burns(
    clock: Res<SimulationClock>,
    mut planner: ResMut<RoutePlanner>,
    mut bodies: Query<(
        &CelestialBody,
        &Position,
        &mut Velocity,
        Option<&FixedBody>,
    )>,
) {
    if planner.nodes.is_empty() {
        return;
    }

    let Some(central_name) = planner.central_body.clone() else {
        return;
    };

    let snapshot: Vec<_> = bodies
        .iter()
        .map(|(c, p, v, fixed)| (c.name.clone(), p.0, v.0, fixed.is_some()))
        .collect();

    let Some(central_pos) = snapshot
        .iter()
        .find(|(name, _, _, fixed)| name == &central_name && *fixed)
        .map(|(_, pos, _, _)| *pos)
    else {
        return;
    };

    let target_name = planner.target_body.clone().or_else(|| {
        snapshot
            .iter()
            .find(|(_, _, _, fixed)| !*fixed)
            .map(|(n, _, _, _)| n.clone())
    });

    let Some(target_name) = target_name else {
        return;
    };

    let mut new_velocity = None;

    for node in &mut planner.nodes {
        if node.executed || clock.time < node.time {
            continue;
        }

        let Some((_, ship_pos, ship_vel, _)) =
            snapshot.iter().find(|(n, _, _, _)| n == &target_name)
        else {
            continue;
        };

        let rel = RelativeState::new(ship_pos - central_pos, *ship_vel);
        let updated = orbit::apply_tnw_delta_v(rel, node.prograde, node.normal, node.radial);
        new_velocity = Some(updated);
        node.executed = true;
    }

    let Some(new_velocity) = new_velocity else {
        return;
    };

    for (celestial, _, mut vel, fixed) in &mut bodies {
        if fixed.is_some() || celestial.name != target_name {
            continue;
        }
        vel.0 = new_velocity;
    }
}

pub fn reset_maneuver_execution(planner: &mut RoutePlanner) {
    for node in &mut planner.nodes {
        node.executed = false;
    }
}
