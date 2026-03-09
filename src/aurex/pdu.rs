#![allow(dead_code)]
// ============================================================================
// Performance Diagnostic Unit (PDU)
// ----------------------------------------------------------------------------
// Tracks per-frame hardware usage and enforces CPU operation caps.
//
// IMPORTANT:
// - Hard 200,000 ops per frame.
// - Frame-based (no time deltas).
// - Will later aggregate DMA usage and audio usage.
//
// Likely to evolve when:
// - Overlay UI is implemented
// - Logging system becomes configurable
// - Budget telemetry is exposed to Library cart
// ============================================================================

pub struct Pdu {
    // CPU usage
    ops_used: u32,
    cpu_rejects: u32,

    // DMA telemetry (read-only from DMA each frame)
    dma_commands_used: u32,
    dma_vram_bytes_used: u32,
    dma_audio_bytes_used: u32,
    dma_rejects: u32,

    // -----------------------------------------------------------------
    // PPU telemetry (latched per frame)
    // -----------------------------------------------------------------
    ppu_sprite_overflow: bool,
    ppu_sprite_overflow_scanlines: u32,

    // Frame tracking
    frame_index: u64,
}

const OPS_CAP: u32 = 200_000;

impl Pdu {
    pub fn new() -> Self {
        Self {
            ops_used: 0,
            dma_commands_used: 0,
            dma_vram_bytes_used: 0,
            dma_audio_bytes_used: 0,
            dma_rejects: 0,
            frame_index: 0,
            cpu_rejects: 0,
            ppu_sprite_overflow: false,
            ppu_sprite_overflow_scanlines: 0,
        }
    }

    // --------------------------------------------------------------------------
    // TEMP ACCESSOR
    // Exposes frame index for debug rendering.
    // Safe to keep long-term; useful for diagnostics and overlays.
    // --------------------------------------------------------------------------
    pub fn frame_index(&self) -> u64 {
        self.frame_index
    }

    pub fn begin_frame(&mut self) {
        self.ops_used = 0;
        self.dma_commands_used = 0;
        self.dma_vram_bytes_used = 0;
        self.dma_audio_bytes_used = 0;
        self.dma_rejects = 0;
        self.cpu_rejects = 0;
        self.ppu_sprite_overflow = false;
        self.ppu_sprite_overflow_scanlines = 0;
    }

    pub fn consume(&mut self, ops: u32) -> bool {
        if self.ops_used + ops > OPS_CAP {
            self.cpu_rejects += 1;
            return false;
        }
        self.ops_used += ops;
        true
    }

    /// Called by Aurex core at end of frame to import DMA stats.
    /// Keeps DMA and PDU decoupled.
    pub fn ingest_dma(&mut self, commands: u32, vram_bytes: u32, audio_bytes: u32, rejects: u32) {
        self.dma_commands_used = commands;
        self.dma_vram_bytes_used = vram_bytes;
        self.dma_audio_bytes_used = audio_bytes;
        self.dma_rejects = rejects;
    }

    pub fn end_frame(&mut self) {
        self.frame_index += 1;

        // TEMP DEBUG
        if self.cpu_rejects > 0 {
            println!("CPU budget exceeded {} times this frame", self.cpu_rejects);
        }
    }

    pub fn ingest_ppu(&mut self, sprite_overflow: bool, overflow_scanlines: u32) {
        self.ppu_sprite_overflow = sprite_overflow;
        self.ppu_sprite_overflow_scanlines = overflow_scanlines;
    }
}
