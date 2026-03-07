mod aurex;

use aurex::game::InputState;
use aurex::ppu::framebuffer::{FB_H, FB_W};
use aurex::runtime::{AudioEngine, AudioMode, FlowController, FlowPhase};
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

    let mut synth = AudioEngine::new(queue.spec().freq as u32);
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

        // Avoid `pressed_scancodes()` iteration because some SDL stacks can report
        // out-of-range scancode values that trigger enum-conversion panics in rust-sdl2.
        // We intentionally poll a curated list of common start keys instead.
        let start_key_pressed = kb.is_scancode_pressed(Scancode::Return)
            || kb.is_scancode_pressed(Scancode::Space)
            || kb.is_scancode_pressed(Scancode::LShift)
            || kb.is_scancode_pressed(Scancode::RShift)
            || kb.is_scancode_pressed(Scancode::LCtrl)
            || kb.is_scancode_pressed(Scancode::RCtrl)
            || kb.is_scancode_pressed(Scancode::Tab)
            || kb.is_scancode_pressed(Scancode::Up)
            || kb.is_scancode_pressed(Scancode::Down)
            || kb.is_scancode_pressed(Scancode::Left)
            || kb.is_scancode_pressed(Scancode::Right)
            || kb.is_scancode_pressed(Scancode::W)
            || kb.is_scancode_pressed(Scancode::A)
            || kb.is_scancode_pressed(Scancode::S)
            || kb.is_scancode_pressed(Scancode::D)
            || kb.is_scancode_pressed(Scancode::Z)
            || kb.is_scancode_pressed(Scancode::X);
        if start_key_pressed && flow.register_start_request() {
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
