#![allow(dead_code)]
//! Game Runtime Trait - Abstract interface for cartridge game execution
//! 
//! This trait defines the lifecycle for any cartridge-based game running on Aurex-16++.
//! Games are instantiated after `TitleLaunchResolved` and run deterministically.

use crate::aurex::cartridge::CartridgeRuntime;
use crate::aurex::dma::controller::DmaController;
use crate::aurex::game::InputState;
use crate::aurex::ppu::ppu::Ppu;
use crate::aurex::ppu::vram::Vram;

use crate::aurex::ppu::framebuffer::{FB_W, FB_H, rgb555};
use crate::aurex::ppu::oam::{Sprite, MAX_SPRITES};
use crate::aurex::ppu::ppu::PPU_SPRITE_ENABLE;

/// Outcome of a game update cycle
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameOutcome {
    /// Game is still running
    Running,
    
    /// Game is paused (awaiting resume)
    Paused,
    
    /// Game completed with final score
    Completed { score: u32 },
    
    /// Game failed with reason
    Failed { reason: &'static str },
}

/// Trait for cartridge-backed game execution
pub trait GameRuntime: Send {
    /// Initialize game with cartridge data
    ///
    /// Called once after `TitleLaunchResolved` to set up game state,
    /// upload VRAM graphics, configure PPU, and prepare for the first update frame.
    fn initialize(&mut self, cartridge: &CartridgeRuntime, vram: &mut Vram, ppu: &mut Ppu);
    
    /// Update game state for one frame
    /// 
    /// # Arguments
    /// * `input` - Current player input state
    /// * `ops_budget` - Remaining CPU ops for this frame (from PDU telemetry)
    /// 
    /// # Returns
    /// Game outcome (Running, Paused, Completed, or Failed)
    fn update(&mut self, input: InputState, ops_budget: u32) -> GameOutcome;
    
    /// Render current game state to PPU
    /// 
    /// Called after update() to draw the current frame.
    /// Queue DMA commands for tilemap, sprites, palettes as needed.
    fn render(&self, ppu: &mut Ppu, dma: &mut DmaController);
    
    /// Shutdown game and cleanup resources
    ///
    /// Called when game is destroyed or cartridge is unloaded.
    fn shutdown(&mut self);

    /// Generate bot input for automated gameplay (optional)
    ///
    /// Default implementation returns None (no bot).
    /// Games can override to provide AI-assisted play for demos/replays.
    fn bot_input(&self) -> Option<InputState> {
        None
    }
}

/// Marker trait for games that support pause/resume
pub trait PauseableGame: GameRuntime {
    /// Toggle pause state
    fn toggle_pause(&mut self) -> bool;
    
    /// Check if currently paused
    fn is_paused(&self) -> bool;
}

/// Marker trait for games that support achievements
pub trait AchievableGame: GameRuntime {
    /// Check achievements for current game state
    fn check_achievements(&self, game_id: &str);
}


/// Simple animated demo: a bouncing ball.
/// Acts as the fallback game when no cartridge is loaded.
pub struct NoopGame {
    x: i32,
    y: i32,
    dx: i8,
    dy: i8,
    color: u16,
    frame: u64,
}

impl NoopGame {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            dx: 0,
            dy: 0,
            color: 0,
            frame: 0,
        }
    }
}

impl NoopGame {
    fn upload_sprite_tile(vram: &mut Vram, color_idx: u8) {
        // Fill an 8x8 4bpp tile (32 bytes) with a solid color index.
        // Each byte packs two pixels (high nibble, low nibble).
        let packed = (color_idx as u8) << 4 | color_idx as u8;
        for row in 0..8 {
            let base = row * 4;
            vram.sprite_tiles[base] = packed;
            vram.sprite_tiles[base + 1] = packed;
            vram.sprite_tiles[base + 2] = packed;
            vram.sprite_tiles[base + 3] = packed;
        }
    }
}

impl GameRuntime for NoopGame {
    fn initialize(&mut self, _cartridge: &CartridgeRuntime, vram: &mut Vram, ppu: &mut Ppu) {
        // Initial position (center)
        self.x = (FB_W as i32 / 2 - 4) as i32;
        self.y = (FB_H as i32 / 2 - 4) as i32;
        self.dx = 2;
        self.dy = 2;
        self.color = rgb555(31, 10, 10); // reddish
        self.frame = 0;

        // Palette: slot 1
        let col = self.color;
        let pal_idx = 1 * 2;
        vram.palettes[pal_idx] = (col & 0xFF) as u8;
        vram.palettes[pal_idx + 1] = (col >> 8) as u8;

        // Upload sprite tile 0 as solid square using palette index 1
        NoopGame::upload_sprite_tile(vram, 1);

        // Enable sprites
        ppu.write_addr(PPU_SPRITE_ENABLE, 1);
    }

    fn update(&mut self, input: InputState, _ops_budget: u32) -> GameOutcome {
        // Move
        self.x += self.dx as i32;
        self.y += self.dy as i32;

        // Bounce off edges, keeping within screen bounds
        if self.x <= 0 {
            self.x = 0;
            self.dx = -self.dx;
        } else if self.x >= (FB_W as i32) - 8 {
            self.x = (FB_W as i32) - 8;
            self.dx = -self.dx;
        }
        if self.y <= 0 {
            self.y = 0;
            self.dy = -self.dy;
        } else if self.y >= (FB_H as i32) - 8 {
            self.y = (FB_H as i32) - 8;
            self.dy = -self.dy;
        }

        // Nudge with input
        if input.up { self.y -= 1; }
        if input.down { self.y += 1; }
        if input.left { self.x -= 1; }
        if input.right { self.x += 1; }

        self.frame = self.frame.wrapping_add(1);
        GameOutcome::Running
    }

    fn render(&self, ppu: &mut Ppu, _dma: &mut DmaController) {
        // Clear all sprites to avoid trails
        for i in 0..MAX_SPRITES {
            ppu.debug_set_sprite(i, Sprite::default());
        }
        // Draw the ball as sprite 0
        let mut s = Sprite::default();
        s.x = self.x as u16;
        s.y = self.y as u16;
        s.tile_index = 0;
        s.palette = 1; // use palette entry 1
        s.visible = true;
        ppu.debug_set_sprite(0, s);
    }

    fn shutdown(&mut self) {}
}
