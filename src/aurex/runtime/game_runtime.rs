#![allow(dead_code)]
//! Game Runtime Trait - Abstract interface for cartridge game execution
//! 
//! This trait defines the lifecycle for any cartridge-based game running on Aurex-16++.
//! Games are instantiated after `TitleLaunchResolved` and run deterministically.

use crate::aurex::cartridge::CartridgeRuntime;
use crate::aurex::dma::controller::DmaController;
use crate::aurex::game::InputState;
use crate::aurex::ppu::ppu::Ppu;

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
    /// queue VRAM/audio uploads, and prepare for the first update frame.
    fn initialize(&mut self, cartridge: &CartridgeRuntime);
    
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

/// Default no-op implementation for compatibility
pub struct NoopGame;

impl GameRuntime for NoopGame {
    fn initialize(&mut self, _cartridge: &CartridgeRuntime) {}
    
    fn update(&mut self, _input: InputState, _ops_budget: u32) -> GameOutcome {
        GameOutcome::Running
    }
    
    fn render(&self, _ppu: &mut Ppu, _dma: &mut DmaController) {}
    
    fn shutdown(&mut self) {}
}
