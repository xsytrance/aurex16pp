use crate::aurex::game::{AudioCue, InputState};
use crate::aurex::ppu::framebuffer::{FB_H, FB_W, Framebuffer, rgb555};

const TITLES: [&str; 6] = [
    "NEON CIRCUIT",
    "SKYLINE DRIFT",
    "PIXEL SENTINEL",
    "VOID CARTOGRAPHER",
    "MECHA RIFT",
    "ORBITAL RUSH",
];

pub struct LibraryScreen {
    selected: usize,
    prev_up: bool,
    prev_down: bool,
}

impl LibraryScreen {
    pub fn new() -> Self {
        Self {
            selected: 0,
            prev_up: false,
            prev_down: false,
        }
    }

    pub fn update(&mut self, input: InputState) -> AudioCue {
        if input.up && !self.prev_up {
            self.selected = (self.selected + TITLES.len() - 1) % TITLES.len();
        }
        if input.down && !self.prev_down {
            self.selected = (self.selected + 1) % TITLES.len();
        }

        self.prev_up = input.up;
        self.prev_down = input.down;

        AudioCue::None
    }

    pub fn draw(&self, fb: &mut Framebuffer, frame: u64) {
        self.draw_backdrop(fb, frame);
        self.draw_header(fb);
        self.draw_cards(fb, frame);
        self.draw_footer(fb);
    }

    fn draw_backdrop(&self, fb: &mut Framebuffer, frame: u64) {
        let pixels = fb.pixels_mut();
        for y in 0..FB_H {
            for x in 0..FB_W {
                let wave = (((x as u64 + frame) >> 4) as i32
                    - ((y as u64 + frame * 2) >> 5) as i32)
                    & 0x07;
                let b = (4 + (y as i32 * 8 / FB_H as i32) + wave).clamp(0, 31) as u8;
                let g = (2 + wave / 2) as u8;
                let color = rgb555(1, g, b);
                pixels[y * FB_W + x] = color;
            }
        }
    }

    fn draw_header(&self, fb: &mut Framebuffer) {
        self.fill_rect(fb, 16, 12, (FB_W - 16) as i32, 42, rgb555(2, 8, 14));
        self.draw_text(fb, "AUREX-16++ LIBRARY", 26, 20, 2, rgb555(23, 30, 31));
    }

    fn draw_cards(&self, fb: &mut Framebuffer, frame: u64) {
        let start_y = 58;
        let card_h = 24;

        for (i, title) in TITLES.iter().enumerate() {
            let y0 = start_y + (i as i32 * (card_h + 6));
            let y1 = y0 + card_h;
            let selected = i == self.selected;
            let bg = if selected {
                rgb555(8, 16, 24)
            } else {
                rgb555(3, 7, 12)
            };
            let border = if selected {
                rgb555(20, 27, 31)
            } else {
                rgb555(8, 12, 18)
            };

            self.fill_rect(fb, 20, y0, (FB_W - 20) as i32, y1, border);
            self.fill_rect(fb, 22, y0 + 2, (FB_W - 22) as i32, y1 - 2, bg);

            // Placeholder cover art block.
            let cover_x0 = 28;
            let cover_x1 = 56;
            let shimmer = (((frame / 6) as i32 + i as i32 * 3) & 7) as u8;
            let cover = rgb555(10 + shimmer, 8 + shimmer / 2, 18 + shimmer);
            self.fill_rect(fb, cover_x0, y0 + 4, cover_x1, y1 - 4, cover);

            self.draw_text(fb, title, 64, y0 + 8, 2, rgb555(26, 30, 31));

            if selected {
                self.draw_text(fb, ">", 10, y0 + 8, 2, rgb555(28, 24, 12));
            }
        }
    }

    fn draw_footer(&self, fb: &mut Framebuffer) {
        self.fill_rect(fb, 16, 216, (FB_W - 16) as i32, 236, rgb555(2, 8, 14));
        self.draw_text(
            fb,
            "UP/DOWN: SELECT   A/START: OPEN (COMING SOON)",
            24,
            222,
            1,
            rgb555(18, 24, 28),
        );
    }

    fn fill_rect(&self, fb: &mut Framebuffer, x0: i32, y0: i32, x1: i32, y1: i32, color: u16) {
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

    fn draw_text(
        &self,
        fb: &mut Framebuffer,
        text: &str,
        x: i32,
        y: i32,
        scale: usize,
        color: u16,
    ) {
        let mut cx = x;
        for ch in text.chars() {
            self.draw_glyph(fb, ch, cx, y, scale, color);
            cx += (6 * scale) as i32;
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
        '6' => [0x0E, 0x10, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '+' => [0x00, 0x04, 0x04, 0x1F, 0x04, 0x04, 0x00],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        ':' => [0x00, 0x04, 0x00, 0x00, 0x00, 0x04, 0x00],
        '/' => [0x01, 0x01, 0x02, 0x04, 0x08, 0x10, 0x10],
        '>' => [0x10, 0x08, 0x04, 0x02, 0x04, 0x08, 0x10],
        '(' => [0x02, 0x04, 0x08, 0x08, 0x08, 0x04, 0x02],
        ')' => [0x08, 0x04, 0x02, 0x02, 0x02, 0x04, 0x08],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        ' ' => [0x00; 7],
        _ => [0x1F, 0x11, 0x15, 0x15, 0x15, 0x11, 0x1F],
    }
}
