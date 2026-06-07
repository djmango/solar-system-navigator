//! Route planner: sim clock, SOI central-body switching, Hohmann helpers.

use bevy::prelude::*;

use crate::components::{
    CelestialBody, FixedBody, Mass, Position, Probe, SoiRadius, TruthOrbit, Velocity,
};
use crate::orbit::RelativeState;
use crate::resources::{EditorState, ManeuverNode, RoutePlanner, SimulationClock};
use crate::scenario::Scenario;
use crate::soi::{self, SoiBodySnapshot};
use crate::transfer::{self, HohmannTransfer};

pub fn sync_route_planner_targets(
    editor: Res<EditorState>,
    mut planner: ResMut<RoutePlanner>,
    bodies: Query<(&CelestialBody, Option<&FixedBody>, Option<&Probe>)>,
) {
    if let Some(name) = &editor.selected_name
        && bodies
            .iter()
            .any(|(body, fixed, _)| &body.name == name && fixed.is_none())
    {
        planner.target_body = Some(name.clone());
    }

    if let Some(first_probe) = bodies
        .iter()
        .find(|(_, _, probe)| probe.is_some())
        .map(|(body, _, _)| body.name.clone())
    {
        let current_is_probe = planner.target_body.as_ref().is_some_and(|target| {
            bodies
                .iter()
                .any(|(body, _, probe)| &body.name == target && probe.is_some())
        });
        if !current_is_probe {
            planner.target_body = Some(first_probe);
        }
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

    let central =
        soi::dominant_soi_body(vessel_pos.0, &snapshots, primary_name, Some(&target_name));
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

/// Default lead time for a new node placed by hotkey: a fraction of the
/// vessel's orbital period (falls back to a fixed offset when unknown).
fn default_lead_time(planner: &RoutePlanner, period: f32) -> f32 {
    if period.is_finite() && period > 0.0 {
        period * 0.25
    } else {
        planner.default_burn_offset.max(1.0)
    }
}

pub fn add_maneuver_node(planner: &mut RoutePlanner, clock: &SimulationClock, period: f32) {
    let t = clock.time + default_lead_time(planner, period);
    add_node_at_time(planner, t);
}

/// Add a fresh zero-Δv node at `time` and make it the selected node.
pub fn add_node_at_time(planner: &mut RoutePlanner, time: f32) -> u32 {
    let id = planner.next_node_id;
    planner.next_node_id = planner.next_node_id.wrapping_add(1).max(1);
    let node = ManeuverNode {
        id,
        time,
        target_body: planner.target_body.clone(),
        central_body: planner.central_body.clone(),
        prograde: 0.0,
        normal: 0.0,
        radial: 0.0,
        executed: false,
    };
    planner.nodes.push(node);
    sort_nodes(planner);
    planner.selected_node = Some(id);
    planner.sync_sliders = true;
    planner.show_previews = true;
    id
}

pub fn select_node(planner: &mut RoutePlanner, id: u32) {
    if planner.nodes.iter().any(|n| n.id == id) {
        planner.selected_node = Some(id);
        planner.sync_sliders = true;
    }
}

/// Cycle the selected node by `dir` (+1 next, -1 previous) in time order.
pub fn cycle_selected_node(planner: &mut RoutePlanner, dir: i32) {
    if planner.nodes.is_empty() {
        planner.selected_node = None;
        return;
    }
    let count = planner.nodes.len() as i32;
    let current = planner.selected_index().map(|i| i as i32);
    let next = match current {
        Some(idx) => (idx + dir).rem_euclid(count),
        None if dir >= 0 => 0,
        None => count - 1,
    };
    planner.selected_node = Some(planner.nodes[next as usize].id);
    planner.sync_sliders = true;
}

pub fn delete_selected_node(planner: &mut RoutePlanner) {
    let Some(idx) = planner.selected_index() else {
        return;
    };
    planner.nodes.remove(idx);
    planner.selected_node = planner
        .nodes
        .get(idx)
        .or_else(|| planner.nodes.last())
        .map(|n| n.id);
    planner.sync_sliders = true;
    if planner.nodes.is_empty() {
        planner.show_previews = false;
    }
}

/// Nudge the selected node's burn time, keeping nodes time-sorted.
pub fn nudge_selected_node_time(planner: &mut RoutePlanner, delta: f32) {
    if let Some(node) = planner.selected_mut() {
        node.time = (node.time + delta).max(0.0);
        sort_nodes(planner);
        planner.sync_sliders = true;
    }
}

pub fn clear_maneuver_nodes(planner: &mut RoutePlanner) {
    planner.nodes.clear();
    planner.selected_node = None;
    planner.sync_sliders = true;
    // Maneuver prediction path only; system orbit rings use `GameUx::show_system_orbits`.
    planner.show_previews = false;
}

pub fn on_simulation_reset(clock: &mut SimulationClock, planner: &mut RoutePlanner) {
    clock.time = 0.0;
    clear_maneuver_nodes(planner);
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
    period: f32,
) {
    let t0 = clock.time + default_lead_time(planner, period);
    let t1 = t0 + xfer.transfer_time;
    let departure = add_node_at_time(planner, t0);
    let arrival = add_node_at_time(planner, t1);
    if let Some(node) = planner.nodes.iter_mut().find(|n| n.id == departure) {
        node.prograde = xfer.dv_departure;
    }
    if let Some(node) = planner.nodes.iter_mut().find(|n| n.id == arrival) {
        node.prograde = xfer.dv_arrival;
    }
    planner.selected_node = Some(departure);
    planner.sync_sliders = true;
    planner.show_previews = true;
}

fn sort_nodes(planner: &mut RoutePlanner) {
    planner.nodes.sort_by(|a, b| a.time.total_cmp(&b.time));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn planner_with_target() -> RoutePlanner {
        RoutePlanner {
            target_body: Some("Probe".into()),
            central_body: Some("Sun".into()),
            ..Default::default()
        }
    }

    #[test]
    fn add_node_assigns_unique_ids_and_selects() {
        let mut planner = planner_with_target();
        let a = add_node_at_time(&mut planner, 100.0);
        let b = add_node_at_time(&mut planner, 50.0);
        assert_ne!(a, b);
        assert_eq!(planner.nodes.len(), 2);
        // Newest node is selected and nodes are kept time-sorted.
        assert_eq!(planner.selected_node, Some(b));
        assert!(planner.nodes[0].time <= planner.nodes[1].time);
        // Selection survives the re-sort.
        assert_eq!(planner.selected().map(|n| n.id), Some(b));
    }

    #[test]
    fn cycle_and_delete_track_selection() {
        let mut planner = planner_with_target();
        let first = add_node_at_time(&mut planner, 10.0);
        let second = add_node_at_time(&mut planner, 20.0);

        cycle_selected_node(&mut planner, 1);
        let after_first = planner.selected_node;
        cycle_selected_node(&mut planner, 1);
        assert_ne!(planner.selected_node, after_first);

        planner.selected_node = Some(first);
        delete_selected_node(&mut planner);
        assert_eq!(planner.nodes.len(), 1);
        assert_eq!(planner.nodes[0].id, second);
        assert!(planner.selected_node.is_some());
    }

    #[test]
    fn nudge_keeps_nodes_sorted() {
        let mut planner = planner_with_target();
        add_node_at_time(&mut planner, 10.0);
        let late = add_node_at_time(&mut planner, 20.0);
        planner.selected_node = Some(late);
        nudge_selected_node_time(&mut planner, -15.0);
        assert!(planner.nodes[0].time <= planner.nodes[1].time);
        // Time floored at zero.
        assert!(planner.nodes.iter().all(|n| n.time >= 0.0));
    }

    #[test]
    fn clear_resets_selection() {
        let mut planner = planner_with_target();
        add_node_at_time(&mut planner, 10.0);
        clear_maneuver_nodes(&mut planner);
        assert!(planner.nodes.is_empty());
        assert_eq!(planner.selected_node, None);
    }
}
