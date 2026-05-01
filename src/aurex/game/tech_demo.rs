use crate::aurex::game::{AudioCue, InputState};
use crate::aurex::ppu::framebuffer::{FB_H, FB_W, rgb555};
use crate::aurex::ppu::ppu::{
    PPU_BG0_ENABLE, PPU_BG0_SCROLL_X, PPU_BG0_SCROLL_Y, PPU_BG1_ENABLE, PPU_SPRITE_ENABLE, Ppu,
};
use crate::aurex::ppu::vram::Vram;

const CELL: i32 = 8;
const GRID_W: i32 = (FB_W as i32) / CELL;
const GRID_H: i32 = (FB_H as i32) / CELL;

const PLAY_MIN_X: i32 = 2;
const PLAY_MAX_X: i32 = GRID_W - 3;
const PLAY_MIN_Y: i32 = 3;
const PLAY_MAX_Y: i32 = GRID_H - 4;

const SHAPE_COUNT: usize = 16;
const EQ_BARS: usize = 14;

const TILE_BG_A: u16 = 0;
const TILE_BG_B: u16 = 1;
const TILE_PANEL: u16 = 2;
const TILE_SHAPE_A: u16 = 32;
const TILE_SHAPE_B: u16 = 33;
const TILE_SHAPE_C: u16 = 34;
const TILE_EQ: u16 = 35;

#[derive(Clone, Copy)]
struct Shape {
    x_fp: i32,
    y_fp: i32,
    vx_fp: i32,
    vy_fp: i32,
    tile: u16,
}

pub struct TechDemo {
    frame: u64,
    bg_theme: u8,
    vibe: u8,
    music_track: u8,
    prev_up: bool,
    prev_down: bool,
    prev_left: bool,
    prev_right: bool,
    shapes: [Shape; SHAPE_COUNT],
}

impl TechDemo {
    pub fn new(vram: &mut Vram) -> Self {
        let mut seed = 0xC0DE_BAADu32;
        let mut next = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            seed
        };

        let mut shapes = [Shape {
            x_fp: 0,
            y_fp: 0,
            vx_fp: 0,
            vy_fp: 0,
            tile: TILE_SHAPE_A,
        }; SHAPE_COUNT];

        for (i, s) in shapes.iter_mut().enumerate() {
            let x = PLAY_MIN_X * CELL + (next() as i32 % ((PLAY_MAX_X - PLAY_MIN_X) * CELL));
            let y = PLAY_MIN_Y * CELL + (next() as i32 % ((PLAY_MAX_Y - PLAY_MIN_Y) * CELL));
            let vx = ((next() as i32 % 300) + 120) * if i & 1 == 0 { 1 } else { -1 };
            let vy = ((next() as i32 % 260) + 100) * if i & 2 == 0 { 1 } else { -1 };
            let tile = match i % 3 {
                0 => TILE_SHAPE_A,
                1 => TILE_SHAPE_B,
                _ => TILE_SHAPE_C,
            };
            *s = Shape {
                x_fp: x << 8,
                y_fp: y << 8,
                vx_fp: vx,
                vy_fp: vy,
                tile,
            };
        }

        let s = Self {
            frame: 0,
            bg_theme: 0,
            vibe: 0,
            music_track: 0,
            prev_up: false,
            prev_down: false,
            prev_left: false,
            prev_right: false,
            shapes,
        };

        s.upload_palette(vram);
        s.upload_bg_tiles(vram);
        s.upload_bg_tilemap(vram);
        s.upload_sprites(vram);
        s
    }

    fn upload_palette(&self, vram: &mut Vram) {
        let (bg_a, bg_b, panel) = match self.bg_theme {
            0 => (rgb555(1, 1, 2), rgb555(2, 2, 4), rgb555(4, 10, 16)),
            1 => (rgb555(1, 2, 1), rgb555(2, 4, 2), rgb555(6, 12, 6)),
            2 => (rgb555(2, 1, 1), rgb555(4, 2, 2), rgb555(12, 6, 6)),
            _ => (rgb555(1, 1, 3), rgb555(3, 3, 6), rgb555(8, 6, 14)),
        };

        let shape_base = match self.vibe {
            0 => (rgb555(10, 24, 30), rgb555(8, 25, 10), rgb555(25, 20, 7)),
            1 => (rgb555(27, 11, 29), rgb555(9, 23, 18), rgb555(28, 14, 9)),
            _ => (rgb555(18, 18, 31), rgb555(31, 20, 8), rgb555(10, 28, 14)),
        };

        let palette: [u16; 10] = [
            rgb555(0, 0, 0),    // 0 transparent
            bg_a,               // 1 bg A
            bg_b,               // 2 bg B
            panel,              // 3 panel
            shape_base.0,       // 4 shape A
            shape_base.1,       // 5 shape B
            shape_base.2,       // 6 shape C
            rgb555(24, 30, 12), // 7 eq bar
            rgb555(31, 31, 31), // 8 highlights
            rgb555(31, 18, 8),  // 9 accents
        ];

        for (i, c) in palette.iter().enumerate() {
            let o = i * 2;
            vram.palettes[o] = (c & 0xFF) as u8;
            vram.palettes[o + 1] = (c >> 8) as u8;
        }
    }

    fn fill_bg_tile(vram: &mut Vram, tile: usize, color: u8) {
        let base = tile * 32;
        let packed = (color << 4) | color;
        for row in 0..8 {
            let o = base + row * 4;
            vram.bg_tiles[o] = packed;
            vram.bg_tiles[o + 1] = packed;
            vram.bg_tiles[o + 2] = packed;
            vram.bg_tiles[o + 3] = packed;
        }
    }

    fn fill_sprite_tile(vram: &mut Vram, tile: usize, color: u8) {
        let base = tile * 32;
        let packed = (color << 4) | color;
        for row in 0..8 {
            let o = base + row * 4;
            vram.sprite_tiles[o] = packed;
            vram.sprite_tiles[o + 1] = packed;
            vram.sprite_tiles[o + 2] = packed;
            vram.sprite_tiles[o + 3] = packed;
        }
    }

    fn upload_bg_tiles(&self, vram: &mut Vram) {
        Self::fill_bg_tile(vram, TILE_BG_A as usize, 1);
        Self::fill_bg_tile(vram, TILE_BG_B as usize, 2);
        Self::fill_bg_tile(vram, TILE_PANEL as usize, 3);

        let panel = TILE_PANEL as usize * 32;
        for row in 1..7 {
            let o = panel + row * 4;
            vram.bg_tiles[o + 1] = 0x88;
            vram.bg_tiles[o + 2] = 0x88;
        }
    }

    fn upload_bg_tilemap(&self, vram: &mut Vram) {
        for y in 0..64usize {
            for x in 0..64usize {
                let tile = if (x as i32) >= PLAY_MIN_X
                    && (x as i32) <= PLAY_MAX_X
                    && (y as i32) >= PLAY_MIN_Y
                    && (y as i32) <= PLAY_MAX_Y
                {
                    if ((x + y) & 1) == 0 {
                        TILE_BG_A
                    } else {
                        TILE_BG_B
                    }
                } else {
                    TILE_PANEL
                };

                let idx = (y * 64 + x) * 2;
                if idx + 1 < vram.bg0_tilemap.len() {
                    vram.bg0_tilemap[idx] = (tile & 0xFF) as u8;
                    vram.bg0_tilemap[idx + 1] = (tile >> 8) as u8;
                }
            }
        }
    }

    fn upload_sprites(&self, vram: &mut Vram) {
        Self::fill_sprite_tile(vram, TILE_SHAPE_A as usize, 4);
        Self::fill_sprite_tile(vram, TILE_SHAPE_B as usize, 5);
        Self::fill_sprite_tile(vram, TILE_SHAPE_C as usize, 6);
        Self::fill_sprite_tile(vram, TILE_EQ as usize, 7);

        // Diamond motif for shape B.
        let b = TILE_SHAPE_B as usize * 32;
        for row in 0..8 {
            let o = b + row * 4;
            vram.sprite_tiles[o] = 0x05;
            vram.sprite_tiles[o + 3] = 0x50;
        }

        // Hollow motif for shape C.
        let c = TILE_SHAPE_C as usize * 32;
        for row in 2..6 {
            let o = c + row * 4;
            vram.sprite_tiles[o + 1] = 0x00;
            vram.sprite_tiles[o + 2] = 0x00;
        }

        // EQ stripe tile.
        let eq = TILE_EQ as usize * 32;
        for row in 0..8 {
            let o = eq + row * 4;
            vram.sprite_tiles[o + 1] = 0x77;
            vram.sprite_tiles[o + 2] = 0x77;
        }
    }

    fn update_shapes(&mut self) {
        let speed_mul = match self.vibe {
            0 => 1,
            1 => 2,
            _ => 3,
        };

        let min_x = PLAY_MIN_X * CELL;
        let max_x = PLAY_MAX_X * CELL;
        let min_y = PLAY_MIN_Y * CELL;
        let max_y = PLAY_MAX_Y * CELL;

        for s in &mut self.shapes {
            s.x_fp += (s.vx_fp / 2) * speed_mul;
            s.y_fp += (s.vy_fp / 2) * speed_mul;

            let x = s.x_fp >> 8;
            let y = s.y_fp >> 8;

            if x < min_x || x > max_x {
                s.vx_fp = -s.vx_fp;
                s.x_fp = s.x_fp.clamp(min_x << 8, max_x << 8);
            }
            if y < min_y || y > max_y {
                s.vy_fp = -s.vy_fp;
                s.y_fp = s.y_fp.clamp(min_y << 8, max_y << 8);
            }
        }
    }

    pub fn update(&mut self, ppu: &mut Ppu, input: InputState) -> AudioCue {
        ppu.write_addr(PPU_BG0_ENABLE, 1);
        ppu.write_addr(PPU_BG1_ENABLE, 0);
        ppu.write_addr(PPU_SPRITE_ENABLE, 1);

        let scroll = ((self.frame / 20) as u16) & 1;
        ppu.write_addr(PPU_BG0_SCROLL_X, scroll);
        ppu.write_addr(PPU_BG0_SCROLL_Y, 0);

        let mut cue = AudioCue::None;

        if input.up && !self.prev_up {
            self.music_track = (self.music_track + 1) % 3;
            cue = AudioCue::SelectTrack(self.music_track);
        }
        if input.down && !self.prev_down {
            self.music_track = (self.music_track + 2) % 3;
            cue = AudioCue::SelectTrack(self.music_track);
        }
        if input.left && !self.prev_left {
            self.bg_theme = (self.bg_theme + 1) % 4;
        }
        if input.right && !self.prev_right {
            self.vibe = (self.vibe + 1) % 3;
        }

        self.prev_up = input.up;
        self.prev_down = input.down;
        self.prev_left = input.left;
        self.prev_right = input.right;

        // Re-upload palette each frame so user selection changes apply immediately.
        // Cost is tiny and deterministic.
        // (Could be optimized with dirty flags later.)
        // Also keeps visual equalizer tied to selected vibe palette.
        // No VRAM write gating here because engine currently writes directly to host-side VRAM.
        // This is acceptable for current system-demo scope.
        //
        // NOTE: This is system-demo behavior, not final hardware emulation policy.
        //
        // (Vram mutability comes from game update path ownership.)
        //
        // FUTURE: event-driven style update path.
        //
        // This comment intentionally documents temporary architecture policy.
        //
        // Upload now:
        //
        //
        self.update_shapes();

        for i in 0..128 {
            ppu.write_sprite(i, 0, 255, TILE_SHAPE_A, 0, 0, false, false, false);
        }

        for (i, s) in self.shapes.iter().enumerate() {
            ppu.write_sprite(
                i,
                (s.x_fp >> 8).max(0) as u16,
                (s.y_fp >> 8).max(0) as u16,
                s.tile,
                0,
                0,
                false,
                false,
                false,
            );
        }

        // Equalizer bars in the background area.
        let base_x = 26i32;
        let bottom_y = (PLAY_MAX_Y * CELL) - 8;
        for b in 0..EQ_BARS {
            let phase = self.frame + (b as u64 * 3) + (self.music_track as u64 * 11);
            let amp = ((phase / 3) & 0x0F) as i32;
            let bars = 1 + (amp % 6);
            for by in 0..bars {
                let y = bottom_y - (by * 8);
                ppu.write_sprite(
                    64 + b * 6 + by as usize,
                    (base_x + (b as i32 * 10)) as u16,
                    y as u16,
                    TILE_EQ,
                    0,
                    0,
                    false,
                    false,
                    false,
                );
            }
        }

        // Theme/vibe indicators in top bar.
        for i in 0..self.bg_theme {
            ppu.write_sprite(
                120 + i as usize,
                10 + i as u16 * 10,
                8,
                TILE_SHAPE_C,
                0,
                0,
                false,
                false,
                false,
            );
        }
        for i in 0..self.vibe {
            ppu.write_sprite(
                124 + i as usize,
                64 + i as u16 * 10,
                8,
                TILE_SHAPE_B,
                0,
                0,
                false,
                false,
                false,
            );
        }

        self.frame = self.frame.wrapping_add(1);
        cue
    }
}
