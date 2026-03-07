use crate::aurex::game::{AudioCue, InputState};
use crate::aurex::ppu::framebuffer::{FB_H, FB_W, rgb555};
use crate::aurex::ppu::ppu::{
    PPU_BG0_ENABLE, PPU_BG0_SCROLL_X, PPU_BG0_SCROLL_Y, PPU_BG1_ENABLE, PPU_SPRITE_ENABLE, Ppu,
};
use crate::aurex::ppu::vram::Vram;

const CELL: i32 = 8;
const GRID_W: i32 = (FB_W as i32) / CELL;
const GRID_H: i32 = (FB_H as i32) / CELL;

const PLAY_MIN_X: i32 = 1;
const PLAY_MAX_X: i32 = GRID_W - 2;
const PLAY_MIN_Y: i32 = 3;
const PLAY_MAX_Y: i32 = GRID_H - 2;

const MAX_SEGMENTS: usize = 96;
const STEP_FRAMES: u64 = 7;

const TILE_BG_DARK: u16 = 0;
const TILE_BG_LIGHT: u16 = 1;
const TILE_BORDER: u16 = 2;

const TILE_HEAD: u16 = 32;
const TILE_BODY: u16 = 33;
const TILE_BODY_GLOW: u16 = 36;
const TILE_FOOD_A: u16 = 34;
const TILE_FOOD_B: u16 = 35;

#[derive(Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

pub struct TechDemo {
    frame: u64,
    dir: Point,
    pending_dir: Point,
    snake: Vec<Point>,
    food: Point,
    alive: bool,
    death_cooldown: u8,
    score: u32,
}

impl TechDemo {
    pub fn new(vram: &mut Vram) -> Self {
        let s = Self {
            frame: 0,
            dir: Point { x: 1, y: 0 },
            pending_dir: Point { x: 1, y: 0 },
            snake: vec![
                Point { x: 12, y: 14 },
                Point { x: 11, y: 14 },
                Point { x: 10, y: 14 },
            ],
            food: Point {
                x: (PLAY_MIN_X + PLAY_MAX_X) / 2,
                y: (PLAY_MIN_Y + PLAY_MAX_Y) / 2,
            },
            alive: true,
            death_cooldown: 0,
            score: 0,
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
            rgb555(1, 1, 3),    // 1 bg near-black blue
            rgb555(2, 2, 5),    // 2 bg dark tile variant
            rgb555(4, 13, 18),  // 3 border deep cyan
            rgb555(6, 22, 8),   // 4 snake body
            rgb555(15, 31, 18), // 5 snake head bright
            rgb555(31, 6, 6),   // 6 food red
            rgb555(31, 18, 6),  // 7 food glow amber
            rgb555(18, 24, 31), // 8 hud sparkle
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
        Self::fill_bg_tile(vram, TILE_BG_DARK as usize, 1);
        Self::fill_bg_tile(vram, TILE_BG_LIGHT as usize, 2);
        Self::fill_bg_tile(vram, TILE_BORDER as usize, 3);

        let border = TILE_BORDER as usize * 32;
        for row in 1..7 {
            let o = border + row * 4;
            vram.bg_tiles[o + 1] = 0x88;
            vram.bg_tiles[o + 2] = 0x88;
        }
    }

    fn upload_bg_tilemap(&self, vram: &mut Vram) {
        // 64x64 BG map space; fill only visible-relevant region with deterministic board.
        for y in 0..64usize {
            for x in 0..64usize {
                let tile = if (x as i32) >= PLAY_MIN_X
                    && (x as i32) <= PLAY_MAX_X
                    && (y as i32) >= PLAY_MIN_Y
                    && (y as i32) <= PLAY_MAX_Y
                {
                    if ((x + y) & 1) == 0 {
                        TILE_BG_DARK
                    } else {
                        TILE_BG_LIGHT
                    }
                } else {
                    TILE_BORDER
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
        Self::fill_sprite_tile(vram, TILE_HEAD as usize, 5);
        Self::fill_sprite_tile(vram, TILE_BODY as usize, 4);
        Self::fill_sprite_tile(vram, TILE_BODY_GLOW as usize, 8);
        Self::fill_sprite_tile(vram, TILE_FOOD_A as usize, 6);
        Self::fill_sprite_tile(vram, TILE_FOOD_B as usize, 7);

        // Head has an edge highlight for readability.
        let head = TILE_HEAD as usize * 32;
        for x in 0..4 {
            vram.sprite_tiles[head + x] = 0x99;
        }

        // Body glow tile inner stripe.
        let body_glow = TILE_BODY_GLOW as usize * 32;
        for row in 2..6 {
            let o = body_glow + row * 4;
            vram.sprite_tiles[o + 1] = 0x88;
            vram.sprite_tiles[o + 2] = 0x88;
        }

        // Food A center sparkle.
        let food_a = TILE_FOOD_A as usize * 32;
        for row in 2..6 {
            let o = food_a + row * 4;
            vram.sprite_tiles[o + 1] = 0x77;
            vram.sprite_tiles[o + 2] = 0x77;
        }

        // Food B ring pattern.
        let food_b = TILE_FOOD_B as usize * 32;
        for row in 1..7 {
            let o = food_b + row * 4;
            vram.sprite_tiles[o] = 0x07;
            vram.sprite_tiles[o + 3] = 0x70;
        }
        vram.sprite_tiles[food_b + 3 * 4 + 1] = 0x77;
        vram.sprite_tiles[food_b + 3 * 4 + 2] = 0x77;
    }

    fn random_food(&self) -> Point {
        let seed = self
            .frame
            .wrapping_mul(1103515245)
            .wrapping_add((self.score as u64).wrapping_mul(12345));

        let span_x = (PLAY_MAX_X - PLAY_MIN_X + 1) as u64;
        let span_y = (PLAY_MAX_Y - PLAY_MIN_Y + 1) as u64;

        let mut x = PLAY_MIN_X + ((seed >> 8) % span_x) as i32;
        let mut y = PLAY_MIN_Y + ((seed >> 17) % span_y) as i32;

        for _ in 0..((PLAY_MAX_X - PLAY_MIN_X + 1) * (PLAY_MAX_Y - PLAY_MIN_Y + 1)) {
            if !self.snake.iter().any(|p| p.x == x && p.y == y) {
                return Point { x, y };
            }
            x += 5;
            if x > PLAY_MAX_X {
                x = PLAY_MIN_X;
            }
            y += 3;
            if y > PLAY_MAX_Y {
                y = PLAY_MIN_Y;
            }
        }

        Point {
            x: PLAY_MIN_X + 3,
            y: PLAY_MIN_Y + 3,
        }
    }

    fn reset(&mut self) {
        self.dir = Point { x: 1, y: 0 };
        self.pending_dir = self.dir;
        self.snake.clear();
        self.snake.push(Point { x: 12, y: 14 });
        self.snake.push(Point { x: 11, y: 14 });
        self.snake.push(Point { x: 10, y: 14 });
        self.food = self.random_food();
        self.alive = true;
        self.death_cooldown = 0;
    }

    fn update_direction(&mut self, input: InputState) {
        let candidate = if input.up {
            Point { x: 0, y: -1 }
        } else if input.down {
            Point { x: 0, y: 1 }
        } else if input.left {
            Point { x: -1, y: 0 }
        } else if input.right {
            Point { x: 1, y: 0 }
        } else {
            self.pending_dir
        };

        if candidate.x == -self.dir.x && candidate.y == -self.dir.y {
            return;
        }

        self.pending_dir = candidate;
    }

    pub fn update(&mut self, ppu: &mut Ppu, input: InputState) -> AudioCue {
        ppu.write_addr(PPU_BG0_ENABLE, 1);
        ppu.write_addr(PPU_BG1_ENABLE, 0);
        ppu.write_addr(PPU_SPRITE_ENABLE, 1);
        // Tiny scan drift to keep board feeling alive without affecting gameplay readability.
        let drift_x = ((self.frame / 20) & 1) as u16;
        ppu.write_addr(PPU_BG0_SCROLL_X, drift_x);
        ppu.write_addr(PPU_BG0_SCROLL_Y, 0);

        for i in 0..128 {
            ppu.write_sprite(i, 0, 255, TILE_BODY, 0, 0, false, false, false);
        }

        let mut cue = AudioCue::None;

        if self.alive {
            self.update_direction(input);

            if self.frame.is_multiple_of(STEP_FRAMES) {
                self.dir = self.pending_dir;

                let head = self.snake[0];
                let next = Point {
                    x: head.x + self.dir.x,
                    y: head.y + self.dir.y,
                };

                let hit_wall = next.x < PLAY_MIN_X
                    || next.y < PLAY_MIN_Y
                    || next.x > PLAY_MAX_X
                    || next.y > PLAY_MAX_Y;
                let hit_self = self.snake.iter().any(|s| s.x == next.x && s.y == next.y);

                if hit_wall || hit_self {
                    self.alive = false;
                    self.death_cooldown = 40;
                    cue = AudioCue::Fail;
                } else {
                    self.snake.insert(0, next);
                    if next.x == self.food.x && next.y == self.food.y {
                        self.score = self.score.saturating_add(1);
                        cue = AudioCue::Eat;
                        self.food = self.random_food();
                        if self.snake.len() > MAX_SEGMENTS {
                            self.snake.pop();
                        }
                    } else {
                        self.snake.pop();
                    }
                }
            }
        } else if self.death_cooldown > 0 {
            self.death_cooldown -= 1;
        } else {
            self.reset();
        }

        // Food pulse animation.
        let food_tile = if (self.frame / 12).is_multiple_of(2) {
            TILE_FOOD_A
        } else {
            TILE_FOOD_B
        };

        ppu.write_sprite(
            0,
            (self.food.x * CELL) as u16,
            (self.food.y * CELL) as u16,
            food_tile,
            0,
            0,
            false,
            false,
            false,
        );

        for (i, seg) in self.snake.iter().take(120).enumerate() {
            let tile = if i == 0 {
                TILE_HEAD
            } else if ((self.frame / 6) + i as u64).is_multiple_of(2) {
                TILE_BODY
            } else {
                TILE_BODY_GLOW
            };
            ppu.write_sprite(
                1 + i,
                (seg.x * CELL) as u16,
                (seg.y * CELL) as u16,
                tile,
                0,
                0,
                false,
                false,
                false,
            );
        }

        // Border corner glints for subtle motion.
        let glint_tile = if (self.frame / 10).is_multiple_of(2) {
            TILE_FOOD_B
        } else {
            TILE_FOOD_A
        };
        let corners = [
            (PLAY_MIN_X * CELL, PLAY_MIN_Y * CELL),
            (PLAY_MAX_X * CELL, PLAY_MIN_Y * CELL),
            (PLAY_MIN_X * CELL, PLAY_MAX_Y * CELL),
            (PLAY_MAX_X * CELL, PLAY_MAX_Y * CELL),
        ];
        for (i, (x, y)) in corners.iter().enumerate() {
            ppu.write_sprite(
                96 + i,
                *x as u16,
                *y as u16,
                glint_tile,
                0,
                0,
                false,
                false,
                false,
            );
        }

        // Top HUD spark pips = score.
        for i in 0..self.score.min(16) {
            ppu.write_sprite(
                112 + i as usize,
                8 + (i as u16 * 12),
                8,
                TILE_FOOD_B,
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
