#![allow(dead_code)]
// ============================================================================
// DMA Command Definition (Phase 3.6)
// ----------------------------------------------------------------------------
// Represents a hardware DMA transfer request.
//
// IMPORTANT:
// - DMA copies from WRAM to a specific VRAM partition.
// - All bounds must be validated at request time.
// - Invalid transfers are rejected immediately (hardware discipline).
//
// Likely-to-change areas:
// - May later include transfer flags (e.g., fill, increment modes)
// - May evolve to include audio RAM regions
// ============================================================================

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VramRegion {
    BgTiles,
    Bg0Tilemap,
    Bg1Tilemap,
    SpriteTiles,
    Mode7Tex,
    Palettes,

    // ASU-816 Sample RAM
    AudioRam,

    Reserved,
}

#[derive(Clone, Debug)]
pub struct DmaCommand {
    pub region: VramRegion,
    pub src_offset: usize, // offset in WRAM
    pub dst_offset: usize, // offset in VRAM partition
    pub bytes: usize,
}

impl DmaCommand {
    pub fn new(region: VramRegion, src_offset: usize, dst_offset: usize, bytes: usize) -> Self {
        Self {
            region,
            src_offset,
            dst_offset,
            bytes,
        }
    }

    pub fn is_audio(&self) -> bool {
        matches!(self.region, VramRegion::AudioRam)
    }
}
