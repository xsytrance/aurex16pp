use crate::aurex::DmaController;
use crate::aurex::ppu::framebuffer::{FB_H, FB_W, Framebuffer, rgb555};
use crate::aurex::ppu::ppu::{PPU_BG0_ENABLE, PPU_BG1_ENABLE, PPU_SPRITE_ENABLE, Ppu};
use crate::aurex::ppu::vram::Vram;
use crate::aurex::wram::Wram;

pub struct PrimeIgnition {
    frame: u32,
    confirming: bool,
}

impl PrimeIgnition {
    pub fn new() -> Self {
        Self {
            frame: 0,
            confirming: false,
        }
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
        let t = self.frame.min(300);
        self.draw_backdrop(fb, t);

        let logo_scale = if t < 120 { 5 } else { 6 };
        let title = "AUREX-16++";
        let title_w = text_width(title, logo_scale);
        let x = ((FB_W as i32 - title_w) / 2).max(0);
        let y = 84;

        let glow = ((t / 8) & 7) as u8;
        draw_text(
            fb,
            title,
            x - 2,
            y - 2,
            logo_scale,
            rgb555(4, 8 + glow, 18 + glow),
        );
        draw_text(fb, title, x, y, logo_scale, rgb555(20, 29, 31));

        if t > 80 {
            let alpha = ((t - 80) / 3).min(20) as u8;
            draw_text(
                fb,
                "NEXT-GEN CARTRIDGE SYSTEM",
                86,
                154,
                2,
                rgb555(8 + alpha / 4, 14 + alpha / 3, 18 + alpha / 2),
            );
        }

        if (220..300).contains(&t) && (t / 10).is_multiple_of(2) {
            draw_text(fb, "BOOTING LIBRARY...", 122, 210, 2, rgb555(14, 22, 28));
        }
    }

    fn draw_backdrop(&self, fb: &mut Framebuffer, t: u32) {
        let pixels = fb.pixels_mut();
        for y in 0..FB_H {
            for x in 0..FB_W {
                let scan = (((x as u32 + t) >> 4) ^ ((y as u32 + t * 2) >> 5)) & 7;
                let b = (2 + (y as i32 * 10 / FB_H as i32) + scan as i32).clamp(0, 31) as u8;
                let g = (1 + (scan / 2)) as u8;
                pixels[y * FB_W + x] = rgb555(0, g, b);
            }
        }

        // Top and bottom framing bars.
        fill_rect(fb, 0, 0, FB_W as i32, 18, rgb555(1, 5, 10));
        fill_rect(
            fb,
            0,
            (FB_H - 18) as i32,
            FB_W as i32,
            FB_H as i32,
            rgb555(1, 5, 10),
        );
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
        'G' => [0x0F, 0x10, 0x10, 0x13, 0x11, 0x11, 0x0F],
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        '0' => [0x0E, 0x13, 0x15, 0x19, 0x11, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x14, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
        '6' => [0x0E, 0x10, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '+' => [0x00, 0x04, 0x04, 0x1F, 0x04, 0x04, 0x00],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        ' ' => [0x00; 7],
        _ => [0x1F, 0x11, 0x15, 0x15, 0x15, 0x11, 0x1F],
    }

    pub fn set_confirming(&mut self, confirming: bool) {
        self.confirming = confirming;
    }

    pub fn draw_overlay(&self, fb: &mut Framebuffer) {
        let logo_color = rgb555(31, 31, 31);
        let shadow_color = rgb555(4, 6, 10);
        let prompt_color = rgb555(20, 29, 31);

        let title = "AUREX-16++";
        let title_w = self.text_width(title, 4);
        let title_x = ((FB_W as i32 - title_w) / 2).max(0);

        self.draw_text(fb, title, title_x + 2, 48, 4, shadow_color);
        self.draw_text(fb, title, title_x, 46, 4, logo_color);

        let prompt = if self.confirming {
            "LOADING..."
        } else {
            "PRESS ANY BUTTON TO CONTINUE"
        };
        let prompt_w = self.text_width(prompt, 2);
        let prompt_x = ((FB_W as i32 - prompt_w) / 2).max(0);

        if self.confirming || (self.frame / 20).is_multiple_of(2) {
            self.draw_text(fb, prompt, prompt_x, 212, 2, prompt_color);
        }
    }

    fn draw_text(
        &self,
        fb: &mut Framebuffer,
        text: &str,
        x: i32,
        y: i32,
        scale: usize,
        color: u16,
    ) {
        let mut cursor_x = x;
        for ch in text.chars() {
            self.draw_glyph(fb, ch, cursor_x, y, scale, color);
            cursor_x += (6 * scale) as i32;
        }
    }

    fn text_width(&self, text: &str, scale: usize) -> i32 {
        let chars = text.chars().count() as i32;
        if chars == 0 {
            0
        } else {
            chars * (6 * scale) as i32 - scale as i32
        }
    }

    fn draw_glyph(&self, fb: &mut Framebuffer, ch: char, x: i32, y: i32, scale: usize, color: u16) {
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
}

fn glyph_5x7(ch: char) -> [u8; 7] {
    match ch {
        'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' => [0x0F, 0x10, 0x10, 0x10, 0x10, 0x10, 0x0F],
        'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        'G' => [0x0F, 0x10, 0x10, 0x13, 0x11, 0x11, 0x0F],
        'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        '6' => [0x0E, 0x10, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        '+' => [0x00, 0x04, 0x04, 0x1F, 0x04, 0x04, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x06],
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        _ => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    }
}
