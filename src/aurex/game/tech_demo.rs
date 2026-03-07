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
const PLAY_MIN_Y: i32 = 4;
const PLAY_MAX_Y: i32 = GRID_H - 3;

const TILE_BG_A: u16 = 0;
const TILE_BG_B: u16 = 1;
const TILE_PANEL: u16 = 2;
const TILE_CURSOR: u16 = 32;
const TILE_NODE_A: u16 = 33;
const TILE_NODE_B: u16 = 34;

#[derive(Clone, Copy)]
struct Node {
    x: i32,
    y: i32,
    phase: u8,
}

pub struct TechDemo {
    frame: u64,
    cursor_x: i32,
    cursor_y: i32,
    move_cooldown: u8,
    nodes: [Node; 8],
}

impl TechDemo {
    pub fn new(vram: &mut Vram) -> Self {
        let s = Self {
            frame: 0,
            cursor_x: (PLAY_MIN_X + PLAY_MAX_X) / 2,
            cursor_y: (PLAY_MIN_Y + PLAY_MAX_Y) / 2,
            move_cooldown: 0,
            nodes: [
                Node {
                    x: PLAY_MIN_X + 2,
                    y: PLAY_MIN_Y + 2,
                    phase: 0,
                },
                Node {
                    x: PLAY_MIN_X + 10,
                    y: PLAY_MIN_Y + 2,
                    phase: 4,
                },
                Node {
                    x: PLAY_MIN_X + 18,
                    y: PLAY_MIN_Y + 2,
                    phase: 8,
                },
                Node {
                    x: PLAY_MIN_X + 26,
                    y: PLAY_MIN_Y + 2,
                    phase: 12,
                },
                Node {
                    x: PLAY_MIN_X + 2,
                    y: PLAY_MAX_Y - 2,
                    phase: 6,
                },
                Node {
                    x: PLAY_MIN_X + 10,
                    y: PLAY_MAX_Y - 2,
                    phase: 10,
                },
                Node {
                    x: PLAY_MIN_X + 18,
                    y: PLAY_MAX_Y - 2,
                    phase: 2,
                },
                Node {
                    x: PLAY_MIN_X + 26,
                    y: PLAY_MAX_Y - 2,
                    phase: 14,
                },
            ],
        };

        s.upload_palette(vram);
        s.upload_bg_tiles(vram);
        s.upload_bg_tilemap(vram);
        s.upload_sprites(vram);
        s
    }

    fn upload_palette(&self, vram: &mut Vram) {
        let palette: [u16; 10] = [
            rgb555(0, 0, 0),    // 0 transparent
            rgb555(1, 1, 2),    // 1 deep bg
            rgb555(2, 2, 4),    // 2 alt bg
            rgb555(4, 10, 16),  // 3 panel border
            rgb555(7, 20, 28),  // 4 panel accent
            rgb555(15, 30, 31), // 5 cursor
            rgb555(6, 24, 8),   // 6 node green
            rgb555(25, 28, 8),  // 7 node amber
            rgb555(29, 10, 10), // 8 node red
            rgb555(31, 31, 31), // 9 white
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
            vram.bg_tiles[o + 1] = 0x44;
            vram.bg_tiles[o + 2] = 0x44;
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
        Self::fill_sprite_tile(vram, TILE_CURSOR as usize, 5);
        Self::fill_sprite_tile(vram, TILE_NODE_A as usize, 6);
        Self::fill_sprite_tile(vram, TILE_NODE_B as usize, 7);

        // Cursor frame hole for readability.
        let c = TILE_CURSOR as usize * 32;
        for row in 2..6 {
            let o = c + row * 4;
            vram.sprite_tiles[o + 1] = 0x00;
            vram.sprite_tiles[o + 2] = 0x00;
        }

        // Node B ring.
        let b = TILE_NODE_B as usize * 32;
        for row in 1..7 {
            let o = b + row * 4;
            vram.sprite_tiles[o] = 0x07;
            vram.sprite_tiles[o + 3] = 0x70;
        }
    }

    pub fn update(&mut self, ppu: &mut Ppu, input: InputState) -> AudioCue {
        ppu.write_addr(PPU_BG0_ENABLE, 1);
        ppu.write_addr(PPU_BG1_ENABLE, 0);
        ppu.write_addr(PPU_SPRITE_ENABLE, 1);
        ppu.write_addr(PPU_BG0_SCROLL_X, ((self.frame / 32) & 1) as u16);
        ppu.write_addr(PPU_BG0_SCROLL_Y, 0);

        for i in 0..128 {
            ppu.write_sprite(i, 0, 255, TILE_NODE_A, 0, 0, false, false, false);
        }

        let mut cue = AudioCue::None;
        if self.move_cooldown > 0 {
            self.move_cooldown -= 1;
        }

        if self.move_cooldown == 0 {
            let (dx, dy) = if input.left {
                (-1, 0)
            } else if input.right {
                (1, 0)
            } else if input.up {
                (0, -1)
            } else if input.down {
                (0, 1)
            } else {
                (0, 0)
            };

            if dx != 0 || dy != 0 {
                self.cursor_x = (self.cursor_x + dx).clamp(PLAY_MIN_X, PLAY_MAX_X);
                self.cursor_y = (self.cursor_y + dy).clamp(PLAY_MIN_Y, PLAY_MAX_Y);
                self.move_cooldown = 4;
                cue = AudioCue::Eat;
            }
        }

        let cursor_tile = if (self.frame / 10).is_multiple_of(2) {
            TILE_CURSOR
        } else {
            TILE_NODE_B
        };
        ppu.write_sprite(
            0,
            (self.cursor_x * CELL) as u16,
            (self.cursor_y * CELL) as u16,
            cursor_tile,
            0,
            0,
            false,
            false,
            false,
        );

        for (i, n) in self.nodes.iter().enumerate() {
            let phase = ((self.frame as u8).wrapping_add(n.phase) / 8) & 0x03;
            let tile = if phase < 2 { TILE_NODE_A } else { TILE_NODE_B };
            ppu.write_sprite(
                8 + i,
                (n.x * CELL) as u16,
                (n.y * CELL) as u16,
                tile,
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
