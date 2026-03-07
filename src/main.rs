mod aurex;

use aurex::ppu::framebuffer::{FB_H, FB_W};
use aurex::runtime::{
    AudioEngine, AudioMode, FlowController, FlowPhase, FramePacer, collect_runtime_diagnostics,
    dispatch_runtime_events, poll_input, present_frame,
};
use sdl2::GameControllerSubsystem;
use sdl2::audio::AudioSpecDesired;
use sdl2::controller::GameController;
use std::time::Duration;

fn open_first_controller(game_controller: &GameControllerSubsystem) -> Option<GameController> {
    for id in 0..game_controller.num_joysticks().unwrap_or(0) {
        if !game_controller.is_game_controller(id) {
            continue;
        }

        if let Ok(c) = game_controller.open(id) {
            println!("Controller connected: {}", c.name());
            return Some(c);
        }
    }
    None
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
    let mut controller = open_first_controller(&game_controller);

    let mut system = aurex::Aurex::new();
    let mut flow = FlowController::new();

    let mut pacer = FramePacer::new(Duration::from_nanos(16_666_667));
    let mut runtime_events = Vec::with_capacity(8);

    'running: loop {
        pump.pump_events();

        let controller_missing_or_detached = match controller.as_ref() {
            None => true,
            Some(c) => !c.attached(),
        };

        if controller_missing_or_detached {
            controller = open_first_controller(&game_controller);
        }

        let polled = poll_input(&pump, controller.as_ref(), flow.game_active());

        if polled.quit_requested {
            break 'running;
        }

        if flow.tick(polled.start_pressed) {
            system.start_game();
            println!("Library ready");
        }

        system.set_boot_waiting_for_start(flow.waiting_for_start());

        let audio_mode = match flow.phase() {
            FlowPhase::Boot | FlowPhase::AwaitStart => AudioMode::Boot,
            FlowPhase::Game => AudioMode::Game,
        };

        if queue.size() < 16_384 {
            let mut block = [0i16; 2048];
            synth.render_block(audio_mode, &mut block);
            let _ = queue.queue_audio(&block);
        }

        let input = polled.gameplay;

        system.run_frame(input);
        runtime_events.clear();
        system.drain_events(&mut runtime_events);

        let diagnostics = collect_runtime_diagnostics(&runtime_events);
        if let Some(scene) = diagnostics.scene_changed {
            println!("Scene changed: {:?}", scene);
        }
        if let Some(title) = diagnostics.launch_requested {
            println!("Launch requested: {title}");
        }

        dispatch_runtime_events(&mut synth, &runtime_events);

        let src = system.framebuffer().pixels();
        present_frame(&mut canvas, &mut texture, src).expect("present frame failed");

        pacer.wait_next_frame();
    }
}
