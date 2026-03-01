// ============================================================================
// PPU-A16 VRAM (Phase 3)
// ----------------------------------------------------------------------------
// We model VRAM as separate fixed partitions (Option B).
//
// Why separate partitions?
// - Prevents accidental cross-region corruption (tiles vs palettes, etc.)
// - Keeps files/systems modular for AI handoffs
// - Makes caps and usage explicit
//
// NOTE: No rendering exists yet. This is memory only.
// NOTE: DMA does not copy into VRAM yet (that integration comes next).
//
// Likely-to-change areas:
// - How tile formats are represented
// - Whether tilemaps become typed structs instead of raw bytes
// - Palette format (currently raw bytes; later will be 5:5:5 words)
// ============================================================================

pub struct Vram {
    /// 384 KB - Background tile graphics (raw bytes for now)
    pub bg_tiles: Box<[u8; 384 * 1024]>,

    /// 128 KB - Tilemaps (layout indices, attributes; raw bytes for now)
    pub tilemaps: Box<[u8; 128 * 1024]>,

    /// 384 KB - Sprite tile graphics (raw bytes for now)
    pub sprite_tiles: Box<[u8; 384 * 1024]>,

    /// 64 KB - Mode 7 texture memory (raw bytes for now)
    pub mode7_tex: Box<[u8; 64 * 1024]>,

    /// 16 KB - Palettes (raw bytes for now; later likely u16 5:5:5)
    pub palettes: Box<[u8; 16 * 1024]>,

    /// 64 KB - Reserved / system / future expansion
    pub reserved: Box<[u8; 64 * 1024]>,
}

impl Vram {
    pub fn new() -> Self {
        Self {
            bg_tiles: Box::new([0; 384 * 1024]),
            tilemaps: Box::new([0; 128 * 1024]),
            sprite_tiles: Box::new([0; 384 * 1024]),
            mode7_tex: Box::new([0; 64 * 1024]),
            palettes: Box::new([0; 16 * 1024]),
            reserved: Box::new([0; 64 * 1024]),
        }
    }

    /// Total VRAM bytes (should always be 1 MiB).
    pub fn total_bytes(&self) -> usize {
        (384 + 128 + 384 + 64 + 16 + 64) * 1024
    }
}
