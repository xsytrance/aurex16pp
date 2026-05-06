use crate::aurex::game::InputState;
use crate::headless::HeadlessAurex;
use crate::recorder::{SessionRecorder, RecorderConfig};
use crate::aurex::runtime::MixProfile;

pub trait InputStrategy: Send {
    fn decide_input(&mut self, frame_number: u64, framebuffer: &[u16]) -> InputState;
    fn name(&self) -> &'static str;
}

pub struct ExplorerStrategy {
    rng_seed: u64,
}

impl InputStrategy for ExplorerStrategy {
    fn decide_input(&mut self, frame: u64, _fb: &[u16]) -> InputState {
        // Simple deterministic pseudo-random exploration
        let hash = self.rng_seed.wrapping_add(frame.wrapping_mul(0x9E3779B97F4A7C15));
        let bits = (hash >> 32) as u32;
        InputState {
            left: bits & 1 != 0 && frame % 20 < 10,
            right: bits & 2 != 0 && frame % 25 < 12,
            up: bits & 4 != 0 && frame % 30 < 8,
            down: bits & 8 != 0 && frame % 35 < 10,
            accept: bits & 16 != 0 && frame % 45 == 0,
            cancel: bits & 32 != 0 && frame % 60 == 0,
        }
    }
    fn name(&self) -> &'static str { "explorer" }
}

pub struct PassiveStrategy;
impl InputStrategy for PassiveStrategy {
    fn decide_input(&mut self, _frame: u64, _fb: &[u16]) -> InputState {
        InputState::default()
    }
    fn name(&self) -> &'static str { "passive" }
}

pub struct AggressiveStrategy;
impl InputStrategy for AggressiveStrategy {
    fn decide_input(&mut self, frame: u64, _fb: &[u16]) -> InputState {
        InputState {
            left: frame % 8 < 3,
            right: frame % 8 >= 3 && frame % 8 < 6,
            up: frame % 12 < 4,
            down: frame % 12 >= 6,
            accept: frame % 5 == 0,
            cancel: false,
        }
    }
    fn name(&self) -> &'static str { "aggressive" }
}

pub struct PrimePilotStrategy;
impl InputStrategy for PrimePilotStrategy {
    fn decide_input(&mut self, frame: u64, _fb: &[u16]) -> InputState {
        // "Manual-style" authored input macro: strafe, micro-adjust, and burst-fire cadence.
        let phase = frame % 120;
        let left = (phase >= 8 && phase < 24) || (phase >= 72 && phase < 84);
        let right = (phase >= 36 && phase < 56) || (phase >= 92 && phase < 110);
        let up = phase % 30 < 3; // tiny lift taps
        let down = phase % 40 >= 28 && phase % 40 < 31; // rare corrective dips
        let accept = frame % 3 == 0 || (phase >= 56 && phase < 64); // sustained fire window

        InputState {
            left,
            right,
            up,
            down,
            accept,
            cancel: false,
        }
    }
    fn name(&self) -> &'static str { "prime" }
}

pub fn strategy_by_name(name: &str) -> Box<dyn InputStrategy> {
    match name {
        "explorer" => Box::new(ExplorerStrategy { rng_seed: 0xDEADBEEF }),
        "passive" => Box::new(PassiveStrategy),
        "aggressive" => Box::new(AggressiveStrategy),
        "prime" => Box::new(PrimePilotStrategy),
        _ => Box::new(ExplorerStrategy { rng_seed: 0xDEADBEEF }),
    }
}

pub struct SessionResult {
    pub recording_path: Option<String>,
    pub frames_played: u64,
    pub strategy_name: String,
}

pub struct AgentSession {
    runtime: HeadlessAurex,
    recorder: Option<SessionRecorder>,
    strategy: Box<dyn InputStrategy>,
    recording_path: Option<String>,
}

impl AgentSession {
    pub fn new(
        _game_id: &str,
        strategy: Box<dyn InputStrategy>,
        record: bool,
        output_dir: &str,
        audio_profile: MixProfile,
    ) -> Result<Self, String> {
        let recorder = if record {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let out_path = format!("{}/session_{}_{}.mp4", output_dir, _game_id, timestamp);

            // Ensure output dir exists
            std::fs::create_dir_all(output_dir).map_err(|e| format!("create dir: {}", e))?;

            Some(SessionRecorder::new(RecorderConfig {
                width: 426,
                height: 240,
                fps: 60,
                sample_rate: 48000,
                output_path: out_path.clone(),
            }).map_err(|e| format!("recorder: {}", e))?)
        } else {
            None
        };

        Ok(Self {
            runtime: HeadlessAurex::new(audio_profile),
            recorder,
            strategy,
            recording_path: None,
        })
    }

    pub fn run_for_frames(&mut self, max_frames: u64) -> Result<SessionResult, String> {
        for frame in 0..max_frames {
            let input = self.strategy.decide_input(frame, &[]);
            let output = self.runtime.run_frame(input);

            if let Some(ref mut rec) = self.recorder {
                rec.write_frame(&output.framebuffer, &output.audio_samples)
                    .map_err(|e| format!("frame {}: {}", frame, e))?;
            }
        }

        let path = if let Some(recorder) = self.recorder.take() {
            Some(recorder.finish()?)
        } else {
            None
        };

        Ok(SessionResult {
            recording_path: path,
            frames_played: max_frames,
            strategy_name: self.strategy.name().to_string(),
        })
    }
}
