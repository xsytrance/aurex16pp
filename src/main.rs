mod aurex;

use aurex::game::{AudioCue, InputState};
use aurex::ppu::framebuffer::{FB_H, FB_W};
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
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

struct BootSynth {
    sample_clock: u64,
    bass_phase: u32,
    lead_phase: u32,
    sample_rate: u32,
}

impl BootSynth {
    fn new(sample_rate: u32) -> Self {
        Self {
            sample_clock: 0,
            bass_phase: 0,
            lead_phase: 0,
            sample_rate,
        }
    }

    fn step_from_hz(&self, hz: u32) -> u32 {
        (((hz as u64) << 32) / self.sample_rate as u64) as u32
    }

    fn render_block(&mut self, out: &mut [i16]) {
        // Original synth line: late-90s robotic trance groove.
        const BPM: u32 = 132;
        const BASS_PATTERN: [u32; 16] = [
            55, 55, 55, 55, 73, 73, 73, 73, 65, 65, 65, 65, 49, 49, 49, 49,
        ];
        const LEAD_PATTERN: [u32; 16] = [
            220, 0, 247, 0, 220, 0, 294, 0, 220, 0, 247, 0, 330, 0, 294, 0,
        ];

        let samples_per_beat = (self.sample_rate * 60) / BPM;

        for s in out.iter_mut() {
            let beat = (self.sample_clock / samples_per_beat as u64) as usize;
            let step = beat % 16;
            let sub = (self.sample_clock % samples_per_beat as u64) as u32;

            let bass_hz = BASS_PATTERN[step];
            let lead_hz = LEAD_PATTERN[step];

            let bass_step = self.step_from_hz(bass_hz);
            self.bass_phase = self.bass_phase.wrapping_add(bass_step);
            let bass_wave = if self.bass_phase < 0x8000_0000 {
                9000i32
            } else {
                -9000i32
            };

            let lead_wave = if lead_hz == 0 {
                0
            } else {
                let lead_step = self.step_from_hz(lead_hz);
                self.lead_phase = self.lead_phase.wrapping_add(lead_step);

                let pulse = if (self.lead_phase >> 28) < 5 {
                    7000i32
                } else {
                    -3000i32
                };
                let gate = if sub < samples_per_beat / 2 { 1 } else { 0 };
                pulse * gate
            };

            // Kick on every beat with fast decay.
            let kick_env = (samples_per_beat.saturating_sub(sub)).min(samples_per_beat / 5) as i32;
            let kick = (kick_env * 6) - 5000;

            let mix =
                ((bass_wave + lead_wave + kick) / 2).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            *s = mix;
            self.sample_clock = self.sample_clock.wrapping_add(1);
        }
    }
}

fn main() {
    let sdl = sdl2::init().expect("SDL init failed");
    let video = sdl.video().expect("SDL video init failed");
    let audio = sdl.audio().expect("SDL audio init failed");

    let desired = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1),
        samples: Some(1024),
    };

    let queue = audio
        .open_queue::<i16, _>(None, &desired)
        .expect("audio queue open failed");

    let mut synth = BootSynth::new(queue.spec().freq as u32);
    queue.resume();

    let scale: u32 = 3;
    let win_w = (FB_W as u32) * scale;
    let win_h = (FB_H as u32) * scale;

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
    let mut system = aurex::Aurex::new();
    let mut flow = FlowController::new();

    let target = Duration::from_nanos(16_666_667);
    let mut last = Instant::now();

    'running: loop {
        for e in pump.poll_iter() {
            match e {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown { .. } => {
                    if flow.register_start_request() {
                        synth.trigger_confirm();
                    }
                }
                Event::ControllerButtonDown { .. } => {
                    if flow.register_start_request() {
                        synth.trigger_confirm();
                    }
                }
                Event::ControllerDeviceAdded { which, .. } => {
                    if controller.is_none() && game_controller.is_game_controller(which) {
                        if let Ok(c) = game_controller.open(which) {
                            println!("Controller connected: {}", c.name());
                            controller = Some(c);
                        }
                    }
                }
                Event::ControllerDeviceRemoved { which, .. } => {
                    if let Some(c) = &controller {
                        if c.instance_id() == which {
                            println!("Controller disconnected");
                            controller = None;
                        }
                    }
                }
                _ => {}
            }
        }

        // Keep a short audio queue full.
        if queue.size() < 16_384 {
            let mut block = [0i16; 2048];
            synth.render_block(&mut block);
            let _ = queue.queue_audio(&block);
        }

        system.run_frame();
        let src = system.framebuffer().pixels();

        let kb = pump.keyboard_state();
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
                        out[o + 0] = (argb & 0xFF) as u8;
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
