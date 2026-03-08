mod aurex;

use aurex::ppu::framebuffer::{FB_H, FB_W};
use aurex::runtime::{
    AudioEngine, AudioMode, FlowController, FlowPhase, FramePacer, LaunchStage,
    LaunchValidationError, MixProfile, collect_runtime_diagnostics, dispatch_runtime_events,
    poll_input, present_frame,
};
use sdl2::GameControllerSubsystem;
use sdl2::audio::AudioSpecDesired;
use sdl2::controller::GameController;
use std::fs;
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

fn parse_usize_arg(args: &[String], flag: &str, default: usize) -> usize {
    for i in 0..args.len().saturating_sub(1) {
        if args[i] == flag {
            if let Ok(v) = args[i + 1].parse::<usize>() {
                return v.max(1);
            }
        }
    }
    default
}

fn parse_string_arg(args: &[String], flag: &str) -> Option<String> {
    for i in 0..args.len().saturating_sub(1) {
        if args[i] == flag {
            return Some(args[i + 1].clone());
        }
    }
    None
}

fn parse_mix_profile(args: &[String]) -> MixProfile {
    if let Some(raw) = parse_string_arg(args, "--audio-profile") {
        return MixProfile::parse(&raw).unwrap_or(MixProfile::Default);
    }
    MixProfile::Default
}

fn replay_capture_smoke_summary_json() -> String {
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
        system.run_frame(input, None);
        events.clear();
        system.drain_events(&mut events);
        for (i, _event) in events.iter().enumerate() {
            cap.capture_event_tag((frame as u64) << 16 | i as u64);
        }
        cap.capture_framebuffer(system.framebuffer().pixels());
        cap.end_frame();
    }

    cap.summary_json()
}

fn format_launch_stage(stage: LaunchStage) -> String {
    match stage {
        LaunchStage::Idle => "idle".to_string(),
        LaunchStage::Pending(desc) => {
            format!(
                "pending title={} cartridge={}",
                desc.title, desc.cartridge_id
            )
        }
        LaunchStage::Validating(desc) => {
            format!(
                "validating title={} cartridge={}",
                desc.title, desc.cartridge_id
            )
        }
        LaunchStage::Ready(desc) => {
            format!("ready title={} cartridge={}", desc.title, desc.cartridge_id)
        }
        LaunchStage::Rejected(reason) => {
            format!("rejected reason={}", format_launch_validation_error(reason))
        }
    }
}

fn format_launch_validation_error(reason: LaunchValidationError) -> &'static str {
    match reason {
        LaunchValidationError::EmptyCartridgeId => "empty_cartridge_id",
        LaunchValidationError::InvalidCartridgeId => "invalid_cartridge_id",
        LaunchValidationError::CartridgeMissing => "cartridge_missing",
        LaunchValidationError::CartridgeManifestInvalid => "cartridge_manifest_invalid",
    }
}

fn docs_sync_check() -> Result<(), String> {
    let architecture = fs::read_to_string("docs/architecture.md")
        .map_err(|e| format!("read docs/architecture.md failed: {e}"))?;
    let canon = fs::read_to_string("docs/ai_handoff_canon.md")
        .map_err(|e| format!("read docs/ai_handoff_canon.md failed: {e}"))?;

    let expected = [
        "--generate-runtime-baseline",
        "--docs-sync-check",
        "Launch stage: pending",
        "Launch stage: validating",
        "Launch stage: ready",
    ];

    for needle in expected {
        if !architecture.contains(needle) {
            return Err(format!(
                "docs sync check failed: docs/architecture.md missing marker '{needle}'"
            ));
        }
    }

    if !canon.contains("Audio diagnostics baseline artifact generation is required") {
        return Err(
            "docs sync check failed: docs/ai_handoff_canon.md missing baseline discipline line"
                .to_string(),
        );
    }

    Ok(())
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
        let frames = parse_usize_arg(&args, "--frames", 48_000);

        let mode = if args.iter().any(|a| a == "--boot") {
            aurex::runtime::AudioMode::Boot
        } else {
            aurex::runtime::AudioMode::Game
        };

        let profile = parse_mix_profile(&args);
        let engine = aurex::runtime::AudioEngine::new_with_profile(48_000, profile);
        let diag = engine.diagnostics_for_frames(mode, frames);
        if args.iter().any(|a| a == "--json") {
            println!("{}", diag.to_json());
        } else {
            println!(
                "Audio diagnostics profile={} frames={} peak_l={} peak_r={} avg_abs_l={} avg_abs_r={} crest_l_q10={} crest_r_q10={} clipped_l={} clipped_r={}",
                profile.as_str(),
                diag.frames,
                diag.peak_l,
                diag.peak_r,
                diag.avg_abs_l,
                diag.avg_abs_r,
                diag.crest_l_q10,
                diag.crest_r_q10,
                diag.clipped_l,
                diag.clipped_r
            );
        }
        return;
    }

    if args.iter().any(|a| a == "--generate-runtime-baseline") {
        let frames = parse_usize_arg(&args, "--frames", 48_000);
        let out = parse_string_arg(&args, "--out")
            .unwrap_or_else(|| "artifacts/runtime_audio_diag_baseline.json".to_string());
        let profile = parse_mix_profile(&args);
        let engine = aurex::runtime::AudioEngine::new_with_profile(48_000, profile);
        let baseline = engine.diagnostics_baseline(frames);
        let replay = replay_capture_smoke_summary_json();
        let json = format!(
            "{{\"audio_diagnostics_baseline\":{},\"replay_capture_smoke\":{}}}",
            baseline.to_json(),
            replay
        );

        if let Some(parent) = std::path::Path::new(&out).parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).expect("baseline output directory create failed");
            }
        }

        fs::write(&out, &json).expect("baseline output write failed");
        println!("Baseline artifact written: {}", out);
        println!("{}", json);
        return;
    }

    if args.iter().any(|a| a == "--docs-sync-check") {
        match docs_sync_check() {
            Ok(()) => {
                println!("Docs sync check: PASS");
                return;
            }
            Err(err) => {
                eprintln!("Docs sync check: FAIL: {err}");
                std::process::exit(2);
            }
        }
    }

    if args.iter().any(|a| a == "--palette-heatmap") {
        let v = aurex::ppu::vram::Vram::new();
        println!("{}", v.bg0_palette_bank_heatmap_json());
        return;
    }

    if args.iter().any(|a| a == "--replay-capture-smoke") {
        println!("{}", replay_capture_smoke_summary_json());
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

    let profile = parse_mix_profile(&args);
    let mut synth = AudioEngine::new_with_profile(queue.spec().freq as u32, profile);
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

        system.run_frame(input, None);
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
            println!("Launch stage: {}", format_launch_stage(stage));
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
            println!(
                "Launch rejected: reason={}",
                format_launch_validation_error(reject)
            );
        }

        dispatch_runtime_events(&mut synth, &runtime_events);

        let src = system.framebuffer().pixels();
        present_frame(&mut canvas, &mut texture, src).expect("present frame failed");

        pacer.wait_next_frame();
    }
}
