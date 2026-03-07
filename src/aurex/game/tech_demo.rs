use crate::aurex::game::InputState;
use crate::aurex::ppu::framebuffer::{FB_H, FB_W, rgb555};
use crate::aurex::ppu::ppu::{
    PPU_BG0_ENABLE, PPU_BG0_SCROLL_X, PPU_BG0_SCROLL_Y, PPU_BG1_ENABLE, PPU_SPRITE_ENABLE, Ppu,
};
use crate::aurex::ppu::vram::Vram;

// NOTE: All visuals in this module intentionally use placeholder art/colors.
// The logic is structured so final authored sprites/tiles can be swapped in later
// without redesigning movement/collision/gameplay systems.

const LEVEL_W_TILES: usize = 128;
const LEVEL_H_TILES: usize = 64;

const TILE_SKY: u16 = 0;
const TILE_GROUND: u16 = 1;
const TILE_BLOCK: u16 = 2;
const TILE_PIPE: u16 = 3;
const TILE_FLAG: u16 = 4;

const PLAYER_W: i32 = 16;
const PLAYER_H: i32 = 16;

const PLAYER_TILE_BASE: u16 = 32;
const ENEMY_TILE_BASE: u16 = 40;
const COIN_TILE_BASE: u16 = 44;

const COYOTE_FRAMES: u8 = 6;
const JUMP_BUFFER_FRAMES: u8 = 6;

#[derive(Clone, Copy)]
struct Coin {
    x: i32,
    y: i32,
    collected: bool,
}

pub struct TechDemo {
    level: Vec<u16>,
    frame: u64,
    player_x_fp: i32,
    player_y_fp: i32,
    player_vx_fp: i32,
    player_vy_fp: i32,
    player_on_ground: bool,
    prev_jump_pressed: bool,
    coyote_timer: u8,
    jump_buffer_timer: u8,
    spawn_x_fp: i32,
    spawn_y_fp: i32,
    enemy_x_fp: i32,
    won: bool,
    coins: Vec<Coin>,
    coins_collected: u32,
}

impl TechDemo {
    pub fn new(vram: &mut Vram) -> Self {
        let mut s = Self {
            level: vec![TILE_SKY; LEVEL_W_TILES * LEVEL_H_TILES],
            frame: 0,
            player_x_fp: 24 << 8,
            player_y_fp: 120 << 8,
            player_vx_fp: 0,
            player_vy_fp: 0,
            player_on_ground: false,
            prev_jump_pressed: false,
            coyote_timer: 0,
            jump_buffer_timer: 0,
            spawn_x_fp: 24 << 8,
            spawn_y_fp: 120 << 8,
            enemy_x_fp: 72 << 8,
            won: false,
            coins: vec![],
            coins_collected: 0,
        };

        s.build_level();
        s.seed_coins();
        s.upload_tiles(vram);
        s.upload_palette(vram);
        s.upload_tilemap(vram);
        s
    }

    fn build_level(&mut self) {
        for y in 0..LEVEL_H_TILES {
            for x in 0..LEVEL_W_TILES {
                self.level[y * LEVEL_W_TILES + x] = if y >= 28 { TILE_GROUND } else { TILE_SKY };
            }
        }

        for x in 12..20 {
            self.set_tile(x, 22, TILE_BLOCK);
        }
        for x in 30..35 {
            self.set_tile(x, 18, TILE_BLOCK);
        }
        for x in 45..52 {
            self.set_tile(x, 24, TILE_BLOCK);
        }

        for y in 24..28 {
            self.set_tile(60, y, TILE_PIPE);
            self.set_tile(61, y, TILE_PIPE);
        }

        for x in 74..85 {
            self.set_tile(x, 20, TILE_BLOCK);
        }

        // Pit
        for y in 26..28 {
            for x in 98..104 {
                self.set_tile(x, y, TILE_SKY);
            }
        }

        // Goal marker tile
        self.set_tile(118, 20, TILE_FLAG);
    }

    fn seed_coins(&mut self) {
        // Placeholder collectible layout (world pixel coords).
        self.coins = vec![
            Coin {
                x: 14 * 8,
                y: 20 * 8,
                collected: false,
            },
            Coin {
                x: 18 * 8,
                y: 20 * 8,
                collected: false,
            },
            Coin {
                x: 32 * 8,
                y: 16 * 8,
                collected: false,
            },
            Coin {
                x: 48 * 8,
                y: 22 * 8,
                collected: false,
            },
            Coin {
                x: 76 * 8,
                y: 18 * 8,
                collected: false,
            },
            Coin {
                x: 82 * 8,
                y: 18 * 8,
                collected: false,
            },
            Coin {
                x: 109 * 8,
                y: 22 * 8,
                collected: false,
            },
            Coin {
                x: 116 * 8,
                y: 19 * 8,
                collected: false,
            },
        ];
    }

    fn upload_palette(&self, vram: &mut Vram) {
        let colors = [
            rgb555(0, 0, 0),    // 0 transparent
            rgb555(7, 18, 31),  // 1 sky
            rgb555(8, 23, 8),   // 2 ground
            rgb555(22, 20, 9),  // 3 block
            rgb555(4, 18, 4),   // 4 pipe
            rgb555(31, 8, 6),   // 5 player red
            rgb555(28, 21, 13), // 6 skin
            rgb555(0, 0, 16),   // 7 overalls
            rgb555(31, 31, 31), // 8 accent
            rgb555(7, 30, 28),  // 9 enemy cyan
            rgb555(30, 30, 6),  // 10 coin/flag yellow
        ];

        for (i, c) in colors.iter().enumerate() {
            let o = i * 2;
            vram.palettes[o] = (c & 0xFF) as u8;
            vram.palettes[o + 1] = (c >> 8) as u8;
        }
    }

    fn fill_tile(vram: &mut Vram, tile_index: usize, color: u8) {
        let base = tile_index * 32;
        let b = (color << 4) | color;
        for row in 0..8 {
            let r = base + row * 4;
            vram.bg_tiles[r] = b;
            vram.bg_tiles[r + 1] = b;
            vram.bg_tiles[r + 2] = b;
            vram.bg_tiles[r + 3] = b;
        }
    }

    fn fill_sprite_tile(vram: &mut Vram, tile_index: usize, color: u8) {
        let base = tile_index * 32;
        let b = (color << 4) | color;
        for row in 0..8 {
            let r = base + row * 4;
            vram.sprite_tiles[r] = b;
            vram.sprite_tiles[r + 1] = b;
            vram.sprite_tiles[r + 2] = b;
            vram.sprite_tiles[r + 3] = b;
        }
    }

    fn upload_tiles(&self, vram: &mut Vram) {
        Self::fill_tile(vram, TILE_SKY as usize, 1);
        Self::fill_tile(vram, TILE_GROUND as usize, 2);
        Self::fill_tile(vram, TILE_BLOCK as usize, 3);
        Self::fill_tile(vram, TILE_PIPE as usize, 4);
        Self::fill_tile(vram, TILE_FLAG as usize, 10);

        // player 16x16 sprite at 32..35
        let player_base = PLAYER_TILE_BASE as usize;
        for i in 0..4 {
            Self::fill_sprite_tile(vram, player_base + i, 5);
        }
        // face placeholders
        let tl = player_base * 32;
        for row in 2..5 {
            let o = tl + row * 4;
            vram.sprite_tiles[o + 1] = 0x66;
            vram.sprite_tiles[o + 2] = 0x66;
        }
        vram.sprite_tiles[tl + 3 * 4 + 1] = 0x88;
        vram.sprite_tiles[tl + 3 * 4 + 2] = 0x88;

        // enemy sprite 16x16 at 40..43
        let enemy_base = ENEMY_TILE_BASE as usize;
        for i in 0..4 {
            Self::fill_sprite_tile(vram, enemy_base + i, 9);
        }

        // coin sprite 16x16 at 44..47 (simple ring-ish placeholder)
        let coin_base = COIN_TILE_BASE as usize;
        for i in 0..4 {
            Self::fill_sprite_tile(vram, coin_base + i, 10);
        }
        let coin_tl = coin_base * 32;
        for row in 1..7 {
            let r = coin_tl + row * 4;
            vram.sprite_tiles[r] = 0x0A;
            vram.sprite_tiles[r + 3] = 0xA0;
        }
    }

    fn upload_tilemap(&self, vram: &mut Vram) {
        for y in 0..LEVEL_H_TILES {
            for x in 0..LEVEL_W_TILES {
                let tile = self.level[y * LEVEL_W_TILES + x];
                let idx = (y * 64 + x) * 2;
                if idx + 1 < vram.bg0_tilemap.len() {
                    vram.bg0_tilemap[idx] = (tile & 0xFF) as u8;
                    vram.bg0_tilemap[idx + 1] = (tile >> 8) as u8;
                }
            }
        }
    }

    fn set_tile(&mut self, x: usize, y: usize, tile: u16) {
        if x < LEVEL_W_TILES && y < LEVEL_H_TILES {
            self.level[y * LEVEL_W_TILES + x] = tile;
        }
    }

    fn tile_solid(&self, tx: i32, ty: i32) -> bool {
        if tx < 0 || ty < 0 || tx as usize >= LEVEL_W_TILES || ty as usize >= LEVEL_H_TILES {
            return true;
        }
        matches!(
            self.level[ty as usize * LEVEL_W_TILES + tx as usize],
            TILE_GROUND | TILE_BLOCK | TILE_PIPE
        )
    }

    fn respawn(&mut self) {
        self.player_x_fp = self.spawn_x_fp;
        self.player_y_fp = self.spawn_y_fp;
        self.player_vx_fp = 0;
        self.player_vy_fp = 0;
        self.player_on_ground = false;
        self.coyote_timer = 0;
        self.jump_buffer_timer = 0;
    }

    pub fn update(&mut self, ppu: &mut Ppu, input: InputState) {
        ppu.write_addr(PPU_BG0_ENABLE, 1);
        ppu.write_addr(PPU_BG1_ENABLE, 0);
        ppu.write_addr(PPU_SPRITE_ENABLE, 1);

        if !self.won {
            // Horizontal input + drag
            let accel = 42;
            let max_vx = 420;
            if input.left {
                self.player_vx_fp -= accel;
            }
            if input.right {
                self.player_vx_fp += accel;
            }
            if !input.left && !input.right {
                self.player_vx_fp = (self.player_vx_fp * 12) / 16;
            }
            self.player_vx_fp = self.player_vx_fp.clamp(-max_vx, max_vx);

            // Jump buffer / coyote for tighter feel.
            if input.jump && !self.prev_jump_pressed {
                self.jump_buffer_timer = JUMP_BUFFER_FRAMES;
            }
            if self.jump_buffer_timer > 0 {
                self.jump_buffer_timer -= 1;
            }

            if self.player_on_ground {
                self.coyote_timer = COYOTE_FRAMES;
            } else if self.coyote_timer > 0 {
                self.coyote_timer -= 1;
            }

            if self.jump_buffer_timer > 0 && self.coyote_timer > 0 {
                self.player_vy_fp = -(6 << 8);
                self.player_on_ground = false;
                self.coyote_timer = 0;
                self.jump_buffer_timer = 0;
            }

            // Gravity
            self.player_vy_fp += 30;
            self.player_vy_fp = self.player_vy_fp.min(6 << 8);

            // Horizontal collision
            let mut next_x = self.player_x_fp + self.player_vx_fp;
            let dir = if self.player_vx_fp >= 0 { 1 } else { -1 };
            let edge_x = ((next_x >> 8) + if dir > 0 { PLAYER_W - 1 } else { 0 }) / 8;
            let top_y = (self.player_y_fp >> 8) / 8;
            let bottom_y = ((self.player_y_fp >> 8) + PLAYER_H - 1) / 8;
            if self.tile_solid(edge_x, top_y) || self.tile_solid(edge_x, bottom_y) {
                next_x = self.player_x_fp;
                self.player_vx_fp = 0;
            }
            self.player_x_fp = next_x;

            // Vertical collision
            let mut next_y = self.player_y_fp + self.player_vy_fp;
            self.player_on_ground = false;
            if self.player_vy_fp >= 0 {
                let fy = ((next_y >> 8) + PLAYER_H) / 8;
                let lx = (self.player_x_fp >> 8) / 8;
                let rx = ((self.player_x_fp >> 8) + PLAYER_W - 1) / 8;
                if self.tile_solid(lx, fy) || self.tile_solid(rx, fy) {
                    next_y = ((fy * 8) - PLAYER_H) << 8;
                    self.player_vy_fp = 0;
                    self.player_on_ground = true;
                }
            } else {
                let hy = (next_y >> 8) / 8;
                let lx = (self.player_x_fp >> 8) / 8;
                let rx = ((self.player_x_fp >> 8) + PLAYER_W - 1) / 8;
                if self.tile_solid(lx, hy) || self.tile_solid(rx, hy) {
                    next_y = ((hy + 1) * 8) << 8;
                    self.player_vy_fp = 0;
                }
            }
            self.player_y_fp = next_y;

            // Pit death reset
            if (self.player_y_fp >> 8) > (FB_H as i32 + 32) {
                self.respawn();
            }

            // Enemy patrol + collision
            let enemy_center = (72 << 8) + (((self.frame as i32 % 120) - 60) * (90 << 8) / 60);
            self.enemy_x_fp = enemy_center;

            let px = self.player_x_fp >> 8;
            let py = self.player_y_fp >> 8;
            let ex = self.enemy_x_fp >> 8;
            let ey = 208;
            let overlap = px < ex + 16 && px + 16 > ex && py < ey + 16 && py + 16 > ey;
            if overlap {
                self.respawn();
            }

            // Coin collection
            for coin in &mut self.coins {
                if coin.collected {
                    continue;
                }
                let overlap = px < coin.x + 12
                    && px + PLAYER_W > coin.x
                    && py < coin.y + 12
                    && py + PLAYER_H > coin.y;
                if overlap {
                    coin.collected = true;
                    self.coins_collected += 1;
                }
            }

            // Win condition: reach flag and collect all coins
            let all_collected = self.coins_collected as usize >= self.coins.len();
            if (self.player_x_fp >> 8) > (118 * 8) && all_collected {
                self.won = true;
                self.player_vx_fp = 0;
            }
        }

        self.prev_jump_pressed = input.jump;

        let world_px = self.player_x_fp >> 8;
        let max_scroll = ((LEVEL_W_TILES * 8) as i32 - FB_W as i32).max(0);
        let cam_x = (world_px - (FB_W as i32 / 3)).clamp(0, max_scroll) as u16;
        ppu.write_addr(PPU_BG0_SCROLL_X, cam_x);
        ppu.write_addr(PPU_BG0_SCROLL_Y, 0);

        // Player
        let screen_x = (world_px - cam_x as i32).max(0) as u16;
        let screen_y = (self.player_y_fp >> 8).clamp(0, (FB_H - 16) as i32) as u16;
        ppu.write_sprite(
            0,
            screen_x,
            screen_y,
            PLAYER_TILE_BASE,
            0,
            0,
            true,
            false,
            false,
        );

        // Enemy
        let enemy_screen_x =
            ((self.enemy_x_fp >> 8) - cam_x as i32).clamp(0, (FB_W - 16) as i32) as u16;
        ppu.write_sprite(
            1,
            enemy_screen_x,
            208,
            ENEMY_TILE_BASE,
            0,
            1,
            true,
            false,
            false,
        );

        // Flag marker
        let flag_x_world = 118 * 8;
        let flag_screen_x = (flag_x_world - cam_x as i32).clamp(0, (FB_W - 16) as i32) as u16;
        ppu.write_sprite(
            2,
            flag_screen_x,
            160,
            COIN_TILE_BASE,
            0,
            2,
            true,
            true,
            false,
        );

        // Coins in world (first 8 visible handled by 8-sprite scanline cap naturally)
        let mut spr = 8usize;
        for coin in &self.coins {
            if coin.collected {
                continue;
            }
            if spr >= 32 {
                break;
            }
            let x = (coin.x - cam_x as i32).clamp(0, (FB_W - 16) as i32) as u16;
            let y = coin.y.clamp(0, FB_H as i32 - 16) as u16;
            ppu.write_sprite(spr, x, y, COIN_TILE_BASE, 0, 1, true, false, false);
            spr += 1;
        }

        // Simple HUD-like coin indicators (top-left placeholder pips)
        let shown = self.coins_collected.min(8);
        for i in 0..shown {
            ppu.write_sprite(
                48 + i as usize,
                6 + (i as u16 * 10),
                6,
                COIN_TILE_BASE,
                0,
                2,
                false,
                false,
                false,
            );
        }

        self.frame = self.frame.wrapping_add(1);
    }
}
