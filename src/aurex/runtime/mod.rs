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

pub fn dispatch_runtime_events(engine: &mut AudioEngine, events: &[RuntimeEvent]) {
    for event in events {
        match event {
            RuntimeEvent::Audio(cue) => engine.trigger_cue(*cue),
            RuntimeEvent::SceneChanged(_scene) => {}
        }
    }
}
