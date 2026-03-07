use crate::aurex::DmaController;
use crate::aurex::dma::command::{DmaCommand, VramRegion};
use crate::aurex::ppu::framebuffer::FB_W;
use crate::aurex::ppu::ppu::{
    PPU_BG0_ENABLE, PPU_BG0_SCROLL_X, PPU_BG0_SCROLL_Y, PPU_BG1_ENABLE, Ppu,
};
use crate::aurex::ppu::vram::Vram;
use crate::aurex::wram::Wram;

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

            // 9 glyphs * 4 tiles each = 36 tiles
            const GLYPHS: usize = 10;
            const TILES_PER_GLYPH: usize = 4;
            const TILE_BYTES: usize = 32;
            const TOTAL_TILES: usize = GLYPHS * TILES_PER_GLYPH;
            const TOTAL_BYTES: usize = TOTAL_TILES * TILE_BYTES;

            fn set_px(buf: &mut [u8; 16 * 16], x: i32, y: i32, v: u8) {
                if x < 0 || y < 0 || x >= 16 || y >= 16 {
                    return;
                }
                buf[(y as usize) * 16 + (x as usize)] = v;
            }

            // Simple stroke helpers (deterministic, integer)
            fn rect(buf: &mut [u8; 16 * 16], x0: i32, y0: i32, x1: i32, y1: i32, v: u8) {
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        set_px(buf, x, y, v);
                    }
                }
            }

            fn diag(buf: &mut [u8; 16 * 16], x0: i32, y0: i32, x1: i32, y1: i32, v: u8) {
                // Bresenham (small, deterministic)
                let mut x = x0;
                let mut y = y0;
                let dx = (x1 - x0).abs();
                let sx = if x0 < x1 { 1 } else { -1 };
                let dy = -(y1 - y0).abs();
                let sy = if y0 < y1 { 1 } else { -1 };
                let mut err = dx + dy;
                loop {
                    set_px(buf, x, y, v);
                    if x == x1 && y == y1 {
                        break;
                    }
                    let e2 = 2 * err;
                    if e2 >= dy {
                        err += dy;
                        x += sx;
                    }
                    if e2 <= dx {
                        err += dx;
                        y += sy;
                    }
                }
            }

            fn pack_16x16_to_tiles(dst: &mut [u8], glyph_index: usize, pix: &[u8; 16 * 16]) {
                // dst contains TOTAL_TILES * 32 bytes
                // Layout for each glyph: 4 tiles (TL,TR,BL,BR), each 8x8, nibble-packed
                let base_tile = glyph_index * 4;

                for ty in 0..2 {
                    for tx in 0..2 {
                        let tile = base_tile + (ty * 2 + tx);
                        let out_base = tile * 32;

                        for row in 0..8 {
                            let src_y = ty * 8 + row;
                            let out_row = out_base + row * 4;

                            // 8 pixels => 4 bytes, high nibble then low nibble
                            for pair in 0..4 {
                                let src_x0 = tx * 8 + pair * 2;
                                let p0 = pix[src_y * 16 + src_x0] & 0x0F;
                                let p1 = pix[src_y * 16 + (src_x0 + 1)] & 0x0F;
                                dst[out_row + pair] = (p0 << 4) | p1;
                            }
                        }
                    }
                }
            }

            // Build all glyphs into a single VRAM sprite-tile blob
            let mut tiles = [0u8; TOTAL_BYTES];

            // Glyph order: A U R E X - 1 6 + +
            for gi in 0..GLYPHS {
                let mut pix = [0u8; 16 * 16];

                // -------------------------------------------------------------
                // Style (clean, bold, readable)
                // -------------------------------------------------------------
                let fg = 1u8;
                let t: i32 = 3; // stroke thickness (3 looks much better at 16x16)

                // Canvas helpers:
                // - rect(buf, x0, y0, x1, y1, v)
                // - diag(buf, x0, y0, x1, y1, v)
                //
                // We draw within a "safe box" to avoid edge clipping:
                // x: 2..13, y: 2..14, baseline at y=15

                match gi {
                    // ---------------------------------------------------------
                    // A
                    // ---------------------------------------------------------
                    0 => {
                        // legs
                        rect(&mut pix, 2, 4, 2 + (t - 1), 14, fg);
                        rect(&mut pix, 13 - (t - 1), 4, 13, 14, fg);

                        // top cap
                        rect(&mut pix, 2, 4, 13, 4 + (t - 1), fg);

                        // cross bar (slightly above center)
                        rect(&mut pix, 2, 9, 13, 9 + (t - 1), fg);
                    }

                    // ---------------------------------------------------------
                    // U
                    // ---------------------------------------------------------
                    1 => {
                        rect(&mut pix, 2, 4, 2 + (t - 1), 13, fg);
                        rect(&mut pix, 13 - (t - 1), 4, 13, 13, fg);
                        rect(&mut pix, 2, 13 - (t - 1), 13, 13, fg);
                    }

                    // ---------------------------------------------------------
                    // R
                    // ---------------------------------------------------------
                    2 => {
                        // spine
                        rect(&mut pix, 2, 4, 2 + (t - 1), 14, fg);

                        // top bar
                        rect(&mut pix, 2, 4, 13, 4 + (t - 1), fg);

                        // bowl right
                        rect(&mut pix, 13 - (t - 1), 4, 13, 10, fg);

                        // mid bar
                        rect(&mut pix, 2, 10 - (t - 1), 13, 10, fg);

                        // leg (clean blocky leg)
                        rect(&mut pix, 9, 11, 13, 14, fg);
                    }

                    // ---------------------------------------------------------
                    // E
                    // ---------------------------------------------------------
                    3 => {
                        rect(&mut pix, 2, 4, 2 + (t - 1), 14, fg); // spine
                        rect(&mut pix, 2, 4, 13, 4 + (t - 1), fg); // top
                        rect(&mut pix, 2, 9, 11, 9 + (t - 1), fg); // mid
                        rect(&mut pix, 2, 14 - (t - 1), 13, 14, fg); // bottom
                    }

                    // ---------------------------------------------------------
                    // X (double diagonals for thickness)
                    // ---------------------------------------------------------
                    4 => {
                        diag(&mut pix, 2, 4, 13, 14, fg);
                        diag(&mut pix, 3, 4, 13, 13, fg);
                        diag(&mut pix, 13, 4, 2, 14, fg);
                        diag(&mut pix, 12, 4, 2, 13, fg);
                    }

                    // ---------------------------------------------------------
                    // - (dash)
                    // ---------------------------------------------------------
                    5 => {
                        rect(&mut pix, 3, 10, 12, 10 + (t - 1), fg);
                    }

                    // ---------------------------------------------------------
                    // 1
                    // ---------------------------------------------------------
                    6 => {
                        rect(&mut pix, 8, 4, 8 + (t - 1), 14, fg); // stem
                        rect(&mut pix, 6, 14 - (t - 1), 12, 14, fg); // base
                        rect(&mut pix, 6, 5, 8 + (t - 1), 4 + (t - 1), fg); // tiny cap
                    }

                    // ---------------------------------------------------------
                    // 6 (clean loop)
                    // ---------------------------------------------------------
                    7 => {
                        rect(&mut pix, 3, 5, 3 + (t - 1), 14, fg); // left
                        rect(&mut pix, 3, 5, 12, 5 + (t - 1), fg); // top
                        rect(&mut pix, 3, 10, 12, 10 + (t - 1), fg); // mid
                        rect(&mut pix, 12 - (t - 1), 10, 12, 14, fg); // right lower
                        rect(&mut pix, 3, 14 - (t - 1), 12, 14, fg); // bottom
                    }

                    // ---------------------------------------------------------
                    // + (both plus glyphs)
                    // ---------------------------------------------------------
                    8 | 9 => {
                        rect(&mut pix, 8, 6, 8 + (t - 1), 13, fg); // vertical
                        rect(&mut pix, 5, 10, 13, 10 + (t - 1), fg); // horizontal
                    }

                    _ => {}
                }

                // -------------------------------------------------------------
                // Baseline anchor (makes glyphs feel “not cut off”)
                // -------------------------------------------------------------
                rect(&mut pix, 2, 15, 13, 15, 0); // keep baseline clean (no forced line)

                // Pack into 4 tiles (TL,TR,BL,BR)
                pack_16x16_to_tiles(&mut tiles[..], gi, &pix);
            }

            // Copy tiles into WRAM staging
            wram.memory_mut()[base_wram_offset..base_wram_offset + TOTAL_BYTES]
                .copy_from_slice(&tiles);

            // DMA to SpriteTiles VRAM
            let cmd = DmaCommand::new(
                VramRegion::SpriteTiles,
                base_wram_offset,
                0, // tile 0
                TOTAL_BYTES,
            );
            dma.request(cmd, wram, vram);

            // -------------------------------------------------------------
            // PALETTE INITIALIZATION (Palette 0)
            // -------------------------------------------------------------

            let palette_wram_offset = 0x0800; // keep separate from tile staging (TOTAL_BYTES can exceed 0x0100)

            let palette_data: [u8; 64] = [
                // ---------------------------
                // Palette 0 (logo - silver)
                // ---------------------------
                0x00, 0x00, // 0: transparent
                0xFF, 0x7F, // 1: white
                0xAD, 0x56, // 2: mid silver
                0x29, 0x21, // 3: dark silver
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                // ---------------------------
                // Palette 1 (glow - cyan)
                // ---------------------------
                0x00, 0x00, // 0: transparent
                0xFF, 0x03, // 1: bright cyan
                0x9F, 0x02, // 2: mid cyan
                0x5F, 0x01, // 3: deep cyan
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];

            wram.memory_mut()[palette_wram_offset..palette_wram_offset + 64]
                .copy_from_slice(&palette_data);

            let palette_cmd = DmaCommand::new(
                VramRegion::Palettes,
                palette_wram_offset,
                0, // start of palette memory
                64,
            );

            dma.request(palette_cmd, wram, vram);
        }

        let frame = self.frame;

        // -------------------------------------------------------------
        // Disable background layers for clean boot screen
        // -------------------------------------------------------------
        ppu.write_addr(PPU_BG0_ENABLE, 0);
        ppu.write_addr(PPU_BG1_ENABLE, 0);

        // -------------------------------------------------------------
        // Background Scroll (slow cosmic drift)
        // -------------------------------------------------------------
        let scroll = (frame / 4) as u16;
        ppu.write_addr(PPU_BG0_SCROLL_X, scroll);
        ppu.write_addr(PPU_BG0_SCROLL_Y, 0);

        // -------------------------------------------------------------
        // Logo Drop Logic
        // -------------------------------------------------------------
        let base_y: i16 = 80;
        let start_y: i16 = -40;
        let spacing: i16 = 20;
        let center_x: i16 = (FB_W as i16) / 2;

        let letters = 10; // A U R E X - 1 6 + +

        for i in 0..letters {
            let appear_frame = 60 + (i as u32 * 8);

            let total_width = letters as i16 * spacing;
            let start_x = center_x - (total_width / 2);

            let x = start_x + (i as i16 * spacing);

            let y = if frame < appear_frame {
                start_y
            } else {
                let t = (frame - appear_frame).min(40);
                let p = t as i32;
                let dur = 40i32;

                // Integer smoothstep in fixed-point [0, 1024].
                // s = p^2 * (3d - 2p) / d^3
                let num = p * p * (3 * dur - 2 * p) * 1024;
                let den = dur * dur * dur;
                let eased_fp = num / den;

                let dy = (base_y - start_y) as i32;
                (start_y as i32 + (dy * eased_fp) / 1024) as i16
            };

            let tile_index = (i * 4) as u16;

            ppu.write_sprite(
                i as usize, // glow uses low indices
                (x + 1) as u16,
                y as u16,
                tile_index,
                1,
                0,
                true,
                false,
                false,
            );

            // Main pass (front)
            ppu.write_sprite(
                32 + i as usize, // main uses higher indices
                x as u16,
                y as u16,
                tile_index,
                0,
                0,
                true,
                false,
                false,
            );
        }

        // -------------------------------------------------------------
        // Cinematic Drop Spike
        // -------------------------------------------------------------
        if frame == 360 {
            let spike = scroll.wrapping_mul(4);
            ppu.write_addr(PPU_BG0_SCROLL_X, spike);
            ppu.write_addr(PPU_BG0_SCROLL_Y, 0);
        }

        self.frame = self.frame.wrapping_add(1);
    }
}
