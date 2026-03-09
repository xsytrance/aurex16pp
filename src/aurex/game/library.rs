#![allow(dead_code)]
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
    bpm: u16,
    style: &'static str,
    tag: &'static str,
    theme: ColorTheme,
    icon: IconKind,
}

const PROFILES: [TitleProfile; 7] = [
    TitleProfile {
        title: "NEON CIRCUIT",
        cartridge_id: "neon_circuit",
        track_id: 0,
        bpm: 124,
        style: "BASSLINE GRID",
        tag: "PLASMA STREETS",
        theme: ColorTheme {
            bg_r: 1,
            bg_g: 6,
            bg_b: 12,
            cover_r: 13,
            cover_g: 25,
            cover_b: 31,
        },
        icon: IconKind::Circuit,
    },
    TitleProfile {
        title: "SKYLINE DRIFT",
        cartridge_id: "skyline_drift",
        track_id: 1,
        bpm: 128,
        style: "NIGHT-RIDE SYNTH",
        tag: "MIDNIGHT FREEWAY",
        theme: ColorTheme {
            bg_r: 1,
            bg_g: 4,
            bg_b: 14,
            cover_r: 17,
            cover_g: 14,
            cover_b: 31,
        },
        icon: IconKind::Skyline,
    },
    TitleProfile {
        title: "PIXEL SENTINEL",
        cartridge_id: "pixel_sentinel",
        track_id: 2,
        bpm: 118,
        style: "INDUSTRIAL PULSE",
        tag: "TOWER LOCKDOWN",
        theme: ColorTheme {
            bg_r: 1,
            bg_g: 7,
            bg_b: 8,
            cover_r: 12,
            cover_g: 28,
            cover_b: 16,
        },
        icon: IconKind::Sentinel,
    },
    TitleProfile {
        title: "VOID CARTOGRAPHER",
        cartridge_id: "void_cartographer",
        track_id: 3,
        bpm: 132,
        style: "COSMIC TRANCE",
        tag: "STAR MAP DIVE",
        theme: ColorTheme {
            bg_r: 2,
            bg_g: 2,
            bg_b: 10,
            cover_r: 22,
            cover_g: 12,
            cover_b: 31,
        },
        icon: IconKind::Map,
    },
    TitleProfile {
        title: "MECHA RIFT",
        cartridge_id: "mecha_rift",
        track_id: 4,
        bpm: 136,
        style: "BROKEN BEAT",
        tag: "CHROME COLLISION",
        theme: ColorTheme {
            bg_r: 4,
            bg_g: 3,
            bg_b: 7,
            cover_r: 31,
            cover_g: 11,
            cover_b: 12,
        },
        icon: IconKind::Mecha,
    },
    TitleProfile {
        title: "ORBITAL RUSH",
        cartridge_id: "orbital_rush",
        track_id: 5,
        bpm: 140,
        style: "ZERO-G EDM",
        tag: "GRAVITY BREAK",
        theme: ColorTheme {
            bg_r: 1,
            bg_g: 6,
            bg_b: 5,
            cover_r: 24,
            cover_g: 30,
            cover_b: 11,
        },
        icon: IconKind::Orbit,
    },
    TitleProfile {
        title: "CHROME DUO BOOT",
        cartridge_id: "chrome_duo_boot",
        track_id: 0,
        bpm: 126,
        style: "FILTER DISCO DRIVE",
        tag: "ROBOTIC NIGHT RUN",
        theme: ColorTheme {
            bg_r: 2,
            bg_g: 2,
            bg_b: 9,
            cover_r: 27,
            cover_g: 29,
            cover_b: 12,
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
            text: "PROFILE READY // EDM BANK ARMED",
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

    pub fn set_launch_stage(&mut self, stage: crate::aurex::runtime::LaunchStage) {
        self.launch_pending = !matches!(stage, crate::aurex::runtime::LaunchStage::Idle);

        self.status_message = match stage {
            crate::aurex::runtime::LaunchStage::Idle => StatusMessage::idle(),
            crate::aurex::runtime::LaunchStage::Pending(_) => StatusMessage {
                text: "LAUNCH INTENT ARMED",
                tint: PROFILES[self.selected].theme,
            },
            crate::aurex::runtime::LaunchStage::Validating(_) => StatusMessage {
                text: "VALIDATING CARTRIDGE",
                tint: PROFILES[self.selected].theme,
            },
            crate::aurex::runtime::LaunchStage::Ready(_) => StatusMessage {
                text: "CARTRIDGE READY",
                tint: PROFILES[self.selected].theme,
            },
            crate::aurex::runtime::LaunchStage::Rejected(_) => StatusMessage {
                text: "CARTRIDGE REJECTED",
                tint: ColorTheme {
                    bg_r: 2,
                    bg_g: 3,
                    bg_b: 6,
                    cover_r: 29,
                    cover_g: 9,
                    cover_b: 9,
                },
            },
        };
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
        self.draw_header(fb, profile);
        self.draw_cards(fb, frame);
        self.draw_footer(fb, profile, frame);
    }

    fn draw_backdrop(&self, fb: &mut Framebuffer, frame: u64, theme: ColorTheme) {
        let pixels = fb.pixels_mut();
        for y in 0..FB_H {
            for x in 0..FB_W {
                let phase = ((x as u64 * 3 + frame * 4) ^ (y as u64 * 5 + frame * 2)) & 31;
                let cross = (((x as i32 - 200).abs() + (y as i32 - 120).abs()) as u64 + frame) & 15;
                let b = (theme.bg_b as i32 + (phase as i32 / 2) + (cross as i32 / 2)).clamp(0, 31)
                    as u8;
                let g = (theme.bg_g as i32 + (phase as i32 / 3)).clamp(0, 31) as u8;
                let r =
                    (theme.bg_r as i32 + (((x as u64 + frame) >> 7) as i32 & 1)).clamp(0, 31) as u8;
                pixels[y * FB_W + x] = rgb555(r, g, b);
            }
        }
    }

    fn draw_header(&self, fb: &mut Framebuffer, profile: TitleProfile) {
        self.fill_rect(fb, 12, 10, (FB_W - 12) as i32, 46, rgb555(2, 8, 14));
        self.draw_text(fb, "AUREX-16++ DEEP LIBRARY", 22, 16, 2, rgb555(23, 30, 31));
        self.draw_text(fb, "HI-FI SYNTH MODE", 24, 32, 1, rgb555(19, 26, 31));
        self.draw_text(fb, profile.style, 150, 32, 1, rgb555(18, 24, 30));
        self.draw_text(fb, profile.tag, 300, 32, 1, rgb555(20, 28, 31));
    }

    fn draw_cards(&self, fb: &mut Framebuffer, frame: u64) {
        let start_y = 54;
        let card_h = 25;

        for (i, p) in PROFILES.iter().enumerate() {
            let y0 = start_y + (i as i32 * (card_h + 5));
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

            self.fill_rect(fb, 16, y0, (FB_W - 16) as i32, y1, border);
            self.fill_rect(fb, 18, y0 + 2, (FB_W - 18) as i32, y1 - 2, bg);

            let cover_x0 = 24;
            let cover_x1 = 58;
            let shimmer = (((frame / 6) as i32 + i as i32 * 3) & 7) as u8;
            let cover = rgb555(
                (p.theme.cover_r + shimmer).min(31),
                (p.theme.cover_g + shimmer / 2).min(31),
                (p.theme.cover_b + shimmer).min(31),
            );
            self.fill_rect(fb, cover_x0, y0 + 3, cover_x1, y1 - 3, cover);
            self.draw_icon(fb, p.icon, cover_x0 + 10, y0 + 7, rgb555(0, 0, 0));

            self.draw_text(fb, p.title, 66, y0 + 6, 2, rgb555(26, 30, 31));
            self.draw_text(fb, p.style, 234, y0 + 7, 1, rgb555(16, 24, 28));

            let pulse_h = ((frame as i32 / 3 + i as i32 * 5) & 7) + 2;
            self.fill_rect(fb, 348, y1 - pulse_h, 372, y1 - 2, rgb555(10, 22, 29));
            self.fill_rect(fb, 376, y1 - pulse_h + 1, 398, y1 - 2, rgb555(18, 26, 31));

            if selected {
                self.draw_text(fb, ">", 6, y0 + 8, 2, rgb555(28, 24, 12));
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
        self.fill_rect(fb, 12, 215, (FB_W - 12) as i32, 238, rgb555(1, 7, 12));
        self.draw_text(
            fb,
            "UP/DOWN: SELECT   A/START: REQUEST   B/ESC: CLEAR",
            18,
            218,
            1,
            rgb555(19, 26, 30),
        );

        let pulse = if self.launch_flash_frames > 0 {
            (((frame >> 1) & 0x03) as u8) + 2
        } else {
            0
        };

        self.draw_text(
            fb,
            self.status_message.text,
            210,
            218,
            1,
            rgb555(
                (self.status_message.tint.cover_r + pulse).min(31),
                (self.status_message.tint.cover_g + pulse).min(31),
                (self.status_message.tint.cover_b + pulse).min(31),
            ),
        );

        let meter_x = 300;
        let meter_y = 188;
        self.fill_rect(
            fb,
            meter_x,
            meter_y - 2,
            meter_x + 98,
            meter_y + 19,
            rgb555(1, 4, 8),
        );

        for bar in 0..12i32 {
            let bpm_wave = ((frame as i32 / 2) + bar * 3 + profile.bpm as i32 / 8) & 7;
            let h = 2 + bpm_wave;
            let x0 = meter_x + 4 + bar * 7;
            let y0 = meter_y + 15 - h;
            let tint = (bar as u8) & 0x03;
            let c = rgb555(
                (profile.theme.cover_r + tint).min(31),
                (profile.theme.cover_g + tint).min(31),
                profile.theme.cover_b.min(31),
            );
            self.fill_rect(fb, x0, y0, x0 + 5, meter_y + 15, c);
        }

        let bpm_label = format!("{} BPM", profile.bpm);
        self.draw_text(fb, &bpm_label, 302, 177, 1, rgb555(20, 28, 31));
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
    use crate::aurex::runtime::AUDIO_TRACK_COUNT;

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
    fn includes_chrome_duo_boot_profile() {
        assert!(PROFILES
            .iter()
            .any(|p| p.cartridge_id == "chrome_duo_boot" && p.title == "CHROME DUO BOOT"));
    }

    #[test]
    fn all_profiles_map_to_valid_runtime_track_ids() {
        assert!(
            PROFILES
                .iter()
                .all(|profile| (profile.track_id as usize) < AUDIO_TRACK_COUNT),
            "library profile references out-of-range runtime track id"
        );
    }

    #[test]
    fn includes_chrome_duo_boot_profile() {
        assert!(
            PROFILES
                .iter()
                .any(|p| p.cartridge_id == "chrome_duo_boot" && p.title == "CHROME DUO BOOT")
        );
    }

    #[test]
    fn all_profiles_map_to_valid_runtime_track_ids() {
        assert!(
            PROFILES
                .iter()
                .all(|profile| (profile.track_id as usize) < AUDIO_TRACK_COUNT),
            "library profile references out-of-range runtime track id"
        );
    }

    #[test]
    fn includes_chrome_duo_boot_profile() {
        assert!(
            PROFILES
                .iter()
                .any(|p| p.cartridge_id == "chrome_duo_boot" && p.title == "CHROME DUO BOOT")
        );
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
