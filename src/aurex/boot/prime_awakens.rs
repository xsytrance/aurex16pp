use crate::aurex::DmaController;
use crate::aurex::ppu::framebuffer::{FB_H, FB_W, Framebuffer, rgb555};
use crate::aurex::ppu::ppu::{PPU_BG0_ENABLE, PPU_BG1_ENABLE, PPU_SPRITE_ENABLE, Ppu};
use crate::aurex::ppu::vram::Vram;
use crate::aurex::wram::Wram;

pub struct PrimeAwakens {
    frame: u32,
    waiting_for_start: bool,
}

impl PrimeAwakens {
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
        self.draw_boot_backdrop(fb, t);
        self.draw_energy_grid(fb, t);
        self.draw_boot_ring(fb, t);
        self.draw_aurex_wordmark(fb, t);
        self.draw_status_panels(fb, t);
        self.draw_prompt(fb, t);
    }

    fn draw_boot_backdrop(&self, fb: &mut Framebuffer, t: u32) {
        let pixels = fb.pixels_mut();
        for y in 0..FB_H {
            for x in 0..FB_W {
                let x_u = x as u32;
                let y_u = y as u32;
                let sky = ((x_u.wrapping_mul(3) + y_u.wrapping_mul(5) + t) >> 3) & 15;
                let neb = ((x_u ^ (y_u.wrapping_mul(3) + t.wrapping_mul(2))) >> 2) & 7;
                let horizon = ((FB_H - y) as u32 * 12 / FB_H as u32) as u8;
                let scan = if ((y_u + t / 3) & 7) == 0 { 1 } else { 0 };

                let r = (2 + sky as u8 / 3 + scan).min(31);
                let g = (5 + neb as u8 + horizon / 4 + scan).min(31);
                let b = (10 + sky as u8 + horizon + scan).min(31);
                pixels[y * FB_W + x] = rgb555(r, g, b);
            }
        }

        for s in 0..54u32 {
            let sx = ((s.wrapping_mul(97) + t.wrapping_mul((s % 5) + 1) * 2) % FB_W as u32) as i32;
            let sy = ((s.wrapping_mul(53) + t / ((s % 3) + 3)) % (FB_H as u32 / 2)) as i32;
            let twinkle = (((t / 4 + s * 5) & 7) as u8).min(5);
            fill_rect(
                fb,
                sx,
                sy,
                sx + 1 + (twinkle > 3) as i32,
                sy + 1,
                rgb555(10 + twinkle, 18 + twinkle, 28 + twinkle / 2),
            );
        }
    }

    fn draw_energy_grid(&self, fb: &mut Framebuffer, t: u32) {
        let horizon = 140;
        for i in 0..18i32 {
            let y = horizon + i * 5;
            let tone = (i as u8 / 3).min(8);
            fill_rect(
                fb,
                0,
                y,
                FB_W as i32,
                y + 1,
                rgb555(2 + tone, 8 + tone, 18 + tone),
            );
        }

        for lane in -12..=12i32 {
            let base_x = FB_W as i32 / 2 + lane * 18;
            let drift = (((t / 5) as i32 + lane * 3) & 7) - 3;
            for depth in 0..15i32 {
                let y0 = horizon + depth * 6;
                let y1 = y0 + 6;
                let x = base_x + drift * (depth + 1) / 5;
                let c = rgb555(
                    2,
                    (11 + depth as u8 / 2).min(31),
                    (20 + depth as u8).min(31),
                );
                fill_rect(fb, x, y0, x + 1, y1, c);
            }
        }
    }

    fn draw_boot_ring(&self, fb: &mut Framebuffer, t: u32) {
        let cx = FB_W as i32 / 2;
        let cy = 92;
        let radius = 48 + ((t / 24) % 3) as i32;
        for y in (cy - radius - 2)..=(cy + radius + 2) {
            let dy = y - cy;
            if dy.abs() > radius + 2 {
                continue;
            }
            let span = radius - (dy.abs() * 3 / 5);
            for x in (cx - span - 2)..=(cx + span + 2) {
                let dx = x - cx;
                let d = dx.abs() + dy.abs();
                if d < radius - 4 || d > radius + 2 {
                    continue;
                }
                let shimmer = (((dx * 2 + dy * 3 + t as i32) & 15) as u8) / 3;
                put_pixel(
                    fb,
                    x,
                    y,
                    rgb555(8 + shimmer, 18 + shimmer * 2, (27 + shimmer).min(31)),
                );
            }
        }
    }

    fn draw_aurex_wordmark(&self, fb: &mut Framebuffer, t: u32) {
        let title = "AUREX-16++";
        let subtitle = "SYSTEM BOOT VECTOR";
        let scale = 4usize;
        let title_w = text_width(title, scale);
        let start_x = (FB_W as i32 - title_w) / 2;
        let base_y = 48;

        let reveal_stride = 10u32;
        for (i, ch) in title.chars().enumerate() {
            let reveal_at = i as u32 * reveal_stride;
            let char_x = start_x + (i as i32 * (6 * scale) as i32);
            let slide_frames = 18;
            if t < reveal_at {
                continue;
            }

            let dt = t.saturating_sub(reveal_at);
            let slide = (slide_frames as i32 - dt.min(slide_frames) as i32) * 2;
            let glow = (((t + i as u32 * 3) / 3) & 7) as u8;

            draw_glyph(
                fb,
                ch,
                char_x,
                base_y + slide,
                scale,
                rgb555(6 + glow / 2, 14 + glow, 22 + glow),
            );
            draw_glyph(
                fb,
                ch,
                char_x,
                base_y + slide - 2,
                scale,
                rgb555(18 + glow / 2, 26 + glow / 2, 31),
            );
        }

        draw_text(
            fb,
            subtitle,
            (FB_W as i32 - text_width(subtitle, 2)) / 2,
            100,
            2,
            rgb555(13, 24, 31),
        );
    }

    fn draw_status_panels(&self, fb: &mut Framebuffer, t: u32) {
        fill_rect(fb, 24, 166, 402, 171, rgb555(4, 11, 18));
        let progress = ((t / 3) as i32).min(360);
        fill_rect(
            fb,
            30,
            167,
            30 + progress,
            170,
            rgb555(9, 22 + ((t / 8) % 5) as u8, 28),
        );

        draw_text(fb, "AUDIO BUS: ASU-32", 30, 178, 2, rgb555(12, 24, 31));
        draw_text(fb, "VIDEO BUS: RASTER LOCK", 30, 192, 2, rgb555(12, 24, 31));
        draw_text(fb, "RUNTIME: DETERMINISTIC", 30, 206, 2, rgb555(12, 24, 31));
    }

    fn draw_prompt(&self, fb: &mut Framebuffer, t: u32) {
        if self.waiting_for_start {
            if (t / 12).is_multiple_of(2) {
                draw_text(
                    fb,
                    "PRESS START TO ENTER LIBRARY",
                    78,
                    224,
                    2,
                    rgb555(20, 30, 31),
                );
            }
        } else if (t / 10).is_multiple_of(2) {
            draw_text(
                fb,
                "INITIALIZING KERNEL AND TIMERS...",
                70,
                224,
                2,
                rgb555(14, 22, 31),
            );
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

fn put_pixel(fb: &mut Framebuffer, x: i32, y: i32, color: u16) {
    if !(0..FB_W as i32).contains(&x) || !(0..FB_H as i32).contains(&y) {
        return;
    }
    fb.pixels_mut()[y as usize * FB_W + x as usize] = color;
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
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0A],
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
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        '>' => [0x10, 0x08, 0x04, 0x02, 0x04, 0x08, 0x10],
        ' ' => [0x00; 7],
        _ => [0x1F, 0x11, 0x15, 0x15, 0x15, 0x11, 0x1F],
    }
}
