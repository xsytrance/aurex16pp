#![allow(dead_code)]
use super::launch::{LaunchDescriptor, LaunchStage, LaunchValidationError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneId {
    Boot,
    Library,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioSfx {
    None,
    Confirm,
    Launch,
    Cancel,
    BootChime,
    PlusChime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeAudioCommand {
    PlayTrack(u8),
    PlaySfx(AudioSfx),
    PlayPcm {
        channel: u8,
        sample_id: u8,
        volume: u16,
    },
    StopTrack,
}

#[derive(Clone, Copy, Debug)]
pub enum RuntimeEvent {
    Audio(RuntimeAudioCommand),
    SceneChanged(SceneId),
    TitleLaunchRequested(LaunchDescriptor),
    TitleLaunchCanceled,
    LaunchStageChanged(LaunchStage),
    TitleLaunchReady(LaunchDescriptor),
    TitleLaunchResolved(LaunchDescriptor),
    TitleLaunchRejected(LaunchValidationError),
    // Game lifecycle events (Phase 1: Agent Console)
    GameStarted(&'static str),
    GamePaused,
    GameResumed,
    GameCompleted(u32),
    GameFailed(&'static str),
    GameCpuRejects(u32),
}

pub struct RuntimeEventQueue {
    events: Vec<RuntimeEvent>,
}

impl RuntimeEventQueue {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            events: Vec::with_capacity(cap),
        }
    }

    pub fn push(&mut self, event: RuntimeEvent) {
        self.events.push(event);
    }

    pub fn drain_to(&mut self, out: &mut Vec<RuntimeEvent>) {
        out.extend(self.events.drain(..));
    }
}
