use bevy::prelude::*;

use crate::components::{CelestialBody, Velocity};
use crate::resources::{
    ActiveScenario, EditorState, ReloadScenario, ScenarioCatalog, SimulationControl, SpawnProbe,
};

pub fn keyboard_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut simulation: ResMut<SimulationControl>,
    mut active: ResMut<ActiveScenario>,
    mut editor: ResMut<EditorState>,
    catalog: Res<ScenarioCatalog>,
    mut reload: MessageWriter<ReloadScenario>,
    bodies: Query<(&CelestialBody, &Velocity)>,
    mut spawn_probe_events: MessageWriter<SpawnProbe>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        simulation.paused = !simulation.paused;
    }
    if keyboard.just_pressed(KeyCode::KeyN) {
        simulation.step_once = true;
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        reload.write(ReloadScenario);
    }
    if keyboard.just_pressed(KeyCode::Equal) || keyboard.just_pressed(KeyCode::NumpadAdd) {
        simulation.speed = (simulation.speed + 0.25).min(8.0);
    }
    if keyboard.just_pressed(KeyCode::Minus) || keyboard.just_pressed(KeyCode::NumpadSubtract) {
        simulation.speed = (simulation.speed - 0.25).max(0.1);
    }
    if keyboard.just_pressed(KeyCode::Digit1) {
        switch_scenario(&mut active, &catalog, 0);
        reload.write(ReloadScenario);
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        switch_scenario(&mut active, &catalog, 1);
        reload.write(ReloadScenario);
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        switch_scenario(&mut active, &catalog, 2);
        reload.write(ReloadScenario);
    }
    if keyboard.just_pressed(KeyCode::KeyP) {
        spawn_probe_events.write(SpawnProbe);
    }
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        editor.probe_delta_v -= 0.5;
    }
    if keyboard.just_pressed(KeyCode::BracketRight) {
        editor.probe_delta_v += 0.5;
    }

    cycle_selection(&keyboard, &mut editor, &bodies);
}

fn switch_scenario(active: &mut ActiveScenario, catalog: &ScenarioCatalog, index: usize) {
    if let Some(entry) = catalog.entries.get(index) {
        active.file_path = entry.path.clone();
        active.name = entry.label.clone();
    }
}

fn cycle_selection(
    keyboard: &ButtonInput<KeyCode>,
    editor: &mut EditorState,
    bodies: &Query<(&CelestialBody, &Velocity)>,
) {
    if !keyboard.just_pressed(KeyCode::Tab) {
        return;
    }
    let names: Vec<_> = bodies
        .iter()
        .filter_map(|(b, _)| {
            if b.name.contains("Probe") {
                None
            } else {
                Some(b.name.clone())
            }
        })
        .collect();
    if names.is_empty() {
        return;
    }
    let next = if let Some(current) = &editor.selected_name {
        let idx = names.iter().position(|n| n == current).unwrap_or(0);
        names[(idx + 1) % names.len()].clone()
    } else {
        names[0].clone()
    };
    editor.selected_name = Some(next);
    editor.selection_changed = true;
    editor.velocity_dirty = false;
}
