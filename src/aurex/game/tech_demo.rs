use crate::aurex::game::{AudioCue, InputState};
use crate::aurex::ppu::ppu::{PPU_BG0_ENABLE, PPU_BG1_ENABLE, PPU_SPRITE_ENABLE, Ppu};
use crate::aurex::ppu::vram::Vram;

const CELL: i32 = 8;
const GRID_W: i32 = 53;
const GRID_H: i32 = 30;
const MAX_SEGMENTS: usize = 96;
const STEP_FRAMES: u64 = 7;

const TILE_HEAD: u16 = 32;
const TILE_BODY: u16 = 33;
const TILE_FOOD: u16 = 34;

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
                Point { x: 10, y: 14 },
                Point { x: 9, y: 14 },
                Point { x: 8, y: 14 },
            ],
            food: Point { x: 28, y: 14 },
            alive: true,
            death_cooldown: 0,
            score: 0,
        };

        s.upload_palette(vram);
        s.upload_sprites(vram);
        s
    }

    fn upload_palette(&self, vram: &mut Vram) {
        let palette: [u16; 8] = [
            0x0000, // transparent
            0x7FFF, // white
            0x03E0, // green
            0x02A0, // dark green
            0x001F, // blue
            0x7C00, // red
            0x03FF, // cyan
            0x4210, // gray
        ];

        for (i, c) in palette.iter().enumerate() {
            let o = i * 2;
            vram.palettes[o] = (c & 0xFF) as u8;
            vram.palettes[o + 1] = (c >> 8) as u8;
        }
    }

    fn fill_tile(vram: &mut Vram, tile: usize, color: u8) {
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

    fn upload_sprites(&self, vram: &mut Vram) {
        Self::fill_tile(vram, TILE_HEAD as usize, 2);
        Self::fill_tile(vram, TILE_BODY as usize, 3);
        Self::fill_tile(vram, TILE_FOOD as usize, 5);

        // Eye dots on head.
        let head = TILE_HEAD as usize * 32;
        vram.sprite_tiles[head + 9] = 0x11;
        vram.sprite_tiles[head + 10] = 0x11;

        // Food center highlight.
        let food = TILE_FOOD as usize * 32;
        for row in 2..6 {
            let o = food + row * 4;
            vram.sprite_tiles[o + 1] = 0x66;
            vram.sprite_tiles[o + 2] = 0x66;
        }
    }

    fn random_food(&self) -> Point {
        // Deterministic pseudo-random placement derived from frame + score.
        let seed = self
            .frame
            .wrapping_mul(1103515245)
            .wrapping_add((self.score as u64).wrapping_mul(12345));

        let mut x = ((seed >> 8) % GRID_W as u64) as i32;
        let mut y = ((seed >> 17) % GRID_H as u64) as i32;

        // Avoid spawning on the snake.
        for _ in 0..(GRID_W * GRID_H) {
            if !self.snake.iter().any(|p| p.x == x && p.y == y) {
                return Point { x, y };
            }
            x = (x + 7).rem_euclid(GRID_W);
            y = (y + 11).rem_euclid(GRID_H);
        }

        Point { x: 4, y: 4 }
    }

    fn reset(&mut self) {
        self.dir = Point { x: 1, y: 0 };
        self.pending_dir = self.dir;
        self.snake.clear();
        self.snake.push(Point { x: 10, y: 14 });
        self.snake.push(Point { x: 9, y: 14 });
        self.snake.push(Point { x: 8, y: 14 });
        self.food = Point { x: 28, y: 14 };
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
        ppu.write_addr(PPU_BG0_ENABLE, 0);
        ppu.write_addr(PPU_BG1_ENABLE, 0);
        ppu.write_addr(PPU_SPRITE_ENABLE, 1);

        // Move all sprites off-screen first so stale sprites do not persist.
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

                let hit_wall = next.x < 0 || next.y < 0 || next.x >= GRID_W || next.y >= GRID_H;
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

        // Draw food.
        ppu.write_sprite(
            0,
            (self.food.x * CELL) as u16,
            (self.food.y * CELL) as u16,
            TILE_FOOD,
            0,
            0,
            false,
            false,
            false,
        );

        // Draw snake body.
        for (i, seg) in self.snake.iter().take(120).enumerate() {
            let tile = if i == 0 { TILE_HEAD } else { TILE_BODY };
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

        // Score pips on top row.
        for i in 0..self.score.min(16) {
            ppu.write_sprite(
                112 + i as usize,
                4 + (i as u16 * 10),
                4,
                TILE_FOOD,
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
