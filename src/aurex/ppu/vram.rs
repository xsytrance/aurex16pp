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
const TILEMAP_BYTES: usize = 128 * 1024;
const SPRITE_TILES_BYTES: usize = 384 * 1024;
const MODE7_TEX_BYTES: usize = 64 * 1024;
const PALETTE_BYTES: usize = 16 * 1024;
const RESERVED_BYTES: usize = 64 * 1024;

const VRAM_TOTAL_BYTES: usize = BG_TILES_BYTES
    + TILEMAP_BYTES
    + SPRITE_TILES_BYTES
    + MODE7_TEX_BYTES
    + PALETTE_BYTES
    + RESERVED_BYTES;

pub struct Vram {
    pub bg_tiles: Box<[u8]>,
    pub tilemaps: Box<[u8]>,
    pub sprite_tiles: Box<[u8]>,
    pub mode7_tex: Box<[u8]>,
    pub palettes: Box<[u8]>,
    pub reserved: Box<[u8]>,
}

impl Vram {
    pub fn new() -> Self {
        let v = Self {
            bg_tiles: vec![0u8; BG_TILES_BYTES].into_boxed_slice(),
            tilemaps: vec![0u8; TILEMAP_BYTES].into_boxed_slice(),
            sprite_tiles: vec![0u8; SPRITE_TILES_BYTES].into_boxed_slice(),
            mode7_tex: vec![0u8; MODE7_TEX_BYTES].into_boxed_slice(),
            palettes: vec![0u8; PALETTE_BYTES].into_boxed_slice(),
            reserved: vec![0u8; RESERVED_BYTES].into_boxed_slice(),
        };

        debug_assert_eq!(v.bg_tiles.len(), BG_TILES_BYTES);
        debug_assert_eq!(v.tilemaps.len(), TILEMAP_BYTES);
        debug_assert_eq!(v.sprite_tiles.len(), SPRITE_TILES_BYTES);
        debug_assert_eq!(v.mode7_tex.len(), MODE7_TEX_BYTES);
        debug_assert_eq!(v.palettes.len(), PALETTE_BYTES);
        debug_assert_eq!(v.reserved.len(), RESERVED_BYTES);
        debug_assert_eq!(v.total_bytes(), VRAM_TOTAL_BYTES);

        v
    }

    pub fn total_bytes(&self) -> usize {
        self.bg_tiles.len()
            + self.tilemaps.len()
            + self.sprite_tiles.len()
            + self.mode7_tex.len()
            + self.palettes.len()
            + self.reserved.len()
    }
    pub fn region_len(&self, region: &VramRegion) -> usize {
        match region {
            VramRegion::BgTiles => self.bg_tiles.len(),
            VramRegion::Tilemaps => self.tilemaps.len(),
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
            VramRegion::Tilemaps => &mut self.tilemaps,
            VramRegion::SpriteTiles => &mut self.sprite_tiles,
            VramRegion::Mode7Tex => &mut self.mode7_tex,
            VramRegion::Palettes => &mut self.palettes,
            VramRegion::Reserved => &mut self.reserved,

            // Audio RAM does not belong to PPU
            VramRegion::AudioRam => panic!("AudioRam accessed through PPU VRAM"),
        }
    }
}
