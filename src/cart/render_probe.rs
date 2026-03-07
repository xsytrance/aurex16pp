use crate::aurex::{
    DmaController,
    dma::{DmaCommand, VramRegion},
    ppu::ppu::Ppu,
    wram::Wram,
};

pub struct RenderProbe {
    initialized: bool,
}

impl RenderProbe {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    pub fn update(&mut self, ppu: &mut Ppu, dma: &mut DmaController, wram: &mut Wram) {
        if !self.initialized {
            self.initialized = true;

            // -------------------------------------------------
            // Build single solid 16x16 white square
            // -------------------------------------------------

            let mut tile_bytes = [0u8; 32 * 4]; // 4 tiles (16x16)

            // Fill all pixels with color index 1
            for t in 0..4 {
                let base = t * 32;
                for row in 0..8 {
                    let row_base = base + row * 4;
                    for b in 0..4 {
                        tile_bytes[row_base + b] = 0x11;
                    }
                }
            }

            // Copy into WRAM
            wram.memory_mut()[0..128].copy_from_slice(&tile_bytes);

            // DMA to SpriteTiles
            let cmd = DmaCommand::new(VramRegion::SpriteTiles, 0, 0, 128);
            dma.request(cmd, wram, &ppu.vram());

            // -------------------------------------------------
            // Upload palette (white)
            // -------------------------------------------------

            let palette: [u8; 4] = [
                0x00, 0x00, // index 0 transparent
                0xFF, 0x7F, // index 1 white (RGB555 0x7FFF)
            ];

            wram.memory_mut()[256..260].copy_from_slice(&palette);

            let pal_cmd = DmaCommand::new(VramRegion::Palettes, 256, 0, 4);

            dma.request(pal_cmd, wram, &ppu.vram());
        }

        // -------------------------------------------------
        // Draw sprite at center
        // -------------------------------------------------

        ppu.write_sprite(0, 120, 80, 0, 0, 0, true, false, false);
    }
}
