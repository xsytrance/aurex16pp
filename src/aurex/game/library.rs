use crate::aurex::game::{AudioCue, InputState};
use crate::aurex::ppu::framebuffer::{FB_H, FB_W, Framebuffer, rgb555};

#[derive(Clone, Copy)]
struct ColorTheme {
    bg_r: u8,
    bg_g: u8,
    bg_b: u8,
    cover_r: u8,
    cover_g: u8,
    cover_b: u8,
}

#[derive(Clone, Copy)]
enum IconKind {
    Circuit,
    Skyline,
    Sentinel,
    Map,
    Mecha,
    Orbit,
}

#[derive(Clone, Copy)]
struct TitleProfile {
    title: &'static str,
    cartridge_id: &'static str,
    track_id: u8,
    theme: ColorTheme,
    icon: IconKind,
}

const PROFILES: [TitleProfile; 6] = [
    TitleProfile {
        title: "NEON CIRCUIT",
        cartridge_id: "neon_circuit",
        track_id: 0,
        theme: ColorTheme {
            bg_r: 1,
            bg_g: 7,
            bg_b: 14,
            cover_r: 9,
            cover_g: 24,
            cover_b: 27,
        },
        icon: IconKind::Circuit,
    },
    TitleProfile {
        title: "SKYLINE DRIFT",
        cartridge_id: "skyline_drift",
        track_id: 1,
        theme: ColorTheme {
            bg_r: 2,
            bg_g: 5,
            bg_b: 16,
            cover_r: 12,
            cover_g: 16,
            cover_b: 30,
        },
        icon: IconKind::Skyline,
    },
    TitleProfile {
        title: "PIXEL SENTINEL",
        cartridge_id: "pixel_sentinel",
        track_id: 2,
        theme: ColorTheme {
            bg_r: 2,
            bg_g: 8,
            bg_b: 10,
            cover_r: 10,
            cover_g: 24,
            cover_b: 14,
        },
        icon: IconKind::Sentinel,
    },
    TitleProfile {
        title: "VOID CARTOGRAPHER",
        cartridge_id: "void_cartographer",
        track_id: 3,
        theme: ColorTheme {
            bg_r: 3,
            bg_g: 3,
            bg_b: 12,
            cover_r: 18,
            cover_g: 10,
            cover_b: 28,
        },
        icon: IconKind::Map,
    },
    TitleProfile {
        title: "MECHA RIFT",
        cartridge_id: "mecha_rift",
        track_id: 4,
        theme: ColorTheme {
            bg_r: 5,
            bg_g: 4,
            bg_b: 8,
            cover_r: 28,
            cover_g: 10,
            cover_b: 10,
        },
        icon: IconKind::Mecha,
    },
    TitleProfile {
        title: "ORBITAL RUSH",
        cartridge_id: "orbital_rush",
        track_id: 5,
        theme: ColorTheme {
            bg_r: 2,
            bg_g: 7,
            bg_b: 6,
            cover_r: 20,
            cover_g: 28,
            cover_b: 10,
        },
        icon: IconKind::Orbit,
    },
];

pub struct LibraryScreen {
    selected: usize,
    prev_up: bool,
    prev_down: bool,
    prev_accept: bool,
    prev_cancel: bool,
    launch_flash_frames: u8,
    launch_pending: bool,
    status_message: StatusMessage,
}

#[derive(Clone, Copy)]
struct StatusMessage {
    text: &'static str,
    tint: ColorTheme,
}

impl StatusMessage {
    fn idle() -> Self {
        Self {
            text: "SONG PROFILE ACTIVE",
            tint: ColorTheme {
                bg_r: 2,
                bg_g: 8,
                bg_b: 14,
                cover_r: 14,
                cover_g: 22,
                cover_b: 28,
            },
        }
    }
}

pub struct LibraryUpdate {
    pub audio_cue: AudioCue,
    pub launch_requested: bool,
    pub launch_canceled: bool,
}

impl LibraryScreen {
    pub fn new() -> Self {
        Self {
            selected: 0,
            prev_up: false,
            prev_down: false,
            prev_accept: false,
            prev_cancel: false,
            launch_flash_frames: 0,
            launch_pending: false,
            status_message: StatusMessage::idle(),
        }
    }

    pub fn current_audio_cue(&self) -> AudioCue {
        AudioCue::SelectTrack(PROFILES[self.selected].track_id)
    }

    pub fn current_title(&self) -> &'static str {
        PROFILES[self.selected].title
    }

    pub fn current_launch_descriptor(&self) -> crate::aurex::runtime::LaunchDescriptor {
        let p = PROFILES[self.selected];
        crate::aurex::runtime::LaunchDescriptor {
            title: p.title,
            cartridge_id: p.cartridge_id,
        }
    }

    pub fn set_launch_pending(&mut self, pending: bool) {
        self.launch_pending = pending;
        if pending {
            self.status_message = StatusMessage {
                text: "LAUNCH INTENT ARMED",
                tint: PROFILES[self.selected].theme,
            };
        } else if self.launch_flash_frames == 0 {
            self.status_message = StatusMessage::idle();
        }
    }

    pub fn update(&mut self, input: InputState) -> LibraryUpdate {
        if self.launch_flash_frames > 0 {
            self.launch_flash_frames -= 1;
        }

        let mut cue = AudioCue::None;
        let mut launch_requested = false;
        let mut launch_canceled = false;

        if input.up && !self.prev_up {
            self.selected = (self.selected + PROFILES.len() - 1) % PROFILES.len();
            cue = self.current_audio_cue();
            if !self.launch_pending {
                self.status_message = StatusMessage::idle();
            }
        }
        if input.down && !self.prev_down {
            self.selected = (self.selected + 1) % PROFILES.len();
            cue = self.current_audio_cue();
            if !self.launch_pending {
                self.status_message = StatusMessage::idle();
            }
        }

        if input.accept && !self.prev_accept {
            launch_requested = true;
            cue = AudioCue::LaunchRequest;
            self.launch_flash_frames = 42;
            self.status_message = StatusMessage {
                text: "CARTRIDGE LAUNCH PIPELINE NEXT",
                tint: PROFILES[self.selected].theme,
            };
        }

        if input.cancel && !self.prev_cancel {
            launch_canceled = true;
            cue = AudioCue::Cancel;
            self.launch_flash_frames = 0;
            if !self.launch_pending {
                self.status_message = StatusMessage::idle();
            }
        }

        self.prev_up = input.up;
        self.prev_down = input.down;
        self.prev_accept = input.accept;
        self.prev_cancel = input.cancel;

        LibraryUpdate {
            audio_cue: cue,
            launch_requested,
            launch_canceled,
        }
    }

    pub fn draw(&self, fb: &mut Framebuffer, frame: u64) {
        let profile = PROFILES[self.selected];
        self.draw_backdrop(fb, frame, profile.theme);
        self.draw_header(fb);
        self.draw_cards(fb, frame);
        self.draw_footer(fb, profile, frame);
    }

    fn draw_backdrop(&self, fb: &mut Framebuffer, frame: u64, theme: ColorTheme) {
        let pixels = fb.pixels_mut();
        for y in 0..FB_H {
            for x in 0..FB_W {
                let wave = (((x as u64 + frame) >> 4) as i32
                    - ((y as u64 + frame * 2) >> 5) as i32)
                    & 0x07;
                let b =
                    (theme.bg_b as i32 + (y as i32 * 6 / FB_H as i32) + wave).clamp(0, 31) as u8;
                let g = (theme.bg_g as i32 + wave / 2).clamp(0, 31) as u8;
                let color = rgb555(theme.bg_r, g, b);
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

        for (i, p) in PROFILES.iter().enumerate() {
            let y0 = start_y + (i as i32 * (card_h + 6));
            let y1 = y0 + card_h;
            let selected = i == self.selected;
            let bg = if selected {
                rgb555(8, 16, 24)
            } else {
                rgb555(3, 7, 12)
            };
            let border = if selected {
                rgb555(
                    p.theme.cover_r.min(31),
                    p.theme.cover_g.min(31),
                    p.theme.cover_b.min(31),
                )
            } else {
                rgb555(8, 12, 18)
            };

            self.fill_rect(fb, 20, y0, (FB_W - 20) as i32, y1, border);
            self.fill_rect(fb, 22, y0 + 2, (FB_W - 22) as i32, y1 - 2, bg);

            let cover_x0 = 28;
            let cover_x1 = 56;
            let shimmer = (((frame / 6) as i32 + i as i32 * 3) & 7) as u8;
            let cover = rgb555(
                (p.theme.cover_r + shimmer).min(31),
                (p.theme.cover_g + shimmer / 2).min(31),
                (p.theme.cover_b + shimmer).min(31),
            );
            self.fill_rect(fb, cover_x0, y0 + 4, cover_x1, y1 - 4, cover);
            self.draw_icon(fb, p.icon, cover_x0 + 6, y0 + 7, rgb555(0, 0, 0));

            self.draw_text(fb, p.title, 64, y0 + 8, 2, rgb555(26, 30, 31));

            if selected {
                self.draw_text(fb, ">", 10, y0 + 8, 2, rgb555(28, 24, 12));
            }
        }
    }

    fn draw_icon(&self, fb: &mut Framebuffer, kind: IconKind, x: i32, y: i32, color: u16) {
        match kind {
            IconKind::Circuit => {
                self.fill_rect(fb, x, y + 2, x + 12, y + 4, color);
                self.fill_rect(fb, x + 2, y, x + 4, y + 10, color);
                self.fill_rect(fb, x + 8, y + 4, x + 10, y + 12, color);
            }
            IconKind::Skyline => {
                self.fill_rect(fb, x, y + 7, x + 12, y + 9, color);
                self.fill_rect(fb, x + 1, y + 3, x + 3, y + 7, color);
                self.fill_rect(fb, x + 5, y + 1, x + 7, y + 7, color);
                self.fill_rect(fb, x + 9, y + 4, x + 11, y + 7, color);
            }
            IconKind::Sentinel => {
                self.fill_rect(fb, x + 3, y + 1, x + 9, y + 3, color);
                self.fill_rect(fb, x + 1, y + 3, x + 11, y + 9, color);
                self.fill_rect(fb, x + 4, y + 9, x + 8, y + 11, color);
            }
            IconKind::Map => {
                self.fill_rect(fb, x, y, x + 12, y + 2, color);
                self.fill_rect(fb, x, y + 10, x + 12, y + 12, color);
                self.fill_rect(fb, x, y, x + 2, y + 12, color);
                self.fill_rect(fb, x + 10, y, x + 12, y + 12, color);
                self.fill_rect(fb, x + 5, y + 2, x + 7, y + 10, color);
            }
            IconKind::Mecha => {
                self.fill_rect(fb, x + 2, y + 1, x + 10, y + 3, color);
                self.fill_rect(fb, x + 1, y + 3, x + 11, y + 8, color);
                self.fill_rect(fb, x + 3, y + 8, x + 5, y + 12, color);
                self.fill_rect(fb, x + 7, y + 8, x + 9, y + 12, color);
            }
            IconKind::Orbit => {
                self.fill_rect(fb, x + 4, y + 4, x + 8, y + 8, color);
                self.fill_rect(fb, x + 1, y + 5, x + 3, y + 7, color);
                self.fill_rect(fb, x + 9, y + 5, x + 11, y + 7, color);
                self.fill_rect(fb, x + 5, y + 1, x + 7, y + 3, color);
                self.fill_rect(fb, x + 5, y + 9, x + 7, y + 11, color);
            }
        }
    }

    fn draw_footer(&self, fb: &mut Framebuffer, profile: TitleProfile, frame: u64) {
        self.fill_rect(fb, 16, 216, (FB_W - 16) as i32, 236, rgb555(2, 8, 14));
        self.draw_text(
            fb,
            "UP/DOWN: SELECT   A/START: REQUEST   B/ESC: CLEAR",
            24,
            222,
            1,
            rgb555(18, 24, 28),
        );

        let pulse = if self.launch_flash_frames > 0 {
            (((frame >> 1) & 0x03) as u8) + 2
        } else {
            0
        };

        self.draw_text(
            fb,
            self.status_message.text,
            226,
            222,
            1,
            rgb555(
                (self.status_message.tint.cover_r + pulse).min(31),
                (self.status_message.tint.cover_g + pulse).min(31),
                (self.status_message.tint.cover_b + pulse).min(31),
            ),
        );

        self.draw_audio_meter(fb, profile, frame);

        if self.launch_pending {
            self.draw_text(fb, "PENDING", 358, 198, 1, rgb555(28, 28, 10));
        }
    }

    fn draw_audio_meter(&self, fb: &mut Framebuffer, profile: TitleProfile, frame: u64) {
        let meter_x = 318;
        let meter_y = 196;
        let bars = 9;

        self.fill_rect(
            fb,
            meter_x - 6,
            meter_y - 4,
            meter_x + 72,
            meter_y + 16,
            rgb555(2, 6, 11),
        );

        for bar in 0..bars {
            let wave = (((frame >> 1) as i32 + bar * 3) & 15) - 7;
            let mut h = 3 + wave.abs();
            if self.launch_pending {
                h += 2;
            }
            if bar == (self.selected as i32 + (frame as i32 >> 3)) % bars {
                h += 2;
            }

            let x0 = meter_x + bar * 7;
            let y0 = meter_y + 10 - h;
            let tint = (bar as u8) & 0x03;
            let c = rgb555(
                (profile.theme.cover_r + tint).min(31),
                (profile.theme.cover_g + tint).min(31),
                profile.theme.cover_b.min(31),
            );
            self.fill_rect(fb, x0, y0, x0 + 5, meter_y + 10, c);
        }
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

#[cfg(test)]
mod tests {
    use super::{LibraryScreen, PROFILES};
    use crate::aurex::game::{AudioCue, InputState};

    #[test]
    fn navigation_wraps_and_emits_track_cue() {
        let mut lib = LibraryScreen::new();
        let update_up = lib.update(InputState {
            up: true,
            ..InputState::default()
        });

        assert!(matches!(
            update_up.audio_cue,
            AudioCue::SelectTrack(track) if track == PROFILES[PROFILES.len() - 1].track_id
        ));

        let update_down = lib.update(InputState::default());
        assert!(matches!(update_down.audio_cue, AudioCue::None));

        let update_down = lib.update(InputState {
            down: true,
            ..InputState::default()
        });
        assert!(matches!(
            update_down.audio_cue,
            AudioCue::SelectTrack(track) if track == PROFILES[0].track_id
        ));
    }

    #[test]
    fn launch_request_is_edge_triggered() {
        let mut lib = LibraryScreen::new();

        let first = lib.update(InputState {
            accept: true,
            ..InputState::default()
        });
        assert!(first.launch_requested);
        assert!(matches!(first.audio_cue, AudioCue::LaunchRequest));

        let held = lib.update(InputState {
            accept: true,
            ..InputState::default()
        });
        assert!(!held.launch_requested);
        assert!(!held.launch_canceled);

        let released = lib.update(InputState::default());
        assert!(!released.launch_requested);

        let second = lib.update(InputState {
            accept: true,
            ..InputState::default()
        });
        assert!(second.launch_requested);
    }

    #[test]
    fn cancel_is_edge_triggered_and_resets_state() {
        let mut lib = LibraryScreen::new();

        let _ = lib.update(InputState {
            accept: true,
            ..InputState::default()
        });

        let canceled = lib.update(InputState {
            cancel: true,
            ..InputState::default()
        });
        assert!(canceled.launch_canceled);
        assert!(matches!(canceled.audio_cue, AudioCue::Cancel));

        let held = lib.update(InputState {
            cancel: true,
            ..InputState::default()
        });
        assert!(!held.launch_canceled);
    }
}
