use crate::aurex::DmaController;
use crate::aurex::dma::command::{DmaCommand, VramRegion};
use crate::aurex::ppu::ppu::Ppu;
use crate::aurex::ppu::vram::Vram;
use crate::aurex::wram::Wram;

// ============================================================================
// RenderProbe (diagnostic cart / boot substitute)
// Goal: prove WRAM -> DMA -> VRAM -> PPU sprite sampling works.
// - Upload a known 16x16 sprite (4 tiles) to SpriteTiles at tile 0
// - Upload a tiny palette: index0 transparent, index1 white
// - Draw one sprite at center
// NOTE: This WILL NOT look correct if DMA apply is VBlank-gated and vblank=false.
// ============================================================================
pub struct RenderProbe {
    initialized: bool,
}

impl RenderProbe {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    pub fn update(
        &mut self,
        ppu: &mut Ppu,
        dma: &mut DmaController,
        wram: &mut Wram,
        _vram: &Vram,
    ) {
        if !self.initialized {
            self.initialized = true;

            // -------------------------------------------------------------
            // Build a solid 16x16 block using color index 1
            // Each 8x8 tile is 32 bytes (4bpp nibble packed: 2 px per byte).
            // 0x11 means: hi nibble = 1, lo nibble = 1 (two pixels both 1).
            // -------------------------------------------------------------
            let mut tiles = [0u8; 128]; // 4 tiles × 32 bytes

            for tile in 0..4 {
                let base = tile * 32;
                for row in 0..8 {
                    let row_base = base + row * 4;
                    tiles[row_base + 0] = 0x11;
                    tiles[row_base + 1] = 0x11;
                    tiles[row_base + 2] = 0x11;
                    tiles[row_base + 3] = 0x11;
                }
            }

            // Stage tiles in WRAM at offset 0x0000
            let tile_wram = 0x0000usize;
            wram.memory_mut()[tile_wram..tile_wram + 128].copy_from_slice(&tiles);

            // DMA -> SpriteTiles at dst_offset 0 (tile 0)
            let cmd = DmaCommand::new(VramRegion::SpriteTiles, tile_wram, 0, 128);
            dma.request(cmd, wram, _vram);

            // -------------------------------------------------------------
            // Palette: index0 transparent (0x0000), index1 white (0x7FFF)
            // -------------------------------------------------------------
            let pal = [0x00u8, 0x00u8, 0xFFu8, 0x7Fu8]; // 4 bytes = 2 colors

            let pal_wram = 0x0100usize;
            wram.memory_mut()[pal_wram..pal_wram + 4].copy_from_slice(&pal);

            // Write into palette base (palette 0, indices 0..1)
            let pal_cmd = DmaCommand::new(VramRegion::Palettes, pal_wram, 0, 4);
            dma.request(pal_cmd, wram, _vram);
        }

        // Draw one 16x16 sprite using tile 0, palette 0, priority 0
        ppu.write_sprite(
            0,     // sprite index
            120,   // x
            80,    // y
            0,     // tile index
            0,     // palette
            0,     // priority
            true,  // 16x16
            false, // hflip
            false, // vflip
        );
    }
}
