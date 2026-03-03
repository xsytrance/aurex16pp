// ============================================================================
// PPU-A16 VRAM (Phase 3)
// ----------------------------------------------------------------------------
// VRAM is modeled as separate partitions (Option B).
//
// IMPORTANT:
// - We allocate via Vec -> Box<[u8]> to avoid stack overflows on Windows.
// - Sizes are enforced by constants + debug assertions.
// - No rendering yet. Memory only.
// ============================================================================
use crate::aurex::dma::command::VramRegion;

const BG_TILES_BYTES: usize = 384 * 1024;
// 64x64 entries, 2 bytes per entry
const TILEMAP_BYTES: usize = 64 * 64 * 2;
const SPRITE_TILES_BYTES: usize = 384 * 1024;
const MODE7_TEX_BYTES: usize = 64 * 1024;
const PALETTE_BYTES: usize = 16 * 1024;
const RESERVED_BYTES: usize = 64 * 1024;

const VRAM_TOTAL_BYTES: usize = BG_TILES_BYTES
    + TILEMAP_BYTES
    + TILEMAP_BYTES
    + SPRITE_TILES_BYTES
    + MODE7_TEX_BYTES
    + PALETTE_BYTES
    + RESERVED_BYTES;

// ============================================================================
// AUREX-16++ VRAM MEMORY MAP (LOCKED)
// ----------------------------------------------------------------------------
// Total VRAM: 1 MB (0x100000 bytes)
// Canonical hardware partition — DO NOT RE-ARCHITECT
// All regions are inclusive ranges.
// Alignment: 0x4000 (16 KB)
// ============================================================================

pub const VRAM_SIZE: usize = 0x100000; // 1,048,576 bytes

// -----------------------------------------------------------------------------
// Region A — General Tile Pattern Memory (BG0/BG1/BG3)
// -----------------------------------------------------------------------------
pub const VRAM_A_BASE: usize = 0x00000;
pub const VRAM_A_END: usize = 0x4FFFF;

// -----------------------------------------------------------------------------
// Region B — BG Tilemaps (BG0/BG1/BG3)
// -----------------------------------------------------------------------------
pub const VRAM_B_BASE: usize = 0x50000;
pub const VRAM_B_END: usize = 0x5FFFF;

// -----------------------------------------------------------------------------
// Region C — Sprite Pattern Memory
// -----------------------------------------------------------------------------
pub const VRAM_C_BASE: usize = 0x60000;
pub const VRAM_C_END: usize = 0x8FFFF;

// -----------------------------------------------------------------------------
// Region D — Sprite Tables
// -----------------------------------------------------------------------------
pub const VRAM_D_BASE: usize = 0x90000;
pub const VRAM_D_END: usize = 0x93FFF;

// -----------------------------------------------------------------------------
// Region E — Mode 7 Map (BG2 ONLY)
// -----------------------------------------------------------------------------
pub const VRAM_E_BASE: usize = 0x94000;
pub const VRAM_E_END: usize = 0xA3FFF;

// -----------------------------------------------------------------------------
// Region F — Mode 7 Texture / Pattern Store (BG2 ONLY)
// -----------------------------------------------------------------------------
pub const VRAM_F_BASE: usize = 0xA4000;
pub const VRAM_F_END: usize = 0xD3FFF;

// -----------------------------------------------------------------------------
// Region G — Line Tables (Scanline Effects)
// -----------------------------------------------------------------------------
pub const VRAM_G_BASE: usize = 0xD4000;
pub const VRAM_G_END: usize = 0xDBFFF;

// -----------------------------------------------------------------------------
// Region H — Cartridge General-Purpose VRAM
// -----------------------------------------------------------------------------
pub const VRAM_H_BASE: usize = 0xDC000;
pub const VRAM_H_END: usize = 0xFBFFF;

// -----------------------------------------------------------------------------
// Region I — RESERVED (DO NOT USE)
// -----------------------------------------------------------------------------
pub const VRAM_I_BASE: usize = 0xFC000;
pub const VRAM_I_END: usize = 0xFFFFF;

// ============================================================================
// VRAM Region Classification
// ============================================================================

pub fn classify_region(addr: usize) -> Option<VramRegion> {
    match addr {
        VRAM_A_BASE..=VRAM_A_END => Some(VramRegion::BgTiles),
        VRAM_B_BASE..=VRAM_B_END => {
            let offset = addr - VRAM_B_BASE;

            if offset < (64 * 64 * 2) {
                Some(VramRegion::Bg0Tilemap)
            } else {
                Some(VramRegion::Bg1Tilemap)
            }
        }
        VRAM_C_BASE..=VRAM_C_END => Some(VramRegion::SpriteTiles),
        VRAM_D_BASE..=VRAM_D_END => Some(VramRegion::SpriteTiles),
        VRAM_E_BASE..=VRAM_E_END => Some(VramRegion::Mode7Tex),
        VRAM_F_BASE..=VRAM_F_END => Some(VramRegion::Mode7Tex),
        VRAM_G_BASE..=VRAM_G_END => Some(VramRegion::BgTiles),
        VRAM_H_BASE..=VRAM_H_END => Some(VramRegion::BgTiles),
        VRAM_I_BASE..=VRAM_I_END => None, // RESERVED
        _ => None,
    }
}

pub fn vram_write_allowed(addr: usize) -> bool {
    classify_region(addr).is_some()
}

pub struct Vram {
    // -----------------------------------------------------------------------------
    // BG pattern tiles (shared by BG0 + BG1)
    // 4bpp, 32 bytes per tile
    // -----------------------------------------------------------------------------
    pub bg_tiles: Box<[u8]>,

    // -----------------------------------------------------------------------------
    // BG0 tilemap (64x64, 2 bytes per entry)
    // -----------------------------------------------------------------------------
    pub bg0_tilemap: Vec<u8>,

    // -----------------------------------------------------------------------------
    // BG1 tilemap (64x64, 2 bytes per entry)
    // -----------------------------------------------------------------------------
    pub bg1_tilemap: Vec<u8>,
    pub sprite_tiles: Box<[u8]>,
    pub mode7_tex: Box<[u8]>,
    pub palettes: Box<[u8]>,
    pub reserved: Box<[u8]>,
}

impl Vram {
    pub fn new() -> Self {
        let v = Self {
            bg_tiles: vec![0u8; BG_TILES_BYTES].into_boxed_slice(),
            bg0_tilemap: vec![0; 64 * 64 * 2],
            bg1_tilemap: vec![0; 64 * 64 * 2],
            sprite_tiles: vec![0u8; SPRITE_TILES_BYTES].into_boxed_slice(),
            mode7_tex: vec![0u8; MODE7_TEX_BYTES].into_boxed_slice(),
            palettes: vec![0u8; PALETTE_BYTES].into_boxed_slice(),
            reserved: vec![0u8; RESERVED_BYTES].into_boxed_slice(),
        };

        debug_assert_eq!(v.bg_tiles.len(), BG_TILES_BYTES);
        debug_assert_eq!(v.bg0_tilemap.len(), TILEMAP_BYTES);
        debug_assert_eq!(v.bg1_tilemap.len(), TILEMAP_BYTES);
        debug_assert_eq!(v.sprite_tiles.len(), SPRITE_TILES_BYTES);
        debug_assert_eq!(v.mode7_tex.len(), MODE7_TEX_BYTES);
        debug_assert_eq!(v.palettes.len(), PALETTE_BYTES);
        debug_assert_eq!(v.reserved.len(), RESERVED_BYTES);
        debug_assert_eq!(v.total_bytes(), VRAM_TOTAL_BYTES);

        v
    }

    pub fn total_bytes(&self) -> usize {
        // -----------------------------------------------------------------------------
        // Total VRAM byte count
        // -----------------------------------------------------------------------------
        self.bg_tiles.len()
            + self.bg0_tilemap.len()
            + self.bg1_tilemap.len()
            + self.sprite_tiles.len()
            + self.mode7_tex.len()
            + self.palettes.len()
            + self.reserved.len()
    }

    pub fn region_base(&self, region: &VramRegion) -> usize {
        match region {
            VramRegion::BgTiles => VRAM_A_BASE,
            VramRegion::Bg0Tilemap => VRAM_B_BASE,
            VramRegion::Bg1Tilemap => VRAM_B_BASE + (64 * 64 * 2),
            VramRegion::SpriteTiles => VRAM_C_BASE,
            VramRegion::Mode7Tex => VRAM_E_BASE,
            VramRegion::Palettes => VRAM_H_BASE,
            VramRegion::AudioRam => panic!("AudioRam accessed through PPU VRAM"),
            VramRegion::Reserved => panic!("Reserved VRAM region cannot be accessed"),
        }
    }

    pub fn region_end_abs(&self, region: &VramRegion) -> usize {
        let base = self.region_base(region);
        let len = self.region_len(region);
        base + len.saturating_sub(1)
    }

    pub fn region_bounds_abs(&self, region: &VramRegion) -> (usize, usize) {
        (self.region_base(region), self.region_end_abs(region))
    }

    pub fn region_len(&self, region: &VramRegion) -> usize {
        match region {
            VramRegion::BgTiles => self.bg_tiles.len(),
            VramRegion::Bg0Tilemap => self.bg0_tilemap.len(),
            VramRegion::Bg1Tilemap => self.bg1_tilemap.len(),
            VramRegion::SpriteTiles => self.sprite_tiles.len(),
            VramRegion::Mode7Tex => self.mode7_tex.len(),
            VramRegion::Palettes => self.palettes.len(),
            VramRegion::Reserved => self.reserved.len(),

            // Audio RAM does not belong to PPU
            VramRegion::AudioRam => panic!("AudioRam accessed through PPU VRAM"),
        }
    }

    pub fn region_mut(&mut self, region: &VramRegion) -> &mut [u8] {
        match region {
            VramRegion::BgTiles => &mut self.bg_tiles,
            VramRegion::Bg0Tilemap => &mut self.bg0_tilemap,
            VramRegion::Bg1Tilemap => &mut self.bg1_tilemap,
            VramRegion::SpriteTiles => &mut self.sprite_tiles,
            VramRegion::Mode7Tex => &mut self.mode7_tex,
            VramRegion::Palettes => &mut self.palettes,
            VramRegion::Reserved => &mut self.reserved,

            // Audio RAM does not belong to PPU
            VramRegion::AudioRam => panic!("AudioRam accessed through PPU VRAM"),
        }
    }
}
