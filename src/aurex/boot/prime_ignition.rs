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
        let t = self.frame.min(360);
        self.draw_backdrop(fb, t);
        self.draw_accent_rails(fb, t);

        let title = "AUREX-16++";
        let logo_scale = if t < 90 { 5 } else { 6 };
        let title_w = text_width(title, logo_scale);
        let x = ((FB_W as i32 - title_w) / 2).max(0);
        let y = 78;

        draw_text(fb, title, x - 2, y - 2, logo_scale, rgb555(6, 14, 23));
        draw_text(fb, title, x, y, logo_scale, rgb555(24, 30, 31));
        draw_text(fb, "NEON IGNITION", 132, 132, 2, rgb555(18, 26, 31));
        draw_text(fb, "EDM CORE ONLINE", 132, 148, 2, rgb555(15, 23, 29));

        self.draw_boot_meter(fb, t);

        if self.waiting_for_start {
            if (self.frame / 12).is_multiple_of(2) {
                draw_text(
                    fb,
                    "PRESS START // GO STRAIGHT",
                    86,
                    210,
                    2,
                    rgb555(24, 29, 31),
                );
            }
        } else if (220..360).contains(&t) && (t / 8).is_multiple_of(2) {
            draw_text(
                fb,
                "BOOTING LIBRARY GRID...",
                112,
                210,
                2,
                rgb555(16, 24, 30),
            );
        }
    }

    fn draw_backdrop(&self, fb: &mut Framebuffer, t: u32) {
        let pixels = fb.pixels_mut();
        for y in 0..FB_H {
            for x in 0..FB_W {
                let cx = x as i32 - FB_W as i32 / 2;
                let cy = y as i32 - FB_H as i32 / 2;
                let ring = ((cx.abs() + cy.abs()) as u32 + t * 2) & 31;
                let stripe = (((x as u32 * 5 + y as u32 * 3 + t * 4) >> 3) & 7) as i32;
                let b = (7 + ring as i32 / 2 + stripe).clamp(0, 31) as u8;
                let g = (3 + stripe / 2 + (t as i32 >> 6)).clamp(0, 31) as u8;
                let r = (((x as u32 + t) >> 7) & 1) as u8;
                pixels[y * FB_W + x] = rgb555(r, g, b);
            }
        }

        fill_rect(fb, 0, 0, FB_W as i32, 16, rgb555(2, 7, 12));
        fill_rect(
            fb,
            0,
            (FB_H - 16) as i32,
            FB_W as i32,
            FB_H as i32,
            rgb555(2, 7, 12),
        );
    }

    fn draw_accent_rails(&self, fb: &mut Framebuffer, t: u32) {
        self.draw_tunnel(fb, t);
    }

    fn draw_boot_meter(&self, fb: &mut Framebuffer, t: u32) {
        self.draw_equalizer(fb, t);
    }

    fn draw_tunnel(&self, fb: &mut Framebuffer, t: u32) {
        for i in 0..8 {
            let s = 20 + i * 16 + ((t as i32 + i * 5) & 7);
            let x0 = FB_W as i32 / 2 - s;
            let y0 = FB_H as i32 / 2 - (s / 2);
            let x1 = FB_W as i32 / 2 + s;
            let y1 = FB_H as i32 / 2 + (s / 2);
            let c = rgb555(
                (2 + i as u8 / 2).min(31),
                (10 + i as u8).min(31),
                (18 + i as u8).min(31),
            );
            stroke_rect(fb, x0, y0, x1, y1, c);
        }
    }

    fn draw_equalizer(&self, fb: &mut Framebuffer, t: u32) {
        let meter_x = 108;
        let meter_y = 176;
        fill_rect(
            fb,
            meter_x - 10,
            meter_y - 10,
            meter_x + 212,
            meter_y + 24,
            rgb555(1, 6, 10),
        );

        for bar in 0..24i32 {
            let wave = (((t as i32 >> 1) + bar * 3) & 15) - 7;
            let h = 3 + wave.abs();
            let x0 = meter_x + bar * 8;
            let c = rgb555(
                (6 + ((bar >> 3) as u8)).min(31),
                (14 + (bar as u8 & 0x03)).min(31),
                (24 + ((t >> 4) as u8 & 0x03)).min(31),
            );
            fill_rect(fb, x0, meter_y + 8 - h, x0 + 5, meter_y + 10, c);
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

fn stroke_rect(fb: &mut Framebuffer, x0: i32, y0: i32, x1: i32, y1: i32, color: u16) {
    fill_rect(fb, x0, y0, x1, y0 + 1, color);
    fill_rect(fb, x0, y1 - 1, x1, y1, color);
    fill_rect(fb, x0, y0, x0 + 1, y1, color);
    fill_rect(fb, x1 - 1, y0, x1, y1, color);
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
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'M' => [0x11, 0x1B, 0x15, 0x11, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
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
