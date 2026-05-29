//! Route planner: sim clock, SOI central-body switching, Hohmann helpers.

use bevy::prelude::*;

use crate::components::{
    CelestialBody, FixedBody, Mass, Position, Probe, SoiRadius, TruthOrbit, Velocity,
};
use crate::maneuver::reset_maneuver_execution;
use crate::orbit::RelativeState;
use crate::resources::{
    EditorState, ManeuverNode, RoutePlanner, SimulationClock, SimulationControl,
};
use crate::scenario::Scenario;
use crate::soi::{self, SoiBodySnapshot};
use crate::transfer::{self, HohmannTransfer};

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

    if planner.central_body.is_none() {
        planner.central_body = bodies
            .iter()
            .find(|(_, fixed, _)| fixed.is_some())
            .map(|(c, _, _)| c.name.clone());
    }
}

pub fn update_soi_central_body(
    active: Res<crate::resources::ActiveScenario>,
    mut planner: ResMut<RoutePlanner>,
    bodies: Query<(
        &CelestialBody,
        &Mass,
        &Position,
        &Velocity,
        &SoiRadius,
        &TruthOrbit,
        Option<&FixedBody>,
        Option<&Probe>,
    )>,
) {
    if !planner.soi_auto {
        return;
    }

    let primary_name = active
        .template
        .bodies
        .iter()
        .find(|b| b.fixed)
        .map(|b| b.name.as_str())
        .unwrap_or("Sun");

    let snapshots: Vec<SoiBodySnapshot> = build_soi_snapshots(&bodies);

    let target_name = planner.target_body.clone();
    let Some(target_name) = target_name else {
        return;
    };

    let Some((_, _, vessel_pos, _, _, _, _, _)) = bodies
        .iter()
        .find(|(c, _, _, _, _, _, _, _)| c.name == target_name)
    else {
        return;
    };

    let central = soi::dominant_soi_body(vessel_pos.0, &snapshots, primary_name);
    planner.central_body = Some(central.to_string());
}

pub fn compute_soi_radius_for_body(
    def: &crate::scenario::BodyDef,
    scenario: &Scenario,
    primary_mass: f32,
    primary_position: Vec3,
) -> f32 {
    if let Some(r) = def.soi_radius {
        return r * scenario.soi_scale;
    }
    if def.fixed {
        return f32::INFINITY;
    }
    let orbital_radius = (def.position_vec3() - primary_position)
        .length()
        .max(def.radius * 2.0);
    soi::hill_soi_radius(orbital_radius, def.mass, primary_mass) * scenario.soi_scale
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
    // Maneuver prediction path only; system orbit rings use `GameUx::show_system_orbits`.
    planner.show_previews = false;
}

pub fn on_simulation_reset(clock: &mut SimulationClock, planner: &mut RoutePlanner) {
    clock.time = 0.0;
    reset_maneuver_execution(planner);
    planner.last_hohmann = None;
}

/// Compute Hohmann from target's current circular orbit to `planner.hohmann_target_radius`.
pub fn compute_hohmann_for_target(
    planner: &mut RoutePlanner,
    physics_g: f32,
    snapshots: &[SoiBodySnapshot],
) -> Option<HohmannTransfer> {
    let central_name = planner.central_body.clone()?;
    let target_name = planner.target_body.clone()?;
    let central = soi::find_body(snapshots, &central_name)?;
    let target = soi::find_body(snapshots, &target_name)?;
    let mu = physics_g * central.mass;
    let rel = soi::relative_state_to_central(target.position, target.velocity, central);
    let r1 = transfer::circular_orbit_radius(mu, rel.position, rel.velocity)?;
    let xfer = transfer::hohmann_circular(mu, r1, planner.hohmann_target_radius).ok()?;
    planner.last_hohmann = Some(xfer);
    Some(xfer)
}

pub fn apply_hohmann_departure_draft(planner: &mut RoutePlanner, xfer: &HohmannTransfer) {
    planner.draft_prograde = xfer.dv_departure;
    planner.draft_normal = 0.0;
    planner.draft_radial = 0.0;
}

pub fn add_hohmann_maneuver_pair(
    planner: &mut RoutePlanner,
    clock: &SimulationClock,
    xfer: &HohmannTransfer,
) {
    let t0 = clock.time + planner.default_burn_offset;
    let t1 = t0 + xfer.transfer_time;
    planner.nodes.push(ManeuverNode {
        time: t0,
        prograde: xfer.dv_departure,
        normal: 0.0,
        radial: 0.0,
        executed: false,
    });
    planner.nodes.push(ManeuverNode {
        time: t1,
        prograde: xfer.dv_arrival,
        normal: 0.0,
        radial: 0.0,
        executed: false,
    });
    planner.show_previews = true;
}

pub fn build_soi_snapshots(
    bodies: &Query<(
        &CelestialBody,
        &Mass,
        &Position,
        &Velocity,
        &SoiRadius,
        &TruthOrbit,
        Option<&FixedBody>,
        Option<&Probe>,
    )>,
) -> Vec<SoiBodySnapshot> {
    bodies
        .iter()
        .map(|(c, m, p, v, soi, _truth, fixed, _probe)| SoiBodySnapshot {
            name: c.name.clone(),
            position: p.0,
            velocity: v.0,
            mass: m.0,
            soi_radius: soi.0,
            is_primary: fixed.is_some(),
        })
        .collect()
}

#[allow(dead_code)]
pub fn relative_target_state(
    snapshots: &[SoiBodySnapshot],
    central_name: &str,
    target_name: &str,
) -> Option<RelativeState> {
    let central = soi::find_body(snapshots, central_name)?;
    let target = soi::find_body(snapshots, target_name)?;
    Some(soi::relative_state_to_central(
        target.position,
        target.velocity,
        central,
    ))
}
