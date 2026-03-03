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
use crate::aurex::ppu::oam::BlendMode;

pub struct Ppu {
    frame_counter: u64,

    // BG0 scroll registers
    bg0_scroll_x: u16,
    bg0_scroll_y: u16,

    // Sprite memory
    oam: Oam,

    // -----------------------------------------------------------------
    // Sprite overflow telemetry (latched per frame)
    // -----------------------------------------------------------------
    sprite_overflow_latched: bool,
    sprite_overflow_scanlines: u32,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            frame_counter: 0,
            bg0_scroll_x: 0,
            bg0_scroll_y: 0,
            oam: Oam::new(),
            sprite_overflow_latched: false,
            sprite_overflow_scanlines: 0,
        }
    }

    // ============================================================================
    // RGB555 Additive Blend
    // ----------------------------------------------------------------------------
    // Adds two RGB555 pixels channel-wise and clamps to 31.
    // Deterministic. No floats. No overflow.
    // ============================================================================
    fn add_rgb555(dst: u16, src: u16) -> u16 {
        let dr = (dst >> 10) & 0x1F;
        let dg = (dst >> 5) & 0x1F;
        let db = dst & 0x1F;

        let sr = (src >> 10) & 0x1F;
        let sg = (src >> 5) & 0x1F;
        let sb = src & 0x1F;

        let r = (dr + sr).min(31);
        let g = (dg + sg).min(31);
        let b = (db + sb).min(31);

        (r << 10) | (g << 5) | b
    }

    // ============================================================================
    // DEBUG ONLY: Direct OAM injection
    // Used for hardware validation (sprite overflow testing)
    // Remove when DMA/OAM pipeline is implemented.
    // ============================================================================
    #[cfg(debug_assertions)]
    pub fn debug_set_sprite(&mut self, index: usize, sprite: super::oam::Sprite) {
        if let Some(slot) = self.oam.sprite_mut(index) {
            *slot = sprite;
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
            sprite.blend = BlendMode::Additive;
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
        // Reset sprite overflow telemetry for this frame
        self.sprite_overflow_latched = false;
        self.sprite_overflow_scanlines = 0;
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
            self.sprite_overflow_latched = true;
            self.sprite_overflow_scanlines += 1;
        }

        // NOTE:
        // - _sprite_indices holds up to 8 sprite indices
        // - _sprite_count is how many are active
        // - _sprite_overflow is true if >8 found
        // Rendering will be implemented in next phase.

        // ---------------------------------------------------------------------
        // BG0 Bring-up (v0.1)
        // - Tilemap source: vram. bg0_tilemap (start)
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

        // -----------------------------------------------------------------------------
        // Per-scanline BG priority buffer (0 = low, 1 = high)
        // Must live for entire scanline (BG + sprite pass)
        // -----------------------------------------------------------------------------
        let mut bg_priority_line = [0u8; FB_W];

        // Screen tiles across (ceil)
        let tiles_x = (FB_W + 7) / 8;

        for tx in 0..tiles_x {
            let sx = (tx * 8).wrapping_add(scroll_x);
            let tile_x = (sx / 8) & 63;

            // Tilemap index in entries (64x64)
            let map_index = tile_y * 64 + tile_x;
            let map_byte = map_index * 2;

            // Read little-endian u16 entry
            let lo = vram.bg0_tilemap[map_byte] as u16;
            let hi = vram.bg0_tilemap[map_byte + 1] as u16;
            let entry = lo | (hi << 8);
            let bg_prio = ((entry >> 14) & 0x1) as u8;

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
                // -----------------------------------------------------------------------------
                // Per-scanline BG priority buffer (0 = low, 1 = high)
                // -----------------------------------------------------------------------------
                let mut bg_priority_line = [0u8; FB_W];
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

                // -----------------------------------------------------------------------------
                // BG transparency tracking (color 0 is transparent)
                // -----------------------------------------------------------------------------
                let bg_transparent = pix4 == 0;

                // Palette bank: 0..3 => 0,16,32,48
                let color_index = (pal_sel as usize) * 16 + (pix4 as usize);

                // Palette lookup: first 256 entries are RGB555 u16 LE
                let pal_ofs = color_index * 2;
                let plo = vram.palettes[pal_ofs] as u16;
                let phi = vram.palettes[pal_ofs + 1] as u16;
                let rgb555 = plo | (phi << 8);

                let fb_index = y * FB_W + dst_x;

                // -----------------------------------------------------------------------------
                // Write BG only if non-transparent
                // -----------------------------------------------------------------------------
                if !bg_transparent {
                    pixels[fb_index] = rgb555;
                    bg_priority_line[dst_x] = bg_prio;
                } else {
                    bg_priority_line[dst_x] = 0;
                }
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

                // -----------------------------------------------------------------------------
                // Sprite pixel decode (supports 8x8 and 16x16)
                // 16x16 uses 2x2 tile composition:
                // [0][1]
                // [2][3]
                // -----------------------------------------------------------------------------

                let sprite_size = if sprite.size_16 { 16 } else { 8 };

                for col in 0..sprite_size {
                    let screen_x = sprite.x as usize + col;

                    if screen_x >= FB_W {
                        continue;
                    }

                    // Determine source coordinates within sprite
                    let local_col = if sprite.hflip {
                        sprite_size - 1 - col
                    } else {
                        col
                    };

                    let local_row = if sprite.vflip {
                        sprite_size - 1 - row_in_sprite
                    } else {
                        row_in_sprite
                    };

                    // Determine which 8x8 quadrant we are in (for 16x16)
                    let (tile_offset, src_col, src_row) = if sprite.size_16 {
                        let quad_x = local_col / 8;
                        let quad_y = local_row / 8;

                        let tile_offset = quad_y * 2 + quad_x;

                        (tile_offset, local_col % 8, local_row % 8)
                    } else {
                        (0, local_col, local_row)
                    };

                    // Each tile is 32 bytes
                    let tile_base = (sprite.tile_index as usize + tile_offset) * 32;

                    let byte_index = tile_base + src_row * 4 + (src_col / 2);

                    if byte_index >= vram.sprite_tiles.len() {
                        continue;
                    }

                    let byte = vram.sprite_tiles[byte_index];

                    let color_index = if src_col % 2 == 0 {
                        (byte >> 4) & 0x0F
                    } else {
                        byte & 0x0F
                    };

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

                    let fb_index = y * FB_W + screen_x;

                    // -----------------------------------------------------------------------------
                    // Sprite vs BG priority resolution
                    // Rule:
                    // - High priority BG blocks low priority sprite
                    // - High priority sprite always wins
                    // -----------------------------------------------------------------------------
                    if bg_priority_line[screen_x] == 1 && sprite.priority == 0 {
                        continue;
                    }

                    match sprite.blend {
                        BlendMode::Normal => {
                            pixels[fb_index] = rgb;
                        }
                        BlendMode::Additive => {
                            let dst = pixels[fb_index];
                            pixels[fb_index] = Self::add_rgb555(dst, rgb);
                        }
                    }
                }
            }
        }
    }

    pub fn sprite_overflow_latched(&self) -> bool {
        self.sprite_overflow_latched
    }

    pub fn sprite_overflow_scanlines(&self) -> u32 {
        self.sprite_overflow_scanlines
    }
}
