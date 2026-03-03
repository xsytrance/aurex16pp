use crate::aurex::DmaController;
use crate::aurex::dma::command::{DmaCommand, VramRegion};
use crate::aurex::ppu::ppu::Ppu;
use crate::aurex::ppu::vram::Vram;
use crate::aurex::wram::Wram;

// -------------------------------------------------------------
// PRIME IGNITION SPRITE TILE DATA
// 4bpp planar, 32 bytes per tile
// Using palette index 1 only
// -------------------------------------------------------------

const TILE_A_0: [u8; 32] = [
    0b00011000, 0, 0, 0, 0b00111100, 0, 0, 0, 0b01100110, 0, 0, 0, 0b01100110, 0, 0, 0, 0b01111110,
    0, 0, 0, 0b01100110, 0, 0, 0, 0b01100110, 0, 0, 0, 0b00000000, 0, 0, 0,
];

const TILE_A_1: [u8; 32] = TILE_A_0;
const TILE_A_2: [u8; 32] = TILE_A_0;
const TILE_A_3: [u8; 32] = TILE_A_0;

pub struct PrimeIgnition {
    frame: u32,
}

impl PrimeIgnition {
    pub fn new() -> Self {
        Self { frame: 0 }
    }

    pub fn update(&mut self, ppu: &mut Ppu, dma: &mut DmaController, wram: &mut Wram, vram: &Vram) {
        if self.frame == 0 {
            let base_wram_offset = 0x0000;

            let tile_data: [u8; 128] = [
                // TILE 0
                0x00, 0x01, 0x10, 0x00, 0x00, 0x11, 0x11, 0x00, 0x01, 0x10, 0x01, 0x10, 0x01, 0x10,
                0x01, 0x10, 0x01, 0x11, 0x11, 0x10, 0x01, 0x10, 0x01, 0x10, 0x01, 0x10, 0x01, 0x10,
                0x00, 0x00, 0x00, 0x00, // TILE 1
                0x00, 0x01, 0x10, 0x00, 0x00, 0x11, 0x11, 0x00, 0x01, 0x10, 0x01, 0x10, 0x01, 0x10,
                0x01, 0x10, 0x01, 0x11, 0x11, 0x10, 0x01, 0x10, 0x01, 0x10, 0x01, 0x10, 0x01, 0x10,
                0x00, 0x00, 0x00, 0x00, // TILE 2
                0x00, 0x01, 0x10, 0x00, 0x00, 0x11, 0x11, 0x00, 0x01, 0x10, 0x01, 0x10, 0x01, 0x10,
                0x01, 0x10, 0x01, 0x11, 0x11, 0x10, 0x01, 0x10, 0x01, 0x10, 0x01, 0x10, 0x01, 0x10,
                0x00, 0x00, 0x00, 0x00, // TILE 3
                0x00, 0x01, 0x10, 0x00, 0x00, 0x11, 0x11, 0x00, 0x01, 0x10, 0x01, 0x10, 0x01, 0x10,
                0x01, 0x10, 0x01, 0x11, 0x11, 0x10, 0x01, 0x10, 0x01, 0x10, 0x01, 0x10, 0x01, 0x10,
                0x00, 0x00, 0x00, 0x00,
            ];

            // Copy to WRAM
            wram.memory_mut()[base_wram_offset..base_wram_offset + 128].copy_from_slice(&tile_data);

            let cmd = DmaCommand::new(VramRegion::SpriteTiles, base_wram_offset, 0, 128);

            dma.request(cmd, wram, vram);
        }
        let frame = self.frame;

        // -------------------------------------------------------------
        // Background Scroll (slow cosmic drift)
        // -------------------------------------------------------------
        let scroll = (frame / 4) as u16;
        ppu.set_bg0_scroll(scroll, 0);

        // -------------------------------------------------------------
        // Logo Drop Logic
        // -------------------------------------------------------------
        let base_y: i16 = 80;
        let start_y: i16 = -40;
        let spacing: i16 = 20;
        let center_x: i16 = 120;

        let letters = 9; // A U R E X - 1 6 + +

        for i in 0..letters {
            let appear_frame = 220 + (i as u32 * 10);

            let x = center_x + ((i as i16 - 4) * spacing);

            let y = if frame < appear_frame {
                start_y
            } else {
                let t = (frame - appear_frame) as i16;

                if t < 20 {
                    start_y + (t * 6)
                } else if t < 30 {
                    base_y + ((30 - t) * 2)
                } else {
                    base_y
                }
            };

            let tile_index = (i * 4) as u16;

            ppu.write_sprite(
                i as usize, x as u16, y as u16, tile_index, 0,     // palette
                0,     // priority
                true,  // 16x16
                false, // hflip
                false, // vflip
            );
        }

        // -------------------------------------------------------------
        // Cinematic Drop Spike
        // -------------------------------------------------------------
        if frame == 360 {
            let spike = scroll.wrapping_mul(4);
            ppu.set_bg0_scroll(spike, 0);
        }

        self.frame = self.frame.wrapping_add(1);
    }
}
