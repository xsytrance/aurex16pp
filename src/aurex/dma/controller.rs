// ============================================================================
// DMA Controller (Phase 3.5 - Apply Stage)
// ----------------------------------------------------------------------------
// Responsibilities:
// - Enforce per-frame caps (commands + byte budgets)
// - Queue accepted DMA commands
// - Apply queued transfers to hardware memory at frame end
//
// IMPORTANT DESIGN:
// - CPU never writes directly to VRAM
// - DMA validates first, applies later
// - Max 4 commands per frame
//
// Likely-to-change areas:
// - Transfer struct may later include src/dst offsets
// - Might evolve into priority scheduling
// - Could become cycle-costed instead of instant
// ============================================================================

use super::command::{DmaCommand, DmaTarget};
use crate::aurex::ppu::vram::Vram;

const DMA_MAX_COMMANDS_PER_FRAME: u32 = 4;
const DMA_MAX_VRAM_BYTES_PER_FRAME: u32 = 64 * 1024;
const DMA_MAX_AUDIO_BYTES_PER_FRAME: u32 = 16 * 1024;

pub struct DmaController {
    commands_used: u32,
    vram_bytes_used: u32,
    audio_bytes_used: u32,

    rejects_this_frame: u32,

    // Queue of accepted transfers
    queue: Vec<DmaCommand>,
}

impl DmaController {
    pub fn new() -> Self {
        Self {
            commands_used: 0,
            vram_bytes_used: 0,
            audio_bytes_used: 0,
            rejects_this_frame: 0,
            queue: Vec::new(),
        }
    }

    pub fn begin_frame(&mut self) {
        self.commands_used = 0;
        self.vram_bytes_used = 0;
        self.audio_bytes_used = 0;
        self.rejects_this_frame = 0;
        self.queue.clear();
    }

    /// CPU requests DMA transfer.
    /// Returns true if accepted, false if rejected.
    pub fn request(&mut self, cmd: DmaCommand) -> bool {
        if self.commands_used + 1 > DMA_MAX_COMMANDS_PER_FRAME {
            self.reject();
            return false;
        }

        match cmd.target {
            DmaTarget::Vram => {
                if self.vram_bytes_used + cmd.bytes > DMA_MAX_VRAM_BYTES_PER_FRAME {
                    self.reject();
                    return false;
                }
                self.vram_bytes_used += cmd.bytes;
            }
            DmaTarget::AudioRam => {
                if self.audio_bytes_used + cmd.bytes > DMA_MAX_AUDIO_BYTES_PER_FRAME {
                    self.reject();
                    return false;
                }
                self.audio_bytes_used += cmd.bytes;
            }
        }

        self.commands_used += 1;
        self.queue.push(cmd);
        true
    }

    fn reject(&mut self) {
        self.rejects_this_frame += 1;
    }

    /// Apply all accepted transfers to hardware memory.
    /// NOTE: Currently writes zeroes as placeholder data.
    /// Later will copy real memory from WRAM.
    pub fn apply(&mut self, vram: &mut Vram) {
        for cmd in &self.queue {
            match cmd.target {
                DmaTarget::Vram => {
                    // Placeholder: just write into BG tiles for now.
                    // Later this will respect explicit destination offsets.
                    let max = vram.bg_tiles.len().min(cmd.bytes as usize);
                    for i in 0..max {
                        vram.bg_tiles[i] = 1; // dummy marker value
                    }
                }
                DmaTarget::AudioRam => {
                    // Audio RAM not implemented yet.
                    // Placeholder: no-op.
                }
            }
        }
    }

    // Telemetry getters
    pub fn commands_used(&self) -> u32 {
        self.commands_used
    }
    pub fn vram_bytes_used(&self) -> u32 {
        self.vram_bytes_used
    }
    pub fn audio_bytes_used(&self) -> u32 {
        self.audio_bytes_used
    }
    pub fn rejects_this_frame(&self) -> u32 {
        self.rejects_this_frame
    }
}
