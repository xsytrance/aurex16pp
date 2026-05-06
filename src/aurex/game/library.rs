#![allow(dead_code)]
use crate::aurex::game::{AudioCue, InputState};
use crate::aurex::ppu::framebuffer::{rgb555, Framebuffer, FB_H, FB_W};

#[derive(Clone, Copy)]
struct Bullet {
    x: i32,
    y: i32,
    vy: i32,
    live: bool,
}

#[derive(Clone, Copy)]
struct Enemy {
    x: i32,
    y: i32,
    hp: i32,
    live: bool,
}

#[derive(Clone, Copy)]
pub struct LibraryUpdate {
    pub audio_cue: AudioCue,
    pub launch_requested: bool,
    pub launch_canceled: bool,
}

pub struct LibraryScreen {
    frame: u64,
    player_x: i32,
    player_y: i32,
    bullets: [Bullet; 32],
    enemies: [Enemy; 24],
    rng: u32,
    fire_cd: u8,
    spawn_cd: u8,
    score: u32,
    lives: u8,
}

impl LibraryScreen {
    pub fn new() -> Self {
        Self {
            frame: 0,
            player_x: (FB_W as i32 / 2) - 6,
            player_y: FB_H as i32 - 30,
            bullets: [Bullet { x: 0, y: 0, vy: -6, live: false }; 32],
            enemies: [Enemy { x: 0, y: 0, hp: 1, live: false }; 24],
            rng: 0xC0FFEE11,
            fire_cd: 0,
            spawn_cd: 0,
            score: 0,
            lives: 3,
        }
    }

    pub fn current_audio_cue(&self) -> AudioCue { AudioCue::SelectTrack(0) }
    pub fn current_title(&self) -> &'static str { "AUREX SHMUP" }
    pub fn current_launch_descriptor(&self) -> crate::aurex::runtime::LaunchDescriptor {
        crate::aurex::runtime::LaunchDescriptor { title: "AUREX SHMUP", cartridge_id: "chrome_duo_boot" }
    }
    pub fn set_launch_stage(&mut self, _stage: crate::aurex::runtime::LaunchStage) {}

    fn rnd(&mut self) -> u32 {
        self.rng = self.rng.wrapping_mul(1664525).wrapping_add(1013904223);
        self.rng
    }

    fn spawn_enemy(&mut self) {
        let x = 12 + (self.rnd() % (FB_W as u32 - 24)) as i32;
        for e in &mut self.enemies {
            if !e.live {
                e.live = true;
                e.x = x;
                e.y = -10;
                e.hp = 1 + ((self.frame / 300) as i32 % 2);
                return;
            }
        }
    }

    fn fire_bullet(&mut self) {
        for b in &mut self.bullets {
            if !b.live {
                b.live = true;
                b.x = self.player_x + 5;
                b.y = self.player_y - 2;
                b.vy = -7;
                return;
            }
        }
    }

    pub fn update(&mut self, input: InputState) -> LibraryUpdate {
        self.frame = self.frame.wrapping_add(1);

        // autopilot if no manual directional input
        let mut dx = 0;
        if input.left { dx -= 3; }
        if input.right { dx += 3; }
        if dx == 0 {
            let wave = (((self.frame / 5) as i32 % 120) - 60).signum();
            dx = wave;
        }

        self.player_x = (self.player_x + dx).clamp(6, FB_W as i32 - 12);

        if self.fire_cd > 0 { self.fire_cd -= 1; }
        let wants_fire = input.accept || input.up || (self.frame % 8 == 0);
        if wants_fire && self.fire_cd == 0 {
            self.fire_bullet();
            self.fire_cd = 5;
        }

        if self.spawn_cd > 0 { self.spawn_cd -= 1; }
        if self.spawn_cd == 0 {
            self.spawn_enemy();
            self.spawn_cd = 12;
        }

        // update bullets
        for b in &mut self.bullets {
            if b.live {
                b.y += b.vy;
                if b.y < -8 { b.live = false; }
            }
        }

        // update enemies
        for e in &mut self.enemies {
            if e.live {
                let drift = (((self.frame as i32 + e.x) / 7) % 3) - 1;
                e.y += 2;
                e.x = (e.x + drift).clamp(6, FB_W as i32 - 12);
                if e.y > FB_H as i32 + 8 {
                    e.live = false;
                }
            }
        }

        // collisions: bullets vs enemies
        for b in &mut self.bullets {
            if !b.live { continue; }
            for e in &mut self.enemies {
                if !e.live { continue; }
                let hit = (b.x - e.x).abs() < 8 && (b.y - e.y).abs() < 8;
                if hit {
                    b.live = false;
                    e.hp -= 1;
                    if e.hp <= 0 {
                        e.live = false;
                        self.score = self.score.saturating_add(10);
                    }
                    break;
                }
            }
        }

        // collisions: enemies vs player
        for e in &mut self.enemies {
            if !e.live { continue; }
            let hit = (self.player_x + 5 - e.x).abs() < 10 && (self.player_y + 5 - e.y).abs() < 10;
            if hit {
                e.live = false;
                if self.lives > 0 { self.lives -= 1; }
            }
        }

        if self.lives == 0 {
            // simple reset loop for continuous 15s footage
            self.lives = 3;
            self.score = 0;
            for e in &mut self.enemies { e.live = false; }
            for b in &mut self.bullets { b.live = false; }
        }

        LibraryUpdate { audio_cue: AudioCue::None, launch_requested: false, launch_canceled: false }
    }

    pub fn draw(&self, fb: &mut Framebuffer, frame: u64) {
        self.draw_bg(fb, frame);
        self.draw_player(fb);
        self.draw_bullets(fb);
        self.draw_enemies(fb, frame);
        self.draw_hud(fb);
    }

    fn draw_bg(&self, fb: &mut Framebuffer, frame: u64) {
        let p = fb.pixels_mut();
        for y in 0..FB_H {
            for x in 0..FB_W {
                let neb = ((x as u64 * 3 + y as u64 * 5 + frame) & 31) as u8;
                p[y * FB_W + x] = rgb555(1 + (neb >> 4), 2 + (neb >> 3), 6 + (neb >> 2));
            }
        }
        // stars
        for i in 0..90 {
            let sx = ((i * 47 + (frame as usize * ((i % 5) + 1))) % FB_W) as i32;
            let sy = ((i * 29 + (frame as usize * ((i % 3) + 1))) % FB_H) as i32;
            self.fill_rect(fb, sx, sy, sx + 1, sy + 1, rgb555(26, 28, 31));
        }
    }

    fn draw_player(&self, fb: &mut Framebuffer) {
        let x = self.player_x;
        let y = self.player_y;
        // procedural ship shape
        self.fill_rect(fb, x + 4, y, x + 8, y + 2, rgb555(30, 30, 31));
        self.fill_rect(fb, x + 3, y + 2, x + 9, y + 6, rgb555(8, 24, 31));
        self.fill_rect(fb, x + 1, y + 6, x + 11, y + 10, rgb555(6, 14, 24));
        self.fill_rect(fb, x + 4, y + 8, x + 8, y + 12, rgb555(30, 10, 8));
    }

    fn draw_bullets(&self, fb: &mut Framebuffer) {
        for b in &self.bullets {
            if b.live {
                self.fill_rect(fb, b.x, b.y, b.x + 2, b.y + 5, rgb555(31, 28, 10));
            }
        }
    }

    fn draw_enemies(&self, fb: &mut Framebuffer, frame: u64) {
        let pulse = ((frame / 6) & 1) as i32;
        for e in &self.enemies {
            if e.live {
                let x = e.x;
                let y = e.y;
                self.fill_rect(fb, x - 5, y - 3, x + 5, y + 4, rgb555(26, 8 + pulse as u8, 9));
                self.fill_rect(fb, x - 3, y + 4, x + 3, y + 7, rgb555(31, 16, 9));
                self.fill_rect(fb, x - 2, y - 1, x + 2, y + 1, rgb555(31, 29, 22));
            }
        }
    }

    fn draw_hud(&self, fb: &mut Framebuffer) {
        self.fill_rect(fb, 0, 0, FB_W as i32, 16, rgb555(1, 6, 10));
        self.draw_text(fb, "AUREX SHMUP", 6, 5, 1, rgb555(24, 30, 31));
        let t = format!("SCORE {:05}  LIVES {}", self.score, self.lives);
        self.draw_text(fb, &t, 130, 5, 1, rgb555(30, 28, 14));
    }

    fn fill_rect(&self, fb: &mut Framebuffer, x0: i32, y0: i32, x1: i32, y1: i32, color: u16) {
        let pixels = fb.pixels_mut();
        let x0 = x0.clamp(0, FB_W as i32);
        let x1 = x1.clamp(0, FB_W as i32);
        let y0 = y0.clamp(0, FB_H as i32);
        let y1 = y1.clamp(0, FB_H as i32);
        for y in y0..y1 {
            let row = y as usize * FB_W;
            for x in x0..x1 { pixels[row + x as usize] = color; }
        }
    }

    fn draw_text(&self, fb: &mut Framebuffer, text: &str, x: i32, y: i32, scale: i32, color: u16) {
        let mut cx = x;
        for ch in text.chars() {
            self.draw_char(fb, ch, cx, y, scale, color);
            cx += 6 * scale;
        }
    }

    fn draw_char(&self, fb: &mut Framebuffer, ch: char, x: i32, y: i32, scale: i32, color: u16) {
        let glyph = match ch {
            'A' => [0x0E,0x11,0x11,0x1F,0x11], 'C' => [0x0F,0x10,0x10,0x10,0x0F],
            'E' => [0x1F,0x10,0x1E,0x10,0x1F], 'H' => [0x11,0x11,0x1F,0x11,0x11],
            'I' => [0x1F,0x04,0x04,0x04,0x1F], 'L' => [0x10,0x10,0x10,0x10,0x1F],
            'M' => [0x11,0x1B,0x15,0x11,0x11], 'O' => [0x0E,0x11,0x11,0x11,0x0E],
            'P' => [0x1E,0x11,0x1E,0x10,0x10], 'R' => [0x1E,0x11,0x1E,0x12,0x11],
            'S' => [0x0F,0x10,0x0E,0x01,0x1E], 'U' => [0x11,0x11,0x11,0x11,0x0E],
            'V' => [0x11,0x11,0x11,0x0A,0x04], 'X' => [0x11,0x0A,0x04,0x0A,0x11],
            '0' => [0x0E,0x13,0x15,0x19,0x0E], '1' => [0x04,0x0C,0x04,0x04,0x0E],
            '2' => [0x0E,0x01,0x0E,0x10,0x1F], '3' => [0x1E,0x01,0x0E,0x01,0x1E],
            '4' => [0x12,0x12,0x1F,0x02,0x02], '5' => [0x1F,0x10,0x1E,0x01,0x1E],
            '6' => [0x0E,0x10,0x1E,0x11,0x0E], '7' => [0x1F,0x01,0x02,0x04,0x04],
            '8' => [0x0E,0x11,0x0E,0x11,0x0E], '9' => [0x0E,0x11,0x0F,0x01,0x0E],
            ':' => [0x00,0x04,0x00,0x04,0x00], ' ' => [0,0,0,0,0], _ => [0,0,0,0,0],
        };
        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..5 {
                if (bits >> (4 - col)) & 1 == 1 {
                    self.fill_rect(fb, x + col * scale, y + row as i32 * scale, x + (col + 1) * scale, y + (row as i32 + 1) * scale, color);
                }
            }
        }
    }
}
