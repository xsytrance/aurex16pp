#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowPhase {
    Boot,
    Game,
}

pub struct FlowController {
    phase: FlowPhase,
    boot_frames_left: u16,
}

impl FlowController {
    pub const BOOT_FRAMES: u16 = 300; // 5 seconds @ 60 fps

    pub fn new() -> Self {
        Self {
            phase: FlowPhase::Boot,
            boot_frames_left: Self::BOOT_FRAMES,
        }
    }

    pub fn phase(&self) -> FlowPhase {
        self.phase
    }

    pub fn game_active(&self) -> bool {
        self.phase == FlowPhase::Game
    }

    pub fn tick(&mut self, skip_requested: bool) -> bool {
        if self.phase == FlowPhase::Game {
            return false;
        }

        if self.boot_frames_left > 0 {
            self.boot_frames_left -= 1;
        }

        if self.boot_frames_left == 0 || skip_requested {
            self.phase = FlowPhase::Game;
            return true;
        }

        false
    }
}
