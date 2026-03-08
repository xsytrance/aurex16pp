use crate::aurex::DmaController;
use crate::aurex::ppu::framebuffer::{FB_H, FB_W, Framebuffer, rgb555};
use crate::aurex::ppu::ppu::{PPU_BG0_ENABLE, PPU_BG1_ENABLE, PPU_SPRITE_ENABLE, Ppu};
use crate::aurex::ppu::vram::Vram;
use crate::aurex::wram::Wram;

pub struct PrimeIgnition {
    frame: u32,
    waiting_for_start: bool,
}

impl PrimeIgnition {
    const EQ_X: i32 = 70;
    const EQ_Y: i32 = 182;

    pub fn new() -> Self {
        Self {
            frame: 0,
            waiting_for_start: false,
        }
    }

    pub fn set_waiting_for_start(&mut self, waiting_for_start: bool) {
        self.waiting_for_start = waiting_for_start;
    }

    pub fn update(
        &mut self,
        ppu: &mut Ppu,
        _dma: &mut DmaController,
        _wram: &mut Wram,
        _vram: &Vram,
    ) {
        ppu.write_addr(PPU_BG0_ENABLE, 0);
        ppu.write_addr(PPU_BG1_ENABLE, 0);
        ppu.write_addr(PPU_SPRITE_ENABLE, 0);
        self.frame = self.frame.wrapping_add(1);
    }

    pub fn draw_overlay(&self, fb: &mut Framebuffer) {
        let t = self.frame;
        self.draw_plasma_backdrop(fb, t);
        self.draw_hypnotic_grid(fb, t);
        self.draw_city_equalizer(fb, t);
        self.draw_logo_stack(fb, t);
        self.draw_prompt(fb, t);
    }

    fn draw_plasma_backdrop(&self, fb: &mut Framebuffer, t: u32) {
        let px = fb.pixels_mut();
        for y in 0..FB_H {
            for x in 0..FB_W {
                let cx = x as i32 - (FB_W as i32 / 2);
                let cy = y as i32 - (FB_H as i32 / 2);
                let d = (cx.abs() + cy.abs()) as u32;
                let band = ((d + t * 3) >> 3) & 15;
                let wave = (((x as u32 * 3 + t * 5) ^ (y as u32 * 5 + t * 2)) >> 4) & 7;

                let r = ((band / 5) + ((t >> 8) & 1)).min(31) as u8;
                let g = (4 + wave / 2 + (band / 4)).min(31) as u8;
                let b = (8 + band + wave).min(31) as u8;

                px[y * FB_W + x] = rgb555(r, g, b);
            }
        }

        fill_rect(fb, 0, 0, FB_W as i32, 16, rgb555(1, 5, 10));
        fill_rect(
            fb,
            0,
            (FB_H - 16) as i32,
            FB_W as i32,
            FB_H as i32,
            rgb555(1, 5, 10),
        );
    }

    fn draw_hypnotic_grid(&self, fb: &mut Framebuffer, t: u32) {
        let horizon = 128;
        for row in 0..20i32 {
            let y = horizon + row * 5;
            let c = rgb555(
                (row as u8 / 7).min(31),
                10 + (row as u8 / 2),
                20 + (row as u8 / 2),
            );
            fill_rect(fb, 0, y, FB_W as i32, y + 1, c);
        }

        for lane in -8..=8i32 {
            let base_x = FB_W as i32 / 2 + lane * 28;
            let sway = (((t as i32 / 3) + lane * 5) & 7) - 3;
            for row in 0..18i32 {
                let y0 = horizon + row * 6;
                let y1 = y0 + 6;
                let width = row + 1;
                let x = base_x + sway * width / 5;
                let c = rgb555(2, (10 + row as u8 / 2).min(31), (18 + row as u8).min(31));
                fill_rect(fb, x, y0, x + 1, y1, c);
            }
        }
    }

    fn draw_city_equalizer(&self, fb: &mut Framebuffer, t: u32) {
        fill_rect(
            fb,
            Self::EQ_X - 10,
            Self::EQ_Y - 14,
            Self::EQ_X + 280,
            Self::EQ_Y + 30,
            rgb555(1, 4, 9),
        );

        for bar in 0..34i32 {
            let phase = ((t as i32 / 2) + bar * 3) & 31;
            let pulse = if phase < 16 { phase } else { 31 - phase };
            let h = 4 + pulse / 2;
            let x0 = Self::EQ_X + bar * 8;
            let c = rgb555(
                (6 + (bar as u8 / 8)).min(31),
                (14 + (bar as u8 & 0x03)).min(31),
                (22 + (pulse as u8 / 3)).min(31),
            );
            fill_rect(fb, x0, Self::EQ_Y + 12 - h, x0 + 5, Self::EQ_Y + 12, c);
        }
    }

    fn draw_logo_stack(&self, fb: &mut Framebuffer, t: u32) {
        let title = "AUREX-16++";
        let scale = if t < 160 { 5 } else { 6 };
        let title_w = text_width(title, scale);
        let x = ((FB_W as i32 - title_w) / 2).max(0);

        draw_text(fb, title, x - 2, 56, scale, rgb555(4, 12, 20));
        draw_text(fb, title, x, 58, scale, rgb555(24, 30, 31));

        let flash = (((t / 5) & 3) as u8).min(3);
        draw_text(
            fb,
            "HYPER RHYTHM OS",
            118,
            118,
            2,
            rgb555(16 + flash, 24 + flash, 30),
        );
        draw_text(
            fb,
            "KOSHIRO-INSPIRED BOOT MIX",
            86,
            136,
            2,
            rgb555(12 + flash, 20 + flash, 27),
        );
    }

    fn draw_prompt(&self, fb: &mut Framebuffer, t: u32) {
        if self.waiting_for_start {
            if (t / 14).is_multiple_of(2) {
                draw_text(
                    fb,
                    "PRESS START TO ENTER LIBRARY",
                    76,
                    212,
                    2,
                    rgb555(22, 29, 31),
                );
            }
        } else if (t / 10).is_multiple_of(2) {
            draw_text(fb, "SYNTH GRID WARMUP...", 130, 212, 2, rgb555(16, 24, 30));
        }
    }
}

fn fill_rect(fb: &mut Framebuffer, x0: i32, y0: i32, x1: i32, y1: i32, color: u16) {
    let pixels = fb.pixels_mut();
    let x0 = x0.clamp(0, FB_W as i32);
    let x1 = x1.clamp(0, FB_W as i32);
    let y0 = y0.clamp(0, FB_H as i32);
    let y1 = y1.clamp(0, FB_H as i32);

    for y in y0..y1 {
        for x in x0..x1 {
            pixels[y as usize * FB_W + x as usize] = color;
        }
    }
}

fn draw_text(fb: &mut Framebuffer, text: &str, x: i32, y: i32, scale: usize, color: u16) {
    let mut cursor_x = x;
    for ch in text.chars() {
        draw_glyph(fb, ch, cursor_x, y, scale, color);
        cursor_x += (6 * scale) as i32;
    }
}

fn text_width(text: &str, scale: usize) -> i32 {
    let chars = text.chars().count() as i32;
    if chars == 0 {
        0
    } else {
        chars * (6 * scale) as i32 - scale as i32
    }
}

fn draw_glyph(fb: &mut Framebuffer, ch: char, x: i32, y: i32, scale: usize, color: u16) {
    let glyph = glyph_5x7(ch);
    let pixels = fb.pixels_mut();

    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..5usize {
            if bits & (1 << (4 - col)) == 0 {
                continue;
            }

            for sy in 0..scale {
                let py = y + (row * scale + sy) as i32;
                if !(0..FB_H as i32).contains(&py) {
                    continue;
                }

                for sx in 0..scale {
                    let px = x + (col * scale + sx) as i32;
                    if !(0..FB_W as i32).contains(&px) {
                        continue;
                    }
                    pixels[py as usize * FB_W + px as usize] = color;
                }
            }
        }
    }
}

fn glyph_5x7(ch: char) -> [u8; 7] {
    match ch {
        'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' => [0x0F, 0x10, 0x10, 0x10, 0x10, 0x10, 0x0F],
        'D' => [0x1E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E],
        'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        'G' => [0x0F, 0x10, 0x10, 0x13, 0x11, 0x11, 0x0F],
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'M' => [0x11, 0x1B, 0x15, 0x11, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x0A, 0x0A, 0x04],
        'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        '0' => [0x0E, 0x13, 0x15, 0x19, 0x11, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x14, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
        '3' => [0x1E, 0x01, 0x01, 0x06, 0x01, 0x01, 0x1E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x10, 0x1E, 0x01, 0x01, 0x1E],
        '6' => [0x0E, 0x10, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x01, 0x0E],
        '+' => [0x00, 0x04, 0x04, 0x1F, 0x04, 0x04, 0x00],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        ':' => [0x00, 0x04, 0x00, 0x00, 0x00, 0x04, 0x00],
        '/' => [0x01, 0x01, 0x02, 0x04, 0x08, 0x10, 0x10],
        '>' => [0x10, 0x08, 0x04, 0x02, 0x04, 0x08, 0x10],
        ' ' => [0x00; 7],
        _ => [0x1F, 0x11, 0x15, 0x15, 0x15, 0x11, 0x1F],
    }
}
