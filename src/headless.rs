use crate::aurex::{
    Aurex,
    game::InputState,
    runtime::{
        AudioEngine, AudioMode, FlowController, FlowPhase,
        MixProfile, RuntimeEvent, dispatch_runtime_events,
    },
};

pub const SAMPLES_PER_FRAME: usize = 800; // 48000 Hz / 60 FPS

pub struct FrameOutput {
    pub framebuffer: Vec<u16>,   // 426x240 RGB555, cloned from Aurex
    pub audio_samples: Vec<i16>, // 800 stereo frames = 1600 i16 samples
    pub events: Vec<RuntimeEvent>,
    pub phase: FlowPhase,
    pub frame_number: u64,
}

pub struct HeadlessAurex {
    system: Aurex,
    audio: AudioEngine,
    flow: FlowController,
    runtime_events: Vec<RuntimeEvent>,
    frame_count: u64,
    boot_auto_advanced: bool,
}

impl HeadlessAurex {
    pub fn new(audio_profile: MixProfile) -> Self {
        Self {
            system: Aurex::new(),
            audio: AudioEngine::new_with_profile(48_000, audio_profile),
            flow: FlowController::new(),
            runtime_events: Vec::with_capacity(8),
            frame_count: 0,
            boot_auto_advanced: false,
        }
    }

    pub fn run_frame(&mut self, input: InputState) -> FrameOutput {
        // Auto-advance boot phase in headless mode
        if self.flow.phase() == FlowPhase::Boot {
            self.flow.tick(false); // tick with no press to count down boot frames
        }

        // Auto-transition from AwaitStart to Game after boot completes
        if self.flow.phase() == FlowPhase::AwaitStart && !self.boot_auto_advanced {
            if self.flow.tick(true) {
                self.system.start_game();
                self.boot_auto_advanced = true;
            }
        }

        let audio_mode = match self.flow.phase() {
            FlowPhase::Boot | FlowPhase::AwaitStart => AudioMode::Boot,
            FlowPhase::Game => AudioMode::Game,
        };

        let boot_beat_step = if matches!(audio_mode, AudioMode::Boot) {
            Some(self.audio.boot_beat_step())
        } else {
            None
        };

        self.system.run_frame(input, boot_beat_step);
        self.runtime_events.clear();
        self.system.drain_events(&mut self.runtime_events);

        // Audio: render 800 stereo samples
        let mut audio_block = [0i16; SAMPLES_PER_FRAME * 2];
        self.audio.render_block(audio_mode, &mut audio_block);

        // Dispatch audio commands from events
        dispatch_runtime_events(&mut self.audio, &self.runtime_events);

        let fb = self.system.framebuffer().pixels().to_vec();

        let output = FrameOutput {
            framebuffer: fb,
            audio_samples: audio_block.to_vec(),
            events: self.runtime_events.clone(),
            phase: self.flow.phase(),
            frame_number: self.frame_count,
        };

        self.frame_count += 1;
        output
    }

    pub fn current_scene(&self) -> crate::aurex::runtime::SceneId {
        self.system.current_scene()
    }
}
