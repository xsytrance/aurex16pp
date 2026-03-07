mod audio;
mod event;
mod flow;
mod frame_pacer;
mod input;
mod render;

pub use flow::{FlowController, FlowPhase};

pub use audio::{AudioEngine, AudioMode};

pub use input::poll_input;

pub use render::present_frame;

pub use frame_pacer::FramePacer;

pub use event::{RuntimeEvent, RuntimeEventQueue, SceneId};

#[derive(Default)]
pub struct RuntimeDiagnostics {
    pub scene_changed: Option<SceneId>,
    pub launch_requested: Option<&'static str>,
}

pub fn collect_runtime_diagnostics(events: &[RuntimeEvent]) -> RuntimeDiagnostics {
    let mut out = RuntimeDiagnostics::default();

    for event in events {
        match event {
            RuntimeEvent::SceneChanged(scene) => out.scene_changed = Some(*scene),
            RuntimeEvent::TitleLaunchRequested(title) => out.launch_requested = Some(*title),
            RuntimeEvent::Audio(_) => {}
        }
    }

    out
}

pub fn dispatch_runtime_events(engine: &mut AudioEngine, events: &[RuntimeEvent]) {
    for event in events {
        match event {
            RuntimeEvent::Audio(cue) => engine.trigger_cue(*cue),
            RuntimeEvent::SceneChanged(_scene) => {}
            RuntimeEvent::TitleLaunchRequested(_title) => {}
        }
    }
}
