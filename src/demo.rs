use bevy::prelude::*;

use crate::resources::{ActiveScenario, DemoRecorder, ReloadScenario, ScenarioCatalog};

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
    let segment = recorder.max_frames as f32 / 30.0 / 3.0;
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
    let duration = recorder.max_frames as f32 / 30.0;
    if time.elapsed_secs() >= duration {
        exit.write(AppExit::Success);
    }
}
