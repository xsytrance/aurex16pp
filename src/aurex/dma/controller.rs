// ============================================================================
// DMA Controller (Phase 3.6 - Real Transfers)
// ----------------------------------------------------------------------------
// Responsibilities:
// - Enforce per-frame caps
// - Validate WRAM and VRAM bounds at request time
// - Queue valid transfers
// - Apply transfers WRAM -> VRAM at frame end
//
// IMPORTANT:
// - Invalid DMA is rejected immediately (Option A).
// - No clipping or partial writes.
// ============================================================================

use super::command::DmaCommand;
use crate::aurex::{ppu::vram::Vram, wram::Wram};

const DMA_MAX_COMMANDS_PER_FRAME: u32 = 4;
const DMA_MAX_VRAM_BYTES_PER_FRAME: u32 = 64 * 1024;
const DMA_MAX_AUDIO_BYTES_PER_FRAME: u32 = 16 * 1024;

pub struct DmaController {
    commands_used: u32,
    vram_bytes_used: u32,
    rejects_this_frame: u32,
    queue: Vec<DmaCommand>,
    audio_bytes_used: u32,
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
        self.rejects_this_frame = 0;
        self.audio_bytes_used = 0;
        self.queue.clear();
    }

    pub fn request(&mut self, cmd: DmaCommand, wram: &Wram, vram: &Vram) -> bool {
        // Cap: number of commands
        if self.commands_used + 1 > DMA_MAX_COMMANDS_PER_FRAME {
            return self.reject();
        }

        // Cap: total VRAM bytes per frame
        if self.vram_bytes_used + cmd.bytes as u32 > DMA_MAX_VRAM_BYTES_PER_FRAME {
            return self.reject();
        }

        // Cap: total AUDIO bytes per frame
        if cmd.is_audio() {
            if self.audio_bytes_used + cmd.bytes as u32 > DMA_MAX_AUDIO_BYTES_PER_FRAME {
                return self.reject();
            }
        }

        // Validate WRAM bounds
        if cmd.src_offset + cmd.bytes > wram.len() {
            return self.reject();
        }

        // Validate VRAM bounds
        let region_len = vram.region_len(&cmd.region);

        if cmd.dst_offset + cmd.bytes > region_len {
            return self.reject();
        }

        self.commands_used += 1;

        if cmd.is_audio() {
            self.audio_bytes_used += cmd.bytes as u32;
        } else {
            self.vram_bytes_used += cmd.bytes as u32;
        }
        self.queue.push(cmd);
        true
    }

    fn reject(&mut self) -> bool {
        self.rejects_this_frame += 1;
        false
    }

    pub fn apply(&mut self, wram: &Wram, vram: &mut Vram) {
        for cmd in &self.queue {
            let src = &wram.memory()[cmd.src_offset..cmd.src_offset + cmd.bytes];

            let dst_slice = vram.region_mut(&cmd.region);

            let dst = &mut dst_slice[cmd.dst_offset..cmd.dst_offset + cmd.bytes];

            dst.copy_from_slice(src);
        }
    }

    pub fn commands_used(&self) -> u32 {
        self.commands_used
    }
    pub fn vram_bytes_used(&self) -> u32 {
        self.vram_bytes_used
    }
    pub fn rejects_this_frame(&self) -> u32 {
        self.rejects_this_frame
    }
}
