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

// === AUREX HARDWARE: PPU REGISTER ADDRESS MAP ===
pub const PPU_BG0_SCROLL_X: u16 = 0x0000;
pub const PPU_BG0_SCROLL_Y: u16 = 0x0002;
pub const PPU_BG1_SCROLL_X: u16 = 0x0004;
pub const PPU_BG1_SCROLL_Y: u16 = 0x0006;

pub const PPU_WINDOW_ENABLE: u16 = 0x0010;
pub const PPU_WINDOW_TOP: u16 = 0x0012;
pub const PPU_WINDOW_BOTTOM: u16 = 0x0014;

pub const PPU_BG0_ENABLE: u16 = 0x0020;
pub const PPU_BG1_ENABLE: u16 = 0x0022;
pub const PPU_SPRITE_ENABLE: u16 = 0x0024;

pub const PPU_WINDOW_LEFT: u16 = 0x0016;
pub const PPU_WINDOW_RIGHT: u16 = 0x0018;

// === AUREX SDK SURFACE: PPU REGISTER ENUM ===
pub enum PpuReg {
    Bg0ScrollX,
    Bg0ScrollY,
    Bg1ScrollX,
    Bg1ScrollY,
    WindowEnable,
    WindowTop,
    WindowBottom,
    WindowLeft,
    WindowRight,
    Bg0Enable,
    Bg1Enable,
    SpriteEnable,
}

// === AUREX SDK SURFACE: PPU STATE SNAPSHOT ===
// Captures PPU register-like state for save-state / deterministic replay.
// Does NOT include VRAM, OAM, or framebuffer.
#[derive(Clone, Copy, Debug, Default)]
pub struct PpuState {
    pub frame_counter: u64,

    // Scroll regs
    pub bg0_scroll_x: u16,
    pub bg0_scroll_y: u16,
    pub bg1_scroll_x: u16,
    pub bg1_scroll_y: u16,

    // Window regs
    pub window_enabled: bool,
    pub window_top: u16,
    pub window_bottom: u16,

    // Layer enable regs
    pub bg0_enable: bool,
    pub bg1_enable: bool,
    pub sprite_enable: bool,
}

pub struct Ppu {
    frame_counter: u64,

    // BG0 scroll registers
    bg0_scroll_x: u16,
    bg0_scroll_y: u16,

    // -----------------------------------------------------------------------------
    // BG1 scroll registers
    // -----------------------------------------------------------------------------
    pub bg1_scroll_x: u16,
    pub bg1_scroll_y: u16,

    // -----------------------------------------------------------------------------
    // Simple window system (Phase 4)
    // -----------------------------------------------------------------------------
    pub window_enabled: bool,
    pub window_top: u16,
    pub window_bottom: u16,
    pub window_left: u16,
    pub window_right: u16,

    // -----------------------------------------------------------------------------
    // Layer enable flags (Phase 5)
    // -----------------------------------------------------------------------------
    pub bg0_enable: bool,
    pub bg1_enable: bool,
    pub sprite_enable: bool,

    // -----------------------------------------------------------------------------
    // Per-scanline scroll tables (Phase 3 - scanline effects)
    // -----------------------------------------------------------------------------
    pub bg0_scroll_x_line: [u16; FB_H],
    pub bg1_scroll_x_line: [u16; FB_H],

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
            bg1_scroll_x: 0,
            bg1_scroll_y: 0,
            bg0_scroll_x_line: [0; FB_H],
            bg1_scroll_x_line: [0; FB_H],
            oam: Oam::new(),
            sprite_overflow_latched: false,
            sprite_overflow_scanlines: 0,
            window_enabled: false,
            window_top: 0,
            window_bottom: 0,
            window_left: 0,
            window_right: (FB_W as u16).saturating_sub(1),
            bg0_enable: true,
            bg1_enable: true,
            sprite_enable: true,
        }
    }

    // === AUREX HARDWARE: ADDRESS-BASED REGISTER READ ===
    pub fn read_addr(&self, addr: u16) -> u16 {
        match addr {
            PPU_BG0_SCROLL_X => self.read_reg(PpuReg::Bg0ScrollX),
            PPU_BG0_SCROLL_Y => self.read_reg(PpuReg::Bg0ScrollY),
            PPU_BG1_SCROLL_X => self.read_reg(PpuReg::Bg1ScrollX),
            PPU_BG1_SCROLL_Y => self.read_reg(PpuReg::Bg1ScrollY),

            PPU_WINDOW_ENABLE => self.read_reg(PpuReg::WindowEnable),
            PPU_WINDOW_TOP => self.read_reg(PpuReg::WindowTop),
            PPU_WINDOW_BOTTOM => self.read_reg(PpuReg::WindowBottom),
            PPU_WINDOW_LEFT => self.read_reg(PpuReg::WindowLeft),
            PPU_WINDOW_RIGHT => self.read_reg(PpuReg::WindowRight),

            PPU_BG0_ENABLE => self.read_reg(PpuReg::Bg0Enable),
            PPU_BG1_ENABLE => self.read_reg(PpuReg::Bg1Enable),
            PPU_SPRITE_ENABLE => self.read_reg(PpuReg::SpriteEnable),

            _ => 0,
        }
    }

    // === AUREX HARDWARE: ADDRESS-BASED REGISTER WRITE ===
    pub fn write_addr(&mut self, addr: u16, value: u16) {
        match addr {
            PPU_BG0_SCROLL_X => self.write_reg(PpuReg::Bg0ScrollX, value),
            PPU_BG0_SCROLL_Y => self.write_reg(PpuReg::Bg0ScrollY, value),
            PPU_BG1_SCROLL_X => self.write_reg(PpuReg::Bg1ScrollX, value),
            PPU_BG1_SCROLL_Y => self.write_reg(PpuReg::Bg1ScrollY, value),

            PPU_WINDOW_ENABLE => self.write_reg(PpuReg::WindowEnable, value),
            PPU_WINDOW_TOP => self.write_reg(PpuReg::WindowTop, value),
            PPU_WINDOW_BOTTOM => self.write_reg(PpuReg::WindowBottom, value),
            PPU_WINDOW_LEFT => self.write_reg(PpuReg::WindowLeft, value),
            PPU_WINDOW_RIGHT => self.write_reg(PpuReg::WindowRight, value),

            PPU_BG0_ENABLE => self.write_reg(PpuReg::Bg0Enable, value),
            PPU_BG1_ENABLE => self.write_reg(PpuReg::Bg1Enable, value),
            PPU_SPRITE_ENABLE => self.write_reg(PpuReg::SpriteEnable, value),

            _ => {
                // Unknown register — ignore for now
            }
        }
    }

    // === AUREX SDK SURFACE: PPU SNAPSHOT ===
    pub fn snapshot(&self) -> PpuState {
        PpuState {
            frame_counter: self.frame_counter,

            bg0_scroll_x: self.bg0_scroll_x,
            bg0_scroll_y: self.bg0_scroll_y,
            bg1_scroll_x: self.bg1_scroll_x,
            bg1_scroll_y: self.bg1_scroll_y,

            window_enabled: self.window_enabled,
            window_top: self.window_top,
            window_bottom: self.window_bottom,

            bg0_enable: self.bg0_enable,
            bg1_enable: self.bg1_enable,
            sprite_enable: self.sprite_enable,
        }
    }

    // === AUREX SDK SURFACE: PPU RESTORE ===
    pub fn restore(&mut self, s: PpuState) {
        self.frame_counter = s.frame_counter;

        // Restore via register bus to preserve layering rules.
        self.write_reg(PpuReg::Bg0ScrollX, s.bg0_scroll_x);
        self.write_reg(PpuReg::Bg0ScrollY, s.bg0_scroll_y);
        self.write_reg(PpuReg::Bg1ScrollX, s.bg1_scroll_x);
        self.write_reg(PpuReg::Bg1ScrollY, s.bg1_scroll_y);

        self.write_reg(PpuReg::WindowEnable, s.window_enabled as u16);
        self.write_reg(PpuReg::WindowTop, s.window_top);
        self.write_reg(PpuReg::WindowBottom, s.window_bottom);

        self.write_reg(PpuReg::Bg0Enable, s.bg0_enable as u16);
        self.write_reg(PpuReg::Bg1Enable, s.bg1_enable as u16);
        self.write_reg(PpuReg::SpriteEnable, s.sprite_enable as u16);
    }

    // === AUREX SDK SURFACE: PPU REGISTER WRITE ===
    pub fn write_reg(&mut self, reg: PpuReg, value: u16) {
        match reg {
            PpuReg::Bg0ScrollX => self.bg0_scroll_x = value,
            PpuReg::Bg0ScrollY => self.bg0_scroll_y = value,
            PpuReg::Bg1ScrollX => self.bg1_scroll_x = value,
            PpuReg::Bg1ScrollY => self.bg1_scroll_y = value,

            PpuReg::WindowEnable => self.window_enabled = value != 0,
            PpuReg::WindowTop => self.window_top = value,
            PpuReg::WindowBottom => self.window_bottom = value,
            PpuReg::WindowLeft => self.window_left = value,
            PpuReg::WindowRight => self.window_right = value,

            PpuReg::Bg0Enable => self.bg0_enable = value != 0,
            PpuReg::Bg1Enable => self.bg1_enable = value != 0,
            PpuReg::SpriteEnable => self.sprite_enable = value != 0,
        }
    }

    // === AUREX SDK SURFACE: PPU REGISTER READ ===
    pub fn read_reg(&self, reg: PpuReg) -> u16 {
        match reg {
            PpuReg::Bg0ScrollX => self.bg0_scroll_x,
            PpuReg::Bg0ScrollY => self.bg0_scroll_y,
            PpuReg::Bg1ScrollX => self.bg1_scroll_x,
            PpuReg::Bg1ScrollY => self.bg1_scroll_y,

            PpuReg::WindowEnable => self.window_enabled as u16,
            PpuReg::WindowTop => self.window_top,
            PpuReg::WindowBottom => self.window_bottom,
            PpuReg::WindowLeft => self.window_left,
            PpuReg::WindowRight => self.window_right,

            PpuReg::Bg0Enable => self.bg0_enable as u16,
            PpuReg::Bg1Enable => self.bg1_enable as u16,
            PpuReg::SpriteEnable => self.sprite_enable as u16,
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
        size_16: bool,
        hflip: bool,
        vflip: bool,
    ) {
        if let Some(sprite) = self.oam.sprite_mut(index) {
            sprite.x = x;
            sprite.y = y;
            sprite.tile_index = tile;
            sprite.palette = palette;
            sprite.priority = priority;
            sprite.visible = true;
            sprite.blend = BlendMode::Additive;
            sprite.size_16 = size_16;
            sprite.hflip = hflip;
            sprite.vflip = vflip;
        }
    }

    // ---------------------------------------------------------------------
    // Register setters (to be wired to CPU later)
    // ---------------------------------------------------------------------
    pub fn set_bg0_scroll(&mut self, x: u16, y: u16) {
        self.bg0_scroll_x = x;
        self.bg0_scroll_y = y;
    }

    // === AUREX CORE: FRAME RENDER ENTRY ===
    // PPU renders strictly from register state.
    // No TEMP TEST register mutation allowed here.

    pub fn render_frame(&mut self, vram: &Vram, fb: &mut Framebuffer) {
        // -------------------------------------------------------------------------
        // Frame Begin
        // -------------------------------------------------------------------------

        // Reset sprite overflow telemetry for this frame
        self.sprite_overflow_latched = false;
        self.sprite_overflow_scanlines = 0;

        // -------------------------------------------------------------------------
        // Read scroll registers from bus
        // (PPU does not mutate registers internally)
        // -------------------------------------------------------------------------
        let bg0_scroll_x = self.read_addr(PPU_BG0_SCROLL_X);
        let bg0_scroll_y = self.read_addr(PPU_BG0_SCROLL_Y);

        // -------------------------------------------------------------------------
        // Build per-scanline scroll tables (Phase 3)
        // -------------------------------------------------------------------------
        for y in 0..FB_H {
            // BG0: normal scroll
            self.bg0_scroll_x_line[y] = bg0_scroll_x;

            // BG1: scroll comes directly from register state
            let bg1_scroll_x = self.read_addr(PPU_BG1_SCROLL_X);
            self.bg1_scroll_x_line[y] = bg1_scroll_x;
        }

        // -------------------------------------------------------------------------
        // Render all scanlines
        // -------------------------------------------------------------------------
        for y in 0..FB_H {
            self.render_scanline(vram, y, fb);
        }

        // Frame counter
        self.frame_counter += 1;
    }

    // === AUREX HOT PATH: SCANLINE RENDER ===

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
        // -------------------------------------------------------------------------
        // BG0 Rendering
        // -------------------------------------------------------------------------
        if self.bg0_enable {
            let scroll_x = self.bg0_scroll_x_line[y] as usize;
            let scroll_y = self.bg0_scroll_y as usize;
            let sy = y.wrapping_add(scroll_y);
            let tile_y = (sy / 8) & 63; // 64-tile wrap (tilemap is treated as 64x64)
            let row_in_tile = sy & 7;

            // -----------------------------------------------------------------------------
            // Per-scanline BG priority buffer (0 = low, 1 = high)
            // Must live for entire scanline (BG + sprite pass)
            // -----------------------------------------------------------------------------
            let mut bg_priority_line = [0u8; FB_W];

            // -----------------------------------------------------------------------------
            // Window check (vertical clip)
            // -----------------------------------------------------------------------------
            let window_active = self.window_enabled
                && (y as u16) >= self.window_top
                && (y as u16) <= self.window_bottom;

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
        }

        // TEMP: Scroll registers (until PPU regs exist)
        // -------------------------------------------------------------------------
        // BG1 Rendering
        // -------------------------------------------------------------------------
        // TEMP TEST — Parallax: BG1 scrolls slower than BG0
        let win_on = self.window_enabled;
        let win_top = self.window_top as usize;
        let win_bot = self.window_bottom as usize;
        let win_left = self.window_left as usize;
        let win_right = self.window_right as usize;

        let y_in_window = !win_on || (y >= win_top && y <= win_bot);

        if self.bg1_enable && y_in_window {
            let scroll_x = self.bg1_scroll_x_line[y] as usize;
            let scroll_y = (self.bg0_scroll_y as usize) / 2;
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
                let lo = vram.bg1_tilemap[map_byte] as u16;
                let hi = vram.bg1_tilemap[map_byte + 1] as u16;
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
                    let dst_x = tx * 8 + px;
                    if dst_x >= FB_W {
                        continue;
                    }

                    // -----------------------------------------------------------------
                    // Horizontal window mask (BG1 only)
                    // -----------------------------------------------------------------
                    if win_on && (dst_x < win_left || dst_x > win_right) {
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
        }

        // -------------------------------------------------------------------------
        // Layer enable: Sprites
        // -----------------------------------------------------------------------------
        if self.sprite_enable {
            // -------------------------------------------------------------------------
            // Sprite Rendering (Phase 1 - no priority, overwrite BG)
            // -------------------------------------------------------------------------
            // Sort visible sprites by priority (low first)
            let mut sorted_indices = _sprite_indices[.._sprite_count].to_vec();

            sorted_indices
                .sort_by_key(|&idx| self.oam.sprite(idx).map(|s| s.priority).unwrap_or(0));

            for sprite_index in sorted_indices {
                if let Some(sprite) = self.oam.sprite(sprite_index) {
                    // -------------------------------------------------------------------------
                    // Sprite Rendering (8x8 / 16x16) + Global Flip Support
                    // -------------------------------------------------------------------------

                    let sprite_x = sprite.x as usize;
                    let sprite_y = sprite.y as usize;

                    let sprite_size = if sprite.size_16 { 16 } else { 8 };

                    // Skip scanlines outside sprite vertically
                    if y < sprite_y || y >= sprite_y + sprite_size {
                        continue;
                    }

                    let row_in_sprite = y - sprite_y;

                    let tiles_per_row = if sprite.size_16 { 2 } else { 1 };

                    // Iterate across full sprite width (8 or 16)
                    for screen_dx in 0..sprite_size {
                        let screen_x = sprite_x + screen_dx;

                        if screen_x >= FB_W {
                            continue;
                        }

                        // Apply full-sprite flip logic
                        let src_x = if sprite.hflip {
                            sprite_size - 1 - screen_dx
                        } else {
                            screen_dx
                        };

                        let src_y = if sprite.vflip {
                            sprite_size - 1 - row_in_sprite
                        } else {
                            row_in_sprite
                        };

                        let tile_col = src_x / 8;
                        let tile_row = src_y / 8;

                        let col_in_tile = src_x & 7;
                        let row_in_tile = src_y & 7;

                        let tile_index_offset = tile_row * tiles_per_row + tile_col;
                        let tile_index = sprite.tile_index as usize + tile_index_offset;

                        let tile_base = tile_index * 32;
                        let byte_index = tile_base + row_in_tile * 4 + (col_in_tile / 2);

                        if byte_index >= vram.sprite_tiles.len() {
                            continue;
                        }

                        let byte = vram.sprite_tiles[byte_index];

                        let color_index = if col_in_tile % 2 == 0 {
                            byte >> 4
                        } else {
                            byte & 0x0F
                        };

                        if color_index == 0 {
                            continue;
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
    }

    pub fn sprite_overflow_latched(&self) -> bool {
        self.sprite_overflow_latched
    }

    pub fn sprite_overflow_scanlines(&self) -> u32 {
        self.sprite_overflow_scanlines
    }
}
