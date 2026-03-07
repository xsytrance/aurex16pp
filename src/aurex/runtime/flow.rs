#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowPhase {
    Boot,
    AwaitStart,
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

    pub fn waiting_for_start(&self) -> bool {
        self.phase == FlowPhase::AwaitStart
    }

    pub fn tick(&mut self, start_pressed: bool) -> bool {
        match self.phase {
            FlowPhase::Boot => {
                if self.boot_frames_left > 0 {
                    self.boot_frames_left -= 1;
                }

                if self.boot_frames_left == 0 {
                    self.phase = FlowPhase::AwaitStart;
                }
                false
            }
            FlowPhase::AwaitStart => {
                if start_pressed {
                    self.phase = FlowPhase::Game;
                    true
                } else {
                    false
                }
            }
            FlowPhase::Game => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FlowController, FlowPhase};

    #[test]
    fn boot_cannot_be_skipped_early() {
        let mut flow = FlowController::new();
        for _ in 0..(FlowController::BOOT_FRAMES - 1) {
            assert!(!flow.tick(true));
            assert_eq!(flow.phase(), FlowPhase::Boot);
        }
    }

    #[test]
    fn boot_transitions_to_await_start_after_timer() {
        let mut flow = FlowController::new();
        for _ in 0..FlowController::BOOT_FRAMES {
            flow.tick(false);
        }
        assert_eq!(flow.phase(), FlowPhase::AwaitStart);
        assert!(flow.waiting_for_start());
        assert!(!flow.game_active());
    }

    #[test]
    fn await_start_requires_start_press() {
        let mut flow = FlowController::new();
        for _ in 0..FlowController::BOOT_FRAMES {
            flow.tick(false);
        }

        assert!(!flow.tick(false));
        assert_eq!(flow.phase(), FlowPhase::AwaitStart);

        assert!(flow.tick(true));
        assert_eq!(flow.phase(), FlowPhase::Game);
        assert!(flow.game_active());
    }
}
