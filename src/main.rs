mod aurex;

use aurex::game::InputState;
use aurex::ppu::framebuffer::{FB_H, FB_W};
use aurex::runtime::{
    AudioEngine, AudioMode, FlowController, FlowPhase, poll_input, present_frame,
};
use sdl2::audio::AudioSpecDesired;
use sdl2::controller::GameController;
use std::time::{Duration, Instant};

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

        let polled = poll_input(&pump, controller.as_ref(), flow.game_active());

        if polled.quit_requested {
            break 'running;
        }

        if polled.start_pressed && flow.register_start_request() {
            synth.trigger_confirm();
        }

        let input = polled.gameplay;

        system.run_frame(input);
        synth.trigger_cue(system.take_audio_cue());

        let src = system.framebuffer().pixels();
        present_frame(&mut canvas, &mut texture, src).expect("present frame failed");

        let elapsed = last.elapsed();
        if elapsed < target {
            std::thread::sleep(target - elapsed);
        }
        last = Instant::now();
    }
}
