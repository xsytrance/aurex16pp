#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowPhase {
    Boot,
    Confirming,
    Game,
}

pub struct FlowController {
    phase: FlowPhase,
    confirm_frames_left: u8,
}

impl FlowController {
    pub const CONFIRM_FRAMES: u8 = 32;

    pub fn new() -> Self {
        Self {
            phase: FlowPhase::Boot,
            confirm_frames_left: 0,
        }
    }

    pub fn phase(&self) -> FlowPhase {
        self.phase
    }

    pub fn game_active(&self) -> bool {
        self.phase == FlowPhase::Game
    }

    pub fn boot_confirming(&self) -> bool {
        self.phase == FlowPhase::Confirming
    }

    pub fn register_start_request(&mut self) -> bool {
        if self.phase != FlowPhase::Boot {
            return false;
        }

        self.phase = FlowPhase::Confirming;
        self.confirm_frames_left = Self::CONFIRM_FRAMES;
        true
    }

    pub fn tick(&mut self) -> bool {
        if self.phase != FlowPhase::Confirming {
            return false;
        }

        if self.confirm_frames_left > 0 {
            self.confirm_frames_left -= 1;
            return false;
        }

        self.phase = FlowPhase::Game;
        true
    }
}
