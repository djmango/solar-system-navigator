use bevy::prelude::*;

use crate::resources::{
    ActiveScenario, DemoRecorder, ReloadScenario, ScenarioCatalog, SimulationControl,
};

pub fn demo_simulation_tuning(
    recorder: Option<Res<DemoRecorder>>,
    mut simulation: ResMut<SimulationControl>,
    mut done: Local<bool>,
) {
    if recorder.is_none() || *done {
        return;
    }
    simulation.speed = crate::astro::DEFAULT_TIME_WARP;
    simulation.ticks_per_frame = 4;
    simulation.paused = false;
    *done = true;
}

pub fn demo_scenario_cycler(
    time: Res<Time>,
    recorder: Option<Res<DemoRecorder>>,
    mut active: ResMut<ActiveScenario>,
    catalog: Res<ScenarioCatalog>,
    mut reload: MessageWriter<ReloadScenario>,
) {
    let Some(recorder) = recorder else {
        return;
    };
    let segment = recorder.duration_secs / catalog.entries.len() as f32;
    let index = (time.elapsed_secs() / segment) as usize % catalog.entries.len();
    let entry = &catalog.entries[index];
    if active.file_path != entry.path {
        active.file_path = entry.path.clone();
        active.name = entry.label.clone();
        reload.write(ReloadScenario);
    }
}

pub fn demo_auto_exit(
    time: Res<Time>,
    recorder: Option<Res<DemoRecorder>>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(recorder) = recorder else {
        return;
    };
    if time.elapsed_secs() >= recorder.duration_secs {
        exit.write(AppExit::Success);
    }
}
