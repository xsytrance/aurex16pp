// ============================================================================
// PPU-A16 (v0.1)
// ----------------------------------------------------------------------------
// Hardware-accurate render entry point.
// Scanline-based deterministic pipeline.
// No blending, no priority yet.
// ============================================================================

use super::framebuffer::{FB_H, FB_W, Framebuffer};
use super::oam::Oam;
use super::vram::Vram;

pub struct Ppu {
    frame_counter: u64,

    // BG0 scroll registers
    bg0_scroll_x: u16,
    bg0_scroll_y: u16,

    // Sprite memory
    oam: Oam,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            frame_counter: 0,
            bg0_scroll_x: 0,
            bg0_scroll_y: 0,
            oam: Oam::new(),
        }
    }

    // -------------------------------------------------------------------------
    // Sprite Scanline Evaluation
    // -------------------------------------------------------------------------
    fn evaluate_sprites_for_scanline(&self, y: usize) -> ([usize; 8], usize, bool) {
        let mut visible_indices = [0usize; 8];
        let mut count = 0;
        let mut overflow = false;

        for i in 0..self.oam.len() {
            if let Some(sprite) = self.oam.sprite(i) {
                if !sprite.visible {
                    continue;
                }

                let sprite_top = sprite.y as usize;
                let sprite_bottom = sprite_top + 8; // 8x8 for now

                if y >= sprite_top && y < sprite_bottom {
                    if count < 8 {
                        visible_indices[count] = i;
                        count += 1;
                    } else {
                        overflow = true;
                        break;
                    }
                }
            }
        }

        (visible_indices, count, overflow)
    }

    // ---------------------------------------------------------------------
    // Sprite write interface (temporary direct API)
    // ---------------------------------------------------------------------
    pub fn write_sprite(
        &mut self,
        index: usize,
        x: u16,
        y: u16,
        tile: u16,
        palette: u8,
        priority: u8,
    ) {
        if let Some(sprite) = self.oam.sprite_mut(index) {
            sprite.x = x;
            sprite.y = y;
            sprite.tile_index = tile;
            sprite.palette = palette;
            sprite.priority = priority;
            sprite.visible = true;
        }
    }

    // ---------------------------------------------------------------------
    // Register setters (to be wired to CPU later)
    // ---------------------------------------------------------------------
    pub fn set_bg0_scroll(&mut self, x: u16, y: u16) {
        self.bg0_scroll_x = x;
        self.bg0_scroll_y = y;
    }

    // -------------------------------------------------------------------------
    // FRAME ENTRY
    // -------------------------------------------------------------------------
    pub fn render_frame(&mut self, vram: &Vram, fb: &mut Framebuffer) {
        // TEMP TEST: auto-scroll BG0 horizontally
        // Removal: delete once CPU register writes exist

        for y in 0..FB_H {
            self.render_scanline(vram, y, fb);
        }

        self.frame_counter += 1;
    }

    // -------------------------------------------------------------------------
    // SCANLINE STUB
    // -------------------------------------------------------------------------
    fn render_scanline(&mut self, vram: &Vram, y: usize, fb: &mut Framebuffer) {
        let pixels = fb.pixels_mut();

        // -------------------------------------------------------------------------
        // Sprite Scanline Evaluation (Phase 1 - no rendering yet)
        // -------------------------------------------------------------------------
        let (_sprite_indices, _sprite_count, sprite_overflow) =
            self.evaluate_sprites_for_scanline(y);

        if sprite_overflow {
            // For now just debug log once per frame later
        }

        // NOTE:
        // - _sprite_indices holds up to 8 sprite indices
        // - _sprite_count is how many are active
        // - _sprite_overflow is true if >8 found
        // Rendering will be implemented in next phase.

        // ---------------------------------------------------------------------
        // BG0 Bring-up (v0.1)
        // - Tilemap source: vram.tilemaps (start)
        // - Pattern source: vram.bg_tiles (start)
        // - Palette source: vram.palettes (first 256 RGB555 entries, little-endian)
        //
        // Tile encoding (LOCKED for now):
        // - 8x8, 4bpp packed, 32 bytes per tile
        // - Each row is 4 bytes => 8 pixels
        // - Each byte stores 2 pixels: hi nibble then lo nibble (left->right)
        //
        // Tilemap entry (u16) bits:
        // 0..9   tile_index
        // 10..11 palette_select (0..3) => palette bank * 16
        // 12     hflip
        // 13     vflip
        // 14..15 priority (ignored in v0.1)
        // ---------------------------------------------------------------------

        // TEMP: Scroll registers (until PPU regs exist)
        let scroll_x = self.bg0_scroll_x as usize;
        let scroll_y = self.bg0_scroll_y as usize;
        let sy = y.wrapping_add(scroll_y);
        let tile_y = (sy / 8) & 63; // 64-tile wrap (tilemap is treated as 64x64)
        let row_in_tile = sy & 7;

        // Screen tiles across (ceil)
        let tiles_x = (FB_W + 7) / 8;

        for tx in 0..tiles_x {
            let sx = (tx * 8).wrapping_add(scroll_x);
            let tile_x = (sx / 8) & 63;

            // Tilemap index in entries (64x64)
            let map_index = tile_y * 64 + tile_x;
            let map_byte = map_index * 2;

            // Read little-endian u16 entry
            let lo = vram.tilemaps[map_byte] as u16;
            let hi = vram.tilemaps[map_byte + 1] as u16;
            let entry = lo | (hi << 8);

            let tile_index = (entry & 0x03FF) as usize;
            let pal_sel = ((entry >> 10) & 0x3) as u8;
            let hflip = ((entry >> 12) & 0x1) != 0;
            let vflip = ((entry >> 13) & 0x1) != 0;

            let row = if vflip { 7 - row_in_tile } else { row_in_tile };

            // Pattern base: 32 bytes per tile
            let tile_base = tile_index * 32;
            let row_base = tile_base + row * 4;

            // Fetch 4 packed bytes for this row
            let b0 = vram.bg_tiles[row_base];
            let b1 = vram.bg_tiles[row_base + 1];
            let b2 = vram.bg_tiles[row_base + 2];
            let b3 = vram.bg_tiles[row_base + 3];

            // Write 8 pixels
            for px in 0..8 {
                let dst_x = tx * 8 + px;
                if dst_x >= FB_W {
                    continue;
                }

                // Determine source pixel index with optional hflip
                let src_px = if hflip { 7 - px } else { px };

                // Packed nibble extraction (hi nibble = even pixel, lo nibble = odd pixel)
                let (byte, shift_hi) = match src_px {
                    0 => (b0, true),
                    1 => (b0, false),
                    2 => (b1, true),
                    3 => (b1, false),
                    4 => (b2, true),
                    5 => (b2, false),
                    6 => (b3, true),
                    _ => (b3, false),
                };

                let pix4 = if shift_hi {
                    (byte >> 4) & 0x0F
                } else {
                    byte & 0x0F
                };

                // Palette bank: 0..3 => 0,16,32,48
                let color_index = (pal_sel as usize) * 16 + (pix4 as usize);

                // Palette lookup: first 256 entries are RGB555 u16 LE
                let pal_ofs = color_index * 2;
                let plo = vram.palettes[pal_ofs] as u16;
                let phi = vram.palettes[pal_ofs + 1] as u16;
                let rgb555 = plo | (phi << 8);

                let fb_index = y * FB_W + dst_x;
                pixels[fb_index] = rgb555;
            }
        }

        // -------------------------------------------------------------------------
        // Sprite Rendering (Phase 1 - no priority, overwrite BG)
        // -------------------------------------------------------------------------
        // Sort visible sprites by priority (low first)
        let mut sorted_indices = _sprite_indices[.._sprite_count].to_vec();

        sorted_indices.sort_by_key(|&idx| self.oam.sprite(idx).map(|s| s.priority).unwrap_or(0));

        for sprite_index in sorted_indices {
            if let Some(sprite) = self.oam.sprite(sprite_index) {
                let sprite_y = sprite.y as usize;
                let row_in_sprite = y - sprite_y;

                // 8x8 tile, 4bpp, 32 bytes per tile
                let tile_base = sprite.tile_index as usize * 32;

                for col in 0..8 {
                    let screen_x = sprite.x as usize + col;

                    if screen_x >= FB_W {
                        continue;
                    }

                    let byte_index = tile_base + row_in_sprite * 4 + (col / 2);

                    if byte_index >= vram.sprite_tiles.len() {
                        continue;
                    }

                    let byte = vram.sprite_tiles[byte_index];

                    let color_index = if col % 2 == 0 { byte >> 4 } else { byte & 0x0F };

                    if color_index == 0 {
                        continue; // transparent
                    }

                    let palette_offset = sprite.palette as usize * 16;
                    let palette_index = palette_offset + color_index as usize;

                    if palette_index * 2 + 1 >= vram.palettes.len() {
                        continue;
                    }

                    let lo = vram.palettes[palette_index * 2] as u16;
                    let hi = vram.palettes[palette_index * 2 + 1] as u16;
                    let rgb = lo | (hi << 8);

                    pixels[y * FB_W + screen_x] = rgb;
                }
            }
        }
    }
}
