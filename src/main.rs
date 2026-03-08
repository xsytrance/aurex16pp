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
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--analyze-cartridges") {
        let report = aurex::cartridge::CartridgeRuntime::analyze_default_cartridges();
        if args.iter().any(|a| a == "--json") {
            println!("{}", report.to_json());
        } else {
            println!(
                "Cartridge analyze: {} valid / {} invalid",
                report.valid_count(),
                report.invalid_count()
            );
            for entry in &report.entries {
                if entry.ok {
                    println!(
                        "[OK] {} uploads={} bytes={} palette_bytes={}",
                        entry.cartridge_id,
                        entry.upload_count,
                        entry.total_upload_bytes,
                        entry.palette_upload_bytes
                    );
                } else {
                    println!(
                        "[FAIL] {}: {}",
                        entry.cartridge_id,
                        entry.issue.as_deref().unwrap_or("unknown error")
                    );
                }
            }
        }

        if report.all_valid() {
            return;
        }

        std::process::exit(2);
    }

    if args.iter().any(|a| a == "--audio-diagnostics") {
        let mut frames = 48_000usize;
        for i in 0..args.len().saturating_sub(1) {
            if args[i] == "--frames" {
                if let Ok(v) = args[i + 1].parse::<usize>() {
                    frames = v.max(1);
                }
            }
        }

        let mode = if args.iter().any(|a| a == "--boot") {
            aurex::runtime::AudioMode::Boot
        } else {
            aurex::runtime::AudioMode::Game
        };

        let engine = aurex::runtime::AudioEngine::new(48_000);
        let diag = engine.diagnostics_for_frames(mode, frames);
        if args.iter().any(|a| a == "--json") {
            println!("{}", diag.to_json());
        } else {
            println!(
                "Audio diagnostics frames={} peak_l={} peak_r={} avg_abs_l={} avg_abs_r={}",
                diag.frames, diag.peak_l, diag.peak_r, diag.avg_abs_l, diag.avg_abs_r
            );
        }
        return;
    }

    if args.iter().any(|a| a == "--palette-heatmap") {
        let v = aurex::ppu::vram::Vram::new();
        println!("{}", v.bg0_palette_bank_heatmap_json());
        return;
    }

    if args.iter().any(|a| a == "--replay-capture-smoke") {
        let mut cap = aurex::runtime::ReplayCapture::new();
        let mut system = aurex::Aurex::new();
        let mut events = Vec::with_capacity(8);

        for frame in 0..120u32 {
            let input = aurex::game::InputState {
                up: frame % 2 == 0,
                down: frame % 3 == 0,
                accept: frame % 11 == 0,
                cancel: frame % 17 == 0,
                ..Default::default()
            };

            cap.capture_input(input);
            system.run_frame(input);
            events.clear();
            system.drain_events(&mut events);
            for (i, _event) in events.iter().enumerate() {
                cap.capture_event_tag((frame as u64) << 16 | i as u64);
            }
            cap.capture_framebuffer(system.framebuffer().pixels());
            cap.end_frame();
        }

        println!("{}", cap.summary_json());
        return;
    }

    if args.iter().any(|a| a == "--audit-cartridges") {
        let report = aurex::cartridge::CartridgeRuntime::audit_default_cartridges();
        if args.iter().any(|a| a == "--json") {
            println!("{}", report.to_json());
        } else {
            println!(
                "Cartridge audit: {} valid / {} invalid",
                report.valid_count(),
                report.invalid_count()
            );
            for entry in &report.entries {
                if entry.ok {
                    println!("[OK] {}", entry.cartridge_id);
                } else {
                    println!(
                        "[FAIL] {}: {}",
                        entry.cartridge_id,
                        entry.issue.as_deref().unwrap_or("unknown error")
                    );
                }
            }
        }

        if report.all_valid() {
            return;
        }

        std::process::exit(2);
    }

    let sdl = sdl2::init().expect("SDL init failed");
    let video = sdl.video().expect("SDL video init failed");
    let audio = sdl.audio().expect("SDL audio init failed");
    let game_controller = sdl
        .game_controller()
        .expect("SDL game controller init failed");

    let desired = AudioSpecDesired {
        freq: Some(48_000),
        channels: Some(2),
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

        if queue.size() < 32_768 {
            let mut block = [0i16; 4096];
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
        if let Some(req) = diagnostics.launch_requested {
            println!("Launch requested: {} ({})", req.title, req.cartridge_id);
        }
        if diagnostics.launch_canceled {
            println!("Launch request cleared");
        }
        if let Some(stage) = diagnostics.launch_stage_changed {
            println!("Launch stage: {:?}", stage);
        }
        if let Some(ready) = diagnostics.launch_ready {
            println!("Launch ready: {} ({})", ready.title, ready.cartridge_id);
        }
        if let Some(resolved) = diagnostics.launch_resolved {
            println!(
                "Cartridge resolved: {} ({})",
                resolved.title, resolved.cartridge_id
            );
        }
        if let Some(reject) = diagnostics.launch_rejected {
            println!("Launch rejected: {:?}", reject);
        }

        dispatch_runtime_events(&mut synth, &runtime_events);

        let src = system.framebuffer().pixels();
        present_frame(&mut canvas, &mut texture, src).expect("present frame failed");

        pacer.wait_next_frame();
    }
}
