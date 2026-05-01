mod audio;
mod event;
mod flow;
mod frame_pacer;
pub mod game_runtime;
#[cfg(feature = "sdl2")]
mod input;
mod launch;
#[cfg(feature = "sdl2")]
mod render;
mod replay;

pub use audio::{AudioEngine, AudioMode, AudioRecorder, MixProfile};
pub use flow::{FlowController, FlowPhase};

pub const AUDIO_TRACK_COUNT: usize = audio::TRACK_COUNT;

pub use game_runtime::{GameOutcome, GameRuntime};

#[cfg(feature = "sdl2")]
pub use input::poll_input;
#[cfg(feature = "sdl2")]
pub use render::present_frame;
pub use frame_pacer::FramePacer;
pub use replay::ReplayCapture;

pub use event::{AudioSfx, RuntimeAudioCommand, RuntimeEvent, RuntimeEventQueue, SceneId};
pub use launch::{
    validate_launch_descriptor, LaunchDescriptor, LaunchIntentController, LaunchStage,
    LaunchValidationError,
};

#[derive(Default)]
pub struct RuntimeDiagnostics {
    pub scene_changed: Option<SceneId>,
    pub launch_requested: Option<LaunchDescriptor>,
    pub launch_canceled: bool,
    pub launch_stage_changed: Option<LaunchStage>,
    pub launch_ready: Option<LaunchDescriptor>,
    pub launch_resolved: Option<LaunchDescriptor>,
    pub launch_rejected: Option<LaunchValidationError>,
    // Game lifecycle (Phase 1: Agent Console)
    pub game_started: Option<&'static str>,
    pub game_paused: bool,
    pub game_resumed: bool,
    pub game_completed: Option<u32>,
    pub game_failed: Option<&'static str>,
    pub game_cpu_rejects: Option<u32>,
}

pub fn collect_runtime_diagnostics(events: &[RuntimeEvent]) -> RuntimeDiagnostics {
    let mut out = RuntimeDiagnostics::default();

    for event in events {
        match event {
            RuntimeEvent::Audio(_) => {}
            RuntimeEvent::SceneChanged(scene) => out.scene_changed = Some(*scene),
            RuntimeEvent::TitleLaunchRequested(req) => out.launch_requested = Some(*req),
            RuntimeEvent::TitleLaunchCanceled => out.launch_canceled = true,
            RuntimeEvent::LaunchStageChanged(stage) => out.launch_stage_changed = Some(*stage),
            RuntimeEvent::TitleLaunchReady(desc) => out.launch_ready = Some(*desc),
            RuntimeEvent::TitleLaunchResolved(desc) => out.launch_resolved = Some(*desc),
            RuntimeEvent::TitleLaunchRejected(reason) => out.launch_rejected = Some(*reason),
            // Game lifecycle (Phase 1: Agent Console)
            RuntimeEvent::GameStarted(cartridge_id) => out.game_started = Some(cartridge_id),
            RuntimeEvent::GamePaused => out.game_paused = true,
            RuntimeEvent::GameResumed => out.game_resumed = true,
            RuntimeEvent::GameCompleted(score) => out.game_completed = Some(*score),
            RuntimeEvent::GameFailed(reason) => out.game_failed = Some(*reason),
            RuntimeEvent::GameCpuRejects(count) => out.game_cpu_rejects = Some(*count),
        }
    }

    out
}

pub fn dispatch_runtime_events(engine: &mut AudioEngine, events: &[RuntimeEvent]) {
    for event in events {
        match event {
            RuntimeEvent::Audio(cmd) => engine.trigger_command(*cmd),
            RuntimeEvent::SceneChanged(_scene) => {}
            RuntimeEvent::TitleLaunchRequested(_title) => {}
            RuntimeEvent::TitleLaunchCanceled => {}
            RuntimeEvent::LaunchStageChanged(_stage) => {}
            RuntimeEvent::TitleLaunchReady(_desc) => {}
            RuntimeEvent::TitleLaunchResolved(_desc) => {}
            RuntimeEvent::TitleLaunchRejected(_reason) => {}
            // Game lifecycle (Phase 1: Agent Console)
            RuntimeEvent::GameStarted(_cartridge_id) => {}
            RuntimeEvent::GamePaused => {}
            RuntimeEvent::GameResumed => {}
            RuntimeEvent::GameCompleted(_score) => {}
            RuntimeEvent::GameFailed(_reason) => {}
            RuntimeEvent::GameCpuRejects(_count) => {}
        }
    }
}
