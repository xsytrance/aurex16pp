use crate::aurex::cartridge::CartridgeRuntime;
use crate::aurex::dma::controller::DmaController;
use crate::aurex::game::InputState;
use crate::aurex::ppu::oam::{Sprite, MAX_SPRITES};
use crate::aurex::ppu::ppu::{Ppu, PPU_SPRITE_ENABLE};
use crate::aurex::ppu::vram::Vram;
use crate::aurex::runtime::game_runtime::{GameOutcome, GameRuntime, PauseableGame};

const PLAYER_Y: u16 = 22;
const PLAYER_W: u16 = 6;
const PLAYER_H: u16 = 2;
const BLOCK_SIZE: u16 = 4;
const PLAYFIELD_Y_START: u16 = 2;

pub struct BlocksAndBricks {
    player_x: u16,
    falling_block_x: i16,
    falling_block_y: u16,
    block_state: BlockState,
    pause: bool,
    score: u32,
    level: u8,
    spawn_timer: u16,
    last_score: u32,
    game_over: bool,
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
            last_score: 0,
            game_over: false,
        }
    }

    fn spawn_new_block(&mut self) {
        self.falling_block_x = (15 + (self.level as i16) - (self.level as i16) / 2)
            .max(0)
            .min(27);
        self.falling_block_y = PLAYFIELD_Y_START;
        self.block_state = BlockState::Falling;
        self.spawn_timer = if self.level <= 3 { 90 } else { 60 } as u16;
    }

    fn set_sprite(ppu: &mut Ppu, index: usize, x: u16, y: u16, tile_idx: u16, palette: u16) {
        if index < MAX_SPRITES {
            let mut sprite = Sprite::default();
            sprite.x = x;
            sprite.y = y;
            sprite.tile_index = tile_idx;
            sprite.palette = palette;
            sprite.visible = true;
            sprite.priority = 1; // Game sprites over background
            ppu.debug_set_sprite(index, sprite);
        }
    }
}

impl GameRuntime for BlocksAndBricks {
    fn initialize(&mut self, _cartridge: &CartridgeRuntime, vram: &mut Vram, ppu: &mut Ppu)  {
        println!("[DEBUG] BlocksAndBricks::initialize called");

        // Palette helper: write RGB555 (little-endian) to VRAM
        fn write_palette(vram: &mut Vram, idx: usize, color: u16) {
            let off = idx * 2;
            vram.palettes[off] = color as u8;
            vram.palettes[off + 1] = (color >> 8) as u8;
        }

        // Sprite palette mapping: effective index = bank * 16 + pixel_value(1)
        write_palette(vram, 0,   0x0000); // transparent
        write_palette(vram, 17,  0x001F); // blue   (bank 1)
        write_palette(vram, 33,  0x03E0); // green  (bank 2)
        write_palette(vram, 49,  0x7FE0); // yellow (bank 3)
        write_palette(vram, 65,  0x7C00); // red    (bank 4)
        write_palette(vram, 241, 0x7FFF); // white  (paddle, bank 0x0F)

        // Sprite tiles (8x8, 4bpp = 32 bytes each)
        let tile0: [u8; 32] = [
            0,0,0,0, 0,0,0,0,
            0,0,1,1, 0,0,1,1,
            0,0,1,1, 0,0,1,1,
            0,0,0,0, 0,0,0,0,
        ];
        let tile1: [u8; 32] = [
            0,0,0,0, 0,0,0,0,
            0,0,0,0, 0,17,17,0,
            0,17,17,0, 0,0,0,0,
            0,0,0,0, 0,0,0,0,
        ];
        let tile2: [u8; 32] = [0; 32];

        vram.sprite_tiles[0..32].copy_from_slice(&tile0);
        vram.sprite_tiles[32..64].copy_from_slice(&tile1);
        vram.sprite_tiles[64..96].copy_from_slice(&tile2);

        self.spawn_new_block();

        // Re-enable sprites (boot ROM disables them)
        ppu.write_addr(PPU_SPRITE_ENABLE, 1);
    }

    fn update(&mut self, input: InputState, ops_budget: u32) -> GameOutcome {
        if self.pause {
            return GameOutcome::Paused;
        }

        if self.game_over {
            // Check for restart
            if input.accept {
                // Restart game
                *self = Self::new();
            }
            return GameOutcome::Running;
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
        // Clamp to playfield
        self.player_x = self.player_x.clamp(0, 31 - PLAYER_W);

        // Update falling block
        self.falling_block_y += 1;

        if self.falling_block_y >= PLAYER_Y {
            self.falling_block_y = PLAYER_Y;
            self.block_state = BlockState::Landed;

            let player_center = self.player_x + (PLAYER_W / 2);
            let _block_center = (self.falling_block_x as u16) + (BLOCK_SIZE / 2);
            if (player_center as i16 - self.falling_block_x as i16).abs() <= 2 {
                // Caught!
                self.score += self.level as u32 * 10;
                self.spawn_new_block();
                self.level = ((self.level as u32 + 1) % 5 + 1) as u8;
            } else {
                // Missed!
                self.game_over = true;
                return GameOutcome::Failed {
                    reason: "missed_block",
                };
            }
        }

        // Spawn timer
        if self.spawn_timer > 0 {
            self.spawn_timer -= 1;
        }

        GameOutcome::Running
    }

    fn render(&self, ppu: &mut Ppu, _dma: &mut DmaController) {
        println!("[DEBUG-render]");
        // Clear all sprites first
        for i in 0..MAX_SPRITES {
            ppu.debug_set_sprite(i, Sprite::default());
        }

        // Draw falling block (sprite 0)
        let block_color = match self.level {
            1 => 0x01, // Blue
            2 => 0x02, // Green
            3 => 0x03, // Yellow
            4 => 0x04, // Red
            _ => 0x01,
        };
        
        Self::set_sprite(ppu, 0, 
            self.falling_block_x as u16, 
            self.falling_block_y, 
            0, // tile_index (would be animated in production)
            block_color as u16
        );

        // Draw player paddle (sprite 1)
        Self::set_sprite(ppu, 1, 
            self.player_x, 
            PLAYER_Y, 
            1, // different tile for player
            0x0F // white/high contrast
        );

        // Draw score display (sprite 2 - could use for HUD)
        // For now, just mark it for future text rendering
        Self::set_sprite(ppu, 2, 
            24, // top-right
            0, 
            2, // score tile
            0x0F
        );

        // Draw overlays
        if self.pause {
            Self::set_sprite(ppu, 3, 12, 11, 3, 0x0F); // "PAUSED" indicator
        } else if self.game_over {
            Self::set_sprite(ppu, 3, 10, 10, 4, 0x0F); // "GAME OVER" indicator
        }
    }

    fn shutdown(&mut self) {
        // Cleanup
    }

    fn bot_input(&self) -> Option<InputState> {
        // Simple AI: center paddle under the falling block
        let target = self.falling_block_x as i16;
        let player_center = self.player_x as i16 + (PLAYER_W as i16) / 2;
        let mut input = InputState::default();
        if target > player_center + 1 {
            input.right = true;
        } else if target < player_center - 1 {
            input.left = true;
        }
        // Bot never presses up/down/accept/cancel
        Some(input)
    }
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
