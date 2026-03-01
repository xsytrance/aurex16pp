// ============================================================================
// DMA Controller (Phase 2)
// ----------------------------------------------------------------------------
// Responsible for enforcing per-frame transfer caps.
// This simulates hardware-level DMA arbitration.
//
// IMPORTANT:
// - No actual memory copying happens yet.
// - This module only enforces limits and tracks usage.
// - VRAM and Audio RAM are not implemented yet.
// - Reject logic will later feed into PDU-visible warnings.
//
// Likely to evolve when:
// - VRAM becomes real memory
// - Audio sample RAM is implemented
// - DMA scheduling becomes ordered/queued
// ============================================================================

use super::command::{DmaCommand, DmaTarget};

const DMA_MAX_COMMANDS_PER_FRAME: u32 = 4;
const DMA_MAX_VRAM_BYTES_PER_FRAME: u32 = 64 * 1024;
const DMA_MAX_AUDIO_BYTES_PER_FRAME: u32 = 16 * 1024;

#[derive(Debug)]
pub struct DmaController {
    // Per-frame usage
    commands_used: u32,
    vram_bytes_used: u32,
    audio_bytes_used: u32,

    // Lifetime diagnostics
    rejects_total: u64,
    rejects_this_frame: u32,
}

impl DmaController {
    pub fn new() -> Self {
        Self {
            commands_used: 0,
            vram_bytes_used: 0,
            audio_bytes_used: 0,
            rejects_total: 0,
            rejects_this_frame: 0,
        }
    }

    pub fn begin_frame(&mut self) {
        self.commands_used = 0;
        self.vram_bytes_used = 0;
        self.audio_bytes_used = 0;
        self.rejects_this_frame = 0;
    }

    /// Request a DMA transfer. Returns true if accepted, false if rejected by hardware caps.
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
        true
    }

    fn reject(&mut self) {
        self.rejects_total += 1;
        self.rejects_this_frame += 1;
    }

    // Minimal getters for later PDU integration
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
    pub fn rejects_total(&self) -> u64 {
        self.rejects_total
    }
}
