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
use crate::aurex::dma::command::VramRegion;
use crate::aurex::ppu::vram::{MAX_PALETTE_ENTRIES, VRAM_I_BASE, VRAM_I_END, Vram};
use crate::aurex::wram::Wram;

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

        if cmd.region == VramRegion::Palettes {
            let palette_bytes = MAX_PALETTE_ENTRIES * 2;
            if cmd.dst_offset + cmd.bytes > palette_bytes {
                debug_assert!(
                    cmd.dst_offset + cmd.bytes <= palette_bytes,
                    "palette DMA overflow: dst={} bytes={} limit={}\n",
                    cmd.dst_offset,
                    cmd.bytes,
                    palette_bytes
                );
                return self.reject();
            }
        }

        // -------------------------------------------------------------------------
        // HARDWARE ENFORCEMENT: REGION BOUNDARY
        // -------------------------------------------------------------------------
        if cmd.dst_offset + cmd.bytes > region_len {
            #[cfg(debug_assertions)]
            panic!("DMA transfer exceeds region boundary");

            #[cfg(not(debug_assertions))]
            {
                return;
            }
        }

        // ⬇⬇⬇ PASTE RESERVED CHECK HERE ⬇⬇⬇

        // -------------------------------------------------------------------------
        // HARDWARE ENFORCEMENT: RESERVED VRAM REGION
        // -------------------------------------------------------------------------
        if cmd.bytes > 0 {
            let (base, _) = vram.region_bounds_abs(&cmd.region);
            let abs_start = base + cmd.dst_offset;
            let abs_end = abs_start + cmd.bytes - 1;

            let intersects_reserved = abs_start <= VRAM_I_END && abs_end >= VRAM_I_BASE;

            if intersects_reserved {
                #[cfg(debug_assertions)]
                panic!("DMA attempted write into reserved VRAM region");

                #[cfg(not(debug_assertions))]
                {
                    return;
                }
            }
        }

        // -------------------------------------------------------------------------
        // HARDWARE ENFORCEMENT: MODE 7 ISOLATION
        // BG2 (Mode7Tex) must reside strictly within Mode 7 reserved regions
        // -------------------------------------------------------------------------
        if cmd.region == VramRegion::Mode7Tex && cmd.bytes > 0 {
            let (base, _) = vram.region_bounds_abs(&cmd.region);
            let abs_start = base + cmd.dst_offset;
            let abs_end = abs_start + cmd.bytes - 1;

            let inside_mode7_region = (abs_start >= 0x94000 && abs_end <= 0xA3FFF) || // Region E
        (abs_start >= 0xA4000 && abs_end <= 0xD3FFF); // Region F

            if !inside_mode7_region {
                #[cfg(debug_assertions)]
                panic!("Mode 7 DMA attempted outside reserved Mode 7 regions");

                #[cfg(not(debug_assertions))]
                {
                    return;
                }
            }
        }

        // ⬇ actual copy logic continues below

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

    pub fn apply(&mut self, wram: &Wram, vram: &mut Vram, vblank: bool) {
        // -------------------------------------------------------------
        // Phase 6: VBlank gating
        // VRAM writes are only allowed during VBlank.
        // Deterministic rejection if outside VBlank.
        // -------------------------------------------------------------
        if !vblank {
            return;
        }

        for cmd in &self.queue {
            let src = &wram.memory()[cmd.src_offset..cmd.src_offset + cmd.bytes];

            let dst_slice = vram.region_mut(&cmd.region);

            let dst = &mut dst_slice[cmd.dst_offset..cmd.dst_offset + cmd.bytes];

            if cmd.region == VramRegion::Palettes {
                debug_assert_eq!(
                    cmd.dst_offset % 2,
                    0,
                    "palette DMA offset must be 16-bit aligned"
                );
                debug_assert_eq!(cmd.bytes % 2, 0, "palette DMA size must be 16-bit aligned");
                let start_index = cmd.dst_offset / 2;
                let entry_count = cmd.bytes / 2;
                debug_assert!(
                    start_index + entry_count <= MAX_PALETTE_ENTRIES,
                    "palette entry overflow: start={} count={} max={}",
                    start_index,
                    entry_count,
                    MAX_PALETTE_ENTRIES
                );
            }

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
