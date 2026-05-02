use crate::aurex::cartridge::CartridgeRuntime;
use crate::aurex::dma::controller::DmaController;
use crate::aurex::game::InputState;
use crate::aurex::ppu::oam::{Sprite, MAX_SPRITES};
use crate::aurex::ppu::ppu::{Ppu, PPU_SPRITE_ENABLE};
use crate::aurex::ppu::vram::Vram;
use crate::aurex::runtime::game_runtime::{GameOutcome, GameRuntime};

// Peg Solitaire (replaces Blocks & Bricks cartridge)
// Standard English board: 7x7 cross shape, 32 pegs, one empty center.
// Controls: D-pad to move cursor, A to select/deselect/jump. Jump by moving to an empty hole two steps away with a peg in between.

pub struct BlocksAndBricks {
    board: [[bool; 7]; 7],          // true = peg present
    selected: Option<(usize, usize)>, // currently selected peg grid position
    cursor: (usize, usize),          // cursor grid position
    won: bool,
}

impl BlocksAndBricks {
    pub fn new() -> Self {
        // Build full cross board: valid positions occupied; center removed.
        let mut board = [[false; 7]; 7];
        for r in 0..7 {
            for c in 0..7 {
                board[r][c] = Self::is_valid_hole(r, c);
            }
        }
        board[3][3] = false; // empty start

        let cursor = (3, 2); // default start at first row leftmost (0,2) or (3,2) - both occupied
        Self {
            board,
            selected: None,
            cursor,
            won: false,
        }
    }

    const fn is_valid_hole(r: usize, c: usize) -> bool {
        // Cross pattern: rows 0,1,5,6 have cols 2-4; rows 2,3,4 all cols (0-6)
        match r {
            0 | 1 | 5 | 6 => c >= 2 && c <= 4,
            _ => c < 7,
        }
    }

    fn pos_to_screen(r: usize, c: usize) -> (i32, i32) {
        const CELL_SIZE: i32 = 16;
        const BOARD_DIM: i32 = 7 * CELL_SIZE; // 112
        const START_X: i32 = (426 - BOARD_DIM) / 2;
        const START_Y: i32 = (240 - BOARD_DIM) / 2;
        let x = START_X + c as i32 * CELL_SIZE;
        let y = START_Y + r as i32 * CELL_SIZE;
        (x, y)
    }

    fn upload_sprite_tile(vram: &mut Vram) {
        // Upload solid 8x8 tile (color index 1) to tile index 0
        // 4bpp format: each of 8 rows = 4 bytes; each byte = 0x11 (all pixels = color 1)
        let packed: u8 = 0x11;
        for row in 0..8 {
            let base = row * 4;
            vram.sprite_tiles[base] = packed;
            vram.sprite_tiles[base + 1] = packed;
            vram.sprite_tiles[base + 2] = packed;
            vram.sprite_tiles[base + 3] = packed;
        }
    }

    fn set_sprite(ppu: &mut Ppu, idx: usize, x: u16, y: u16, palette: u16) {
        if idx >= MAX_SPRITES { return; }
        let mut s = Sprite::default();
        s.x = x;
        s.y = y;
        s.tile_index = 0;
        s.palette = palette;
        s.visible = true;
        s.priority = 1;
        ppu.debug_set_sprite(idx, s);
    }

    fn attempt_jump(&mut self, from: (usize, usize), to: (usize, usize)) -> bool {
        let (fr, fc) = from;
        let (tr, tc) = to;
        let dr = (fr as i32 - tr as i32).abs() as usize;
        let dc = (fc as i32 - tc as i32).abs() as usize;
        if (dr == 2 && dc == 0) || (dr == 0 && dc == 2) {
            let mid_r = (fr + tr) / 2;
            let mid_c = (fc + tc) / 2;
            if self.board[mid_r][mid_c] {
                self.board[mid_r][mid_c] = false;
                self.board[tr][tc] = true;
                self.board[fr][fc] = false;
                self.check_win();
                return true;
            }
        }
        false
    }

    fn check_win(&mut self) {
        let count = self.board.iter().flatten().filter(|&&occ| occ).count();
        if count == 1 {
            self.won = true;
        }
    }
}

impl GameRuntime for BlocksAndBricks {
    fn initialize(&mut self, _cartridge: &CartridgeRuntime, vram: &mut Vram, ppu: &mut Ppu) {
        // Upload solid tile (uses color index 1 in VRAM)
        Self::upload_sprite_tile(vram);

        // Set palette entry 1 (normal peg) to red
        let idx = 1;
        let color = 0x7C00; // red (RGB555)
        let off = idx * 2;
        vram.palettes[off] = (color & 0xFF) as u8;
        vram.palettes[off + 1] = (color >> 8) as u8;

        // Set palette entry 2 (selected) to green
        let idx2 = 2;
        let color2 = 0x03E0; // green
        let off2 = idx2 * 2;
        vram.palettes[off2] = (color2 & 0xFF) as u8;
        vram.palettes[off2 + 1] = (color2 >> 8) as u8;

        // Enable sprites rendering
        ppu.write_addr(PPU_SPRITE_ENABLE, 1);
    }

    fn update(&mut self, input: InputState, _ops_budget: u32) -> GameOutcome {
        if self.won {
            return GameOutcome::Running;
        }

        // Move cursor
        let (mut r, mut c) = self.cursor;
        let mut moved = false;
        if input.left && c > 0 {
            c -= 1; moved = true;
        }
        if input.right && c < 6 {
            c += 1; moved = true;
        }
        if input.up && r > 0 {
            r -= 1; moved = true;
        }
        if input.down && r < 6 {
            r += 1; moved = true;
        }
        if moved && Self::is_valid_hole(r, c) {
            self.cursor = (r, c);
        }

        // Accept: select peg or jump
        if input.accept {
            match self.selected {
                None => {
                    if self.board[self.cursor.0][self.cursor.1] {
                        self.selected = Some(self.cursor);
                    }
                }
                Some(sel) => {
                    if sel == self.cursor {
                        self.selected = None;
                    } else if !self.board[self.cursor.0][self.cursor.1] {
                        let _ = self.attempt_jump(sel, self.cursor);
                        self.selected = None;
                    } else {
                        self.selected = Some(self.cursor);
                    }
                }
            }
        }

        GameOutcome::Running
    }

    fn render(&self, ppu: &mut Ppu, _dma: &mut DmaController) {
        // Clear all sprites
        for i in 0..MAX_SPRITES {
            ppu.debug_set_sprite(i, Sprite::default());
        }

        // Draw all pegs (sprites)
        let mut sprite_idx = 0;
        for r in 0..7 {
            for c in 0..7 {
                if !Self::is_valid_hole(r, c) { continue; }
                if self.board[r][c] {
                    let (x, y) = Self::pos_to_screen(r, c);
                    let palette = if self.selected == Some((r, c)) { 2 } else { 1 };
                    Self::set_sprite(ppu, sprite_idx, x as u16, y as u16, palette);
                }
                sprite_idx += 1;
            }
        }
    }

    fn shutdown(&mut self) {}
}
