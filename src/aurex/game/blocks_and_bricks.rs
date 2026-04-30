use crate::aurex::cartridge::CartridgeRuntime;
use crate::aurex::dma::controller::DmaController;
use crate::aurex::game::InputState;
use crate::aurex::ppu::ppu::Ppu;
use crate::aurex::runtime::game_runtime::{GameOutcome, GameRuntime, PauseableGame};

const PLAYER_Y: u16 = 22;
const PLAYER_W: u16 = 6;

pub struct BlocksAndBricks {
    player_x: u16,
    falling_block_x: i16,
    falling_block_y: u16,
    block_state: BlockState,
    pause: bool,
    score: u32,
    level: u8,
    spawn_timer: u16,
}

#[derive(Clone, Copy, PartialEq)]
enum BlockState {
    Falling,
    Landed,
}

impl Default for BlocksAndBricks {
    fn default() -> Self {
        Self::new()
    }
}

impl BlocksAndBricks {
    pub fn new() -> Self {
        Self {
            player_x: 13,
            falling_block_x: 15,
            falling_block_y: 2,
            block_state: BlockState::Falling,
            pause: false,
            score: 0,
            level: 1,
            spawn_timer: 0,
        }
    }

    fn spawn_new_block(&mut self) {
        self.falling_block_x = (15 + (self.level as i16) - (self.level as i16) / 2)
            .max(0)
            .min(31);
        self.falling_block_y = 0;
        self.block_state = BlockState::Falling;
        self.spawn_timer = 60;
    }
}

impl GameRuntime for BlocksAndBricks {
    fn initialize(&mut self, _cartridge: &CartridgeRuntime) {
        self.spawn_new_block();
    }

    fn update(&mut self, input: InputState, ops_budget: u32) -> GameOutcome {
        if self.pause {
            return GameOutcome::Paused;
        }

        if ops_budget == 0 {
            return GameOutcome::Failed {
                reason: "cpu_overload",
            };
        }

        // Player movement
        let move_amt = 1u16;
        if input.left {
            self.player_x = self.player_x.saturating_sub(move_amt);
        }
        if input.right {
            self.player_x = self.player_x.saturating_add(move_amt);
        }

        // Update falling block
        self.falling_block_y += 1;

        if self.falling_block_y >= PLAYER_Y {
            self.falling_block_y = PLAYER_Y;
            self.block_state = BlockState::Landed;

            let player_center = self.player_x + (PLAYER_W / 2);
                let _block_center = self.falling_block_x as u16 + 1;
            if (player_center as i16 - self.falling_block_x as i16).abs() <= 2 {
                self.score += self.level as u32 * 10;
                self.spawn_new_block();
                self.level = ((self.level as u32 + 1) % 5 + 1) as u8;
            } else {
                return GameOutcome::Failed {
                    reason: "missed_block",
                };
            }
        }

        if self.spawn_timer > 0 {
            self.spawn_timer -= 1;
        }

        GameOutcome::Running
    }

    fn render(&self, _ppu: &mut Ppu, _dma: &mut DmaController) {
        // TODO: Render actual game graphics
    }

    fn shutdown(&mut self) {}
}

impl PauseableGame for BlocksAndBricks {
    fn toggle_pause(&mut self) -> bool {
        self.pause = !self.pause;
        self.pause
    }

    fn is_paused(&self) -> bool {
        self.pause
    }
}
