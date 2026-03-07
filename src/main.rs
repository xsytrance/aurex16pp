mod aurex;

use aurex::game::{AudioCue, InputState};
use aurex::ppu::framebuffer::{FB_H, FB_W};
use aurex::runtime::{FlowController, FlowPhase};
use sdl2::audio::AudioSpecDesired;
use sdl2::controller::{Axis, Button, GameController};
use sdl2::keyboard::Scancode;
use std::time::{Duration, Instant};

fn rgb555_to_argb8888(c: u16) -> u32 {
    let r5 = ((c >> 10) & 0x1F) as u32;
    let g5 = ((c >> 5) & 0x1F) as u32;
    let b5 = (c & 0x1F) as u32;

    let r8 = (r5 << 3) | (r5 >> 2);
    let g8 = (g5 << 3) | (g5 >> 2);
    let b8 = (b5 << 3) | (b5 >> 2);

    (0xFF << 24) | (r8 << 16) | (g8 << 8) | b8
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AudioMode {
    Boot,
    Confirm,
    Game,
}

struct RetroSynth {
    sample_clock: u64,
    bass_phase: u32,
    lead_phase: u32,
    sample_rate: u32,
    confirm_samples_left: u32,
    eat_samples_left: u32,
    fail_samples_left: u32,
    noise_state: u32,
}

impl RetroSynth {
    fn new(sample_rate: u32) -> Self {
        Self {
            sample_clock: 0,
            bass_phase: 0,
            lead_phase: 0,
            sample_rate,
            confirm_samples_left: 0,
            eat_samples_left: 0,
            fail_samples_left: 0,
            noise_state: 0xA5A5_1357,
        }
    }

    fn step_from_hz(&self, hz: u32) -> u32 {
        (((hz as u64) << 32) / self.sample_rate as u64) as u32
    }

    fn trigger_confirm(&mut self) {
        self.confirm_samples_left = self.sample_rate / 3;
    }

    fn trigger_cue(&mut self, cue: AudioCue) {
        match cue {
            AudioCue::Eat => self.eat_samples_left = self.sample_rate / 9,
            AudioCue::Fail => self.fail_samples_left = self.sample_rate / 4,
            AudioCue::None => {}
        }
    }

    fn render_block(&mut self, mode: AudioMode, out: &mut [i16]) {
        for s in out.iter_mut() {
            let music = match mode {
                AudioMode::Boot => self.boot_sample(),
                AudioMode::Confirm => 0,
                AudioMode::Game => self.game_sample(),
            };

            let sfx = self.sfx_sample();
            *s = (music + sfx).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            self.sample_clock = self.sample_clock.wrapping_add(1);
        }
    }

    fn boot_sample(&mut self) -> i32 {
        const BPM: u32 = 132;
        const BASS: [u32; 16] = [
            55, 55, 55, 55, 73, 73, 73, 73, 65, 65, 65, 65, 49, 49, 49, 49,
        ];
        const LEAD: [u32; 16] = [
            220, 0, 247, 0, 220, 0, 294, 0, 220, 0, 247, 0, 330, 0, 294, 0,
        ];
        self.pattern_sample(BPM, &BASS, &LEAD, 9000, 6500)
    }

    fn game_sample(&mut self) -> i32 {
        const BPM: u32 = 148;
        const BASS: [u32; 16] = [
            82, 82, 110, 82, 98, 98, 123, 98, 82, 82, 110, 82, 73, 73, 98, 73,
        ];
        const LEAD: [u32; 16] = [
            330, 392, 440, 392, 349, 392, 523, 392, 330, 392, 440, 392, 294, 330, 392, 330,
        ];
        self.pattern_sample(BPM, &BASS, &LEAD, 7000, 5000)
    }

    fn pattern_sample(
        &mut self,
        bpm: u32,
        bass: &[u32; 16],
        lead: &[u32; 16],
        bass_amp: i32,
        lead_amp: i32,
    ) -> i32 {
        let spb = (self.sample_rate * 60) / bpm;
        let beat = (self.sample_clock / spb as u64) as usize;
        let step = beat % 16;
        let sub = (self.sample_clock % spb as u64) as u32;

        self.bass_phase = self.bass_phase.wrapping_add(self.step_from_hz(bass[step]));
        let bass_wave = if self.bass_phase < 0x8000_0000 {
            bass_amp
        } else {
            -bass_amp
        };

        let lead_hz = lead[step];
        let lead_wave = if lead_hz == 0 {
            0
        } else {
            self.lead_phase = self.lead_phase.wrapping_add(self.step_from_hz(lead_hz));
            let pulse = if (self.lead_phase >> 28) < 6 {
                lead_amp
            } else {
                -(lead_amp / 3)
            };
            if sub < spb / 2 { pulse } else { pulse / 2 }
        };

        let kick_env = (spb.saturating_sub(sub)).min(spb / 6) as i32;
        let kick = (kick_env * 6) - 3500;

        // Deterministic hi-hat/noise pulse for extra texture.
        let hat_window = spb / 8;
        let hat = if sub < hat_window || (sub > spb / 2 && sub < (spb / 2 + hat_window / 2)) {
            self.noise_state = self
                .noise_state
                .wrapping_mul(1664525)
                .wrapping_add(1013904223);
            ((self.noise_state >> 24) as i8 as i32) * 38
        } else {
            0
        };

        (bass_wave + lead_wave + kick + hat) / 2
    }

    fn sfx_sample(&mut self) -> i32 {
        if self.confirm_samples_left > 0 {
            let t = self.sample_rate / 3 - self.confirm_samples_left;
            let hz = 700 + (t * 900 / (self.sample_rate / 3).max(1));
            self.confirm_samples_left -= 1;
            self.lead_phase = self.lead_phase.wrapping_add(self.step_from_hz(hz));
            return if self.lead_phase < 0x8000_0000 {
                9000
            } else {
                -9000
            };
        }

        if self.fail_samples_left > 0 {
            let t = self.sample_rate / 4 - self.fail_samples_left;
            let hz = 420_u32.saturating_sub(t / 180);
            self.fail_samples_left -= 1;
            self.bass_phase = self.bass_phase.wrapping_add(self.step_from_hz(hz.max(90)));
            return if self.bass_phase < 0x8000_0000 {
                11000
            } else {
                -11000
            };
        }

        if self.eat_samples_left > 0 {
            let t = self.sample_rate / 9 - self.eat_samples_left;
            let hz = 1800_u32.saturating_sub(t * 6).max(500);
            self.eat_samples_left -= 1;
            self.lead_phase = self.lead_phase.wrapping_add(self.step_from_hz(hz));
            return if self.lead_phase < 0x8000_0000 {
                7000
            } else {
                -2000
            };
        }

        0
    }
}

fn main() {
    let sdl = sdl2::init().expect("SDL init failed");
    let video = sdl.video().expect("SDL video init failed");
    let audio = sdl.audio().expect("SDL audio init failed");
    let game_controller = sdl
        .game_controller()
        .expect("SDL game controller init failed");

    let desired = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1),
        samples: Some(1024),
    };

    let queue = audio
        .open_queue::<i16, _>(None, &desired)
        .expect("audio queue open failed");

    let mut synth = RetroSynth::new(queue.spec().freq as u32);
    queue.resume();

    let scale: u32 = 3;
    let window = video
        .window("Aurex-16++", (FB_W as u32) * scale, (FB_H as u32) * scale)
        .position_centered()
        .build()
        .expect("window build failed");

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("canvas build failed");

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(
            sdl2::pixels::PixelFormatEnum::ARGB8888,
            FB_W as u32,
            FB_H as u32,
        )
        .expect("texture create failed");

    let mut pump = sdl.event_pump().expect("event pump failed");
    let mut controller: Option<GameController> = None;
    for id in 0..game_controller.num_joysticks().unwrap_or(0) {
        if !game_controller.is_game_controller(id) {
            continue;
        }
        if let Ok(c) = game_controller.open(id) {
            println!("Controller connected: {}", c.name());
            controller = Some(c);
            break;
        }
    }

    let mut system = aurex::Aurex::new();
    let mut flow = FlowController::new();

    let target = Duration::from_nanos(16_666_667);
    let mut last = Instant::now();

    'running: loop {
        // NOTE: We intentionally avoid SDL event enum decoding here because some
        // controller firmwares/drivers can emit unknown event tags that older
        // rust-sdl2 releases fail to decode safely (panic on invalid enum value).
        // We pump events for SDL internals, then consume keyboard/controller state.
        pump.pump_events();

        if let Some(c) = &controller {
            let lx = c.axis(Axis::LeftX);
            let press = lx < -8_000
                || lx > 8_000
                || c.button(Button::DPadLeft)
                || c.button(Button::DPadRight)
                || c.button(Button::DPadUp)
                || c.button(Button::DPadDown)
                || c.button(Button::A)
                || c.button(Button::B)
                || c.button(Button::X)
                || c.button(Button::Y);

            if press && flow.register_start_request() {
                synth.trigger_confirm();
            }
        }

        if flow.tick() {
            system.start_game();
            println!("Snake demo loaded");
        }

        system.set_boot_confirming(flow.boot_confirming());

        let audio_mode = match flow.phase() {
            FlowPhase::Boot => AudioMode::Boot,
            FlowPhase::Confirming => AudioMode::Confirm,
            FlowPhase::Game => AudioMode::Game,
        };

        if queue.size() < 16_384 {
            let mut block = [0i16; 2048];
            synth.render_block(audio_mode, &mut block);
            let _ = queue.queue_audio(&block);
        }

        let kb = pump.keyboard_state();

        if kb.is_scancode_pressed(Scancode::Escape) {
            break 'running;
        }

        let any_key_pressed = kb.pressed_scancodes().any(|sc| sc != Scancode::Escape);
        if any_key_pressed && flow.register_start_request() {
            synth.trigger_confirm();
        }

        let mut pad_left = false;
        let mut pad_right = false;
        let mut pad_up = false;
        let mut pad_down = false;

        if let Some(c) = &controller {
            let lx = c.axis(Axis::LeftX);
            let ly = c.axis(Axis::LeftY);
            pad_left = lx < -8_000 || c.button(Button::DPadLeft);
            pad_right = lx > 8_000 || c.button(Button::DPadRight);
            pad_up = ly < -8_000 || c.button(Button::DPadUp);
            pad_down = ly > 8_000 || c.button(Button::DPadDown);
        }

        let input = if flow.game_active() {
            InputState {
                left: kb.is_scancode_pressed(Scancode::Left)
                    || kb.is_scancode_pressed(Scancode::A)
                    || pad_left,
                right: kb.is_scancode_pressed(Scancode::Right)
                    || kb.is_scancode_pressed(Scancode::D)
                    || pad_right,
                up: kb.is_scancode_pressed(Scancode::Up)
                    || kb.is_scancode_pressed(Scancode::W)
                    || pad_up,
                down: kb.is_scancode_pressed(Scancode::Down)
                    || kb.is_scancode_pressed(Scancode::S)
                    || pad_down,
            }
        } else {
            InputState::default()
        };

        system.run_frame(input);
        synth.trigger_cue(system.take_audio_cue());

        let src = system.framebuffer().pixels();
        texture
            .with_lock(None, |dst: &mut [u8], pitch: usize| {
                for y in 0..FB_H {
                    let row = &src[y * FB_W..(y + 1) * FB_W];
                    let out = &mut dst[y * pitch..y * pitch + FB_W * 4];
                    for (x, &c) in row.iter().enumerate() {
                        let argb = rgb555_to_argb8888(c);
                        let o = x * 4;
                        out[o] = (argb & 0xFF) as u8;
                        out[o + 1] = ((argb >> 8) & 0xFF) as u8;
                        out[o + 2] = ((argb >> 16) & 0xFF) as u8;
                        out[o + 3] = ((argb >> 24) & 0xFF) as u8;
                    }
                }
            })
            .expect("texture lock failed");

        canvas.clear();
        canvas
            .copy(&texture, None, None)
            .expect("canvas copy failed");
        canvas.present();

        let elapsed = last.elapsed();
        if elapsed < target {
            std::thread::sleep(target - elapsed);
        }
        last = Instant::now();
    }
}
