mod aurex;

use aurex::ppu::framebuffer::{FB_H, FB_W, Framebuffer};
use aurex::runtime::{
    AudioEngine, AudioMode, FlowController, FlowPhase, FramePacer, LaunchStage,
    LaunchValidationError, MixProfile, collect_runtime_diagnostics, dispatch_runtime_events,
    AudioRecorder,
};
#[cfg(feature = "sdl2")]
use aurex::runtime::{poll_input, present_frame};
#[cfg(feature = "sdl2")]
use sdl2::GameControllerSubsystem;
#[cfg(feature = "sdl2")]
use sdl2::audio::AudioSpecDesired;
#[cfg(feature = "sdl2")]
use sdl2::controller::GameController;
use std::fs;
use std::process::Command;
use std::time::Duration;
use png;

#[cfg(feature = "sdl2")]
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

fn runtime_baseline_json(profile: MixProfile, baseline_json: &str, replay_json: &str) -> String {
    format!(
        "{{\"audio_profile\":\"{}\",\"audio_diagnostics_baseline\":{},\"replay_capture_smoke\":{}}}",
        profile.as_str(),
        baseline_json,
        replay_json
    )
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

fn save_screenshot(fb: &Framebuffer, path: &str) -> std::io::Result<()> {
    let width = FB_W as u32;
    let height = FB_H as u32;
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let file = std::fs::File::create(path)?;
    let mut encoder = png::Encoder::new(&file, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    let mut img_data = Vec::with_capacity((width * height * 3) as usize);
    for &pixel in fb.pixels() {
        let r5 = ((pixel >> 10) & 0x1F) as u8;
        let g5 = ((pixel >> 5) & 0x1F) as u8;
        let b5 = (pixel & 0x1F) as u8;
        img_data.push((r5 << 3) | (r5 >> 2));
        img_data.push((g5 << 3) | (g5 >> 2));
        img_data.push((b5 << 3) | (b5 >> 2));
    }
    writer.write_image_data(&img_data)?;
    Ok(())
}


#[cfg(feature = "sdl2")]
fn interactive_main() {
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
                "Audio diagnostics profile={} frames={} peak_l={} peak_r={} avg_abs_l={} avg_abs_r={} crest_l_q10={} crest_r_q10={} clipped_l={} clipped_r={} boot_beat_step={}",
                profile.as_str(),
                diag.frames,
                diag.peak_l,
                diag.peak_r,
                diag.avg_abs_l,
                diag.avg_abs_r,
                diag.crest_l_q10,
                diag.crest_r_q10,
                diag.clipped_l,
                diag.clipped_r,
                diag.boot_beat_step
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
        let json = runtime_baseline_json(profile, &baseline.to_json(), &replay);

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

    let screenshot_frame = parse_usize_arg(&args, "--screenshot-frame", 0);
    let no_audio = args.iter().any(|a| a == "--no-audio");
    let exit_after_screenshot = args.iter().any(|a| a == "--exit-after-screenshot");
    // Bot AI & video recording flags
    let bot_play = args.iter().any(|a| a == "--bot-play");
    let attract_mode = args.iter().any(|a| a == "--attract-mode");
    let bot_play = bot_play || attract_mode;  // attract mode implies bot play
    let record_dir: Option<String> = parse_string_arg(&args, "--record-video");
    if let Some(ref dir) = record_dir {
        std::fs::create_dir_all(dir).expect("create record dir failed");
    }
    let duration_secs = parse_usize_arg(&args, "--duration", 10);
    let max_frames = if duration_secs > 0 { Some(duration_secs as u64 * 60) } else { None };
    let profile = parse_mix_profile(&args);
    let sdl = sdl2::init().expect("SDL init failed");
    let video = sdl.video().expect("SDL video init failed");
    let audio = if no_audio {
        None
    } else {
        Some(sdl.audio().expect("SDL audio init failed"))
    };
    let game_controller = sdl
        .game_controller()
        .expect("SDL game controller init failed");

    let (mut queue, mut synth) = if let Some(audio) = audio {
        let desired = AudioSpecDesired {
            freq: Some(48_000),
            channels: Some(2),
            samples: Some(1024),
        };
        let queue = audio
            .open_queue::<i16, _>(None, &desired)
            .expect("audio queue open failed");
        let sample_rate = queue.spec().freq as u32;
        queue.resume();
        (Some(queue), Some(AudioEngine::new_with_profile(sample_rate, profile)))
    } else {
        (None, None)
    };

    let scale: u32 = 3;
    let window = video
        .window("Aurex-16++", (FB_W as u32) * scale, (FB_H as u32) * scale)
        .position_centered()
        .build()
        .expect("window build failed");

    let mut canvas = if std::env::var("SDL_VIDEODRIVER").map(|v| v == "dummy").unwrap_or(false) {
        window
            .into_canvas()
            .build()
            .expect("canvas build failed (dummy mode)")
    } else {
        window
            .into_canvas()
            .present_vsync()
            .build()
            .expect("canvas build failed (vsync)")
    };

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

    // Attract/kiosk mode: skip boot+library, go straight to game
    if attract_mode {
        if flow.attract_mode() {
            system.start_game();
        }
    }

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

        let start_pressed = if bot_play && flow.phase() == FlowPhase::AwaitStart {
            true
        } else {
            polled.start_pressed
        };
        if flow.tick(start_pressed) {
            system.start_game();
            println!("Library ready");
        }

        system.set_boot_waiting_for_start(flow.waiting_for_start());

        let audio_mode = match flow.phase() {
            FlowPhase::Boot | FlowPhase::AwaitStart => AudioMode::Boot,
            FlowPhase::Game => AudioMode::Game,
        };

        let mut input = polled.gameplay;
        let boot_beat_step = if matches!(audio_mode, AudioMode::Boot) {
            synth.as_ref().map(|s| s.boot_beat_step())
        } else {
            None
        };

        // Bot AI override: full system navigation + gameplay
        if bot_play {
            let mut bot_input = aurex::game::InputState::default();
            // Phase 1: During AwaitStart (boot complete), press START to enter library
            if flow.phase() == FlowPhase::AwaitStart {
                bot_input.accept = true;
            }
            // Phase 2: In library (Game mode but no cartridge loaded), press START to launch selected
            if flow.phase() == FlowPhase::Game && system.game_runtime_ref().is_none() {
                bot_input.accept = true;
            }
            // Phase 3: If a game cartridge is active, use its bot AI (overrides navigation inputs)
            if let Some(game) = system.game_runtime_ref() {
                if let Some(game_bot) = game.bot_input() {
                    bot_input = game_bot;
                }
            }
            input = bot_input;
        }

        system.run_frame(input, boot_beat_step);
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

        if let (Some(queue), Some(synth)) = (queue.as_mut(), synth.as_mut()) {
            dispatch_runtime_events(synth, &runtime_events);
            if queue.size() < 32_768 {
                let mut block = [0i16; 4096];
                synth.render_block(audio_mode, &mut block);
                let _ = queue.queue_audio(&block);
            }
        }

        let src = system.framebuffer().pixels();
        present_frame(&mut canvas, &mut texture, src).expect("present frame failed");
        if screenshot_frame > 0 && system.ui_frame == screenshot_frame as u64 {
            let _ = save_screenshot(system.framebuffer(), "artifacts/screenshot.png");
            if exit_after_screenshot {
                break 'running;
            }
        }

        // Continuous video recording: save every frame as PNG
        if let Some(ref rec_dir) = record_dir {
            let frame_path = format!("{}/frame_{:06}.png", rec_dir, system.ui_frame);
            let _ = save_screenshot(system.framebuffer(), &frame_path);
        }

        // Duration limit: exit after requested number of frames
        if let Some(max) = max_frames {
            if system.ui_frame >= max {
                println!("⏱️  Max duration ({} frames) reached, exiting.", max);
                break 'running;
            }
        }

        pacer.wait_next_frame();
    }
}

fn headless_main() {
    let args: Vec<String> = std::env::args().collect();

    // Early diagnostic/utility modes
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
                "Audio diagnostics profile={} frames={} peak_l={} peak_r={} avg_abs_l={} avg_abs_r={} crest_l_q10={} crest_r_q10={} clipped_l={} clipped_r={} boot_beat_step={}",
                profile.as_str(),
                diag.frames,
                diag.peak_l,
                diag.peak_r,
                diag.avg_abs_l,
                diag.avg_abs_r,
                diag.crest_l_q10,
                diag.crest_r_q10,
                diag.clipped_l,
                diag.clipped_r,
                diag.boot_beat_step
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
        let json = runtime_baseline_json(profile, &baseline.to_json(), &replay);
        if let Some(parent) = std::path::Path::new(&out).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).expect("baseline output directory create failed");
            }
        }
        std::fs::write(&out, &json).expect("baseline output write failed");
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

    // Normal operation flags
    let bot_play = args.iter().any(|a| a == "--bot-play");
    let attract_mode = args.iter().any(|a| a == "--attract-mode");
    let bot_play = bot_play || attract_mode;  // attract mode implies bot play
    let record_dir: Option<String> = parse_string_arg(&args, "--record-video");
    if let Some(dir) = &record_dir {
        std::fs::create_dir_all(dir).expect("create record dir failed");
    }
    let duration_secs = parse_usize_arg(&args, "--duration", 10);
    let max_frames = if duration_secs > 0 {
        Some(duration_secs as u64 * 60)
    } else {
        None
    };
    let _profile = parse_mix_profile(&args);

    // Audio recorder + synthesizer (only needed if recording)
    let mut audio_recorder: Option<AudioRecorder> = if let Some(dir) = &record_dir {
        let wav_path = format!("{}/audio.wav", dir);
        Some(AudioRecorder::new(&wav_path, 48_000).expect("Failed to create audio recorder"))
    } else {
        None
    };
    let mut synth: Option<AudioEngine> = if audio_recorder.is_some() {
        Some(AudioEngine::new_with_profile(48_000, MixProfile::Arcade))
    } else {
        None
    };

    // Initialize system
    let mut system = aurex::Aurex::new();
    let mut flow = FlowController::new();
    let mut pacer = FramePacer::new(Duration::from_nanos(16_666_667));
    let mut runtime_events = Vec::with_capacity(8);

    // Attract/kiosk mode: skip boot+library, go straight to gameplay
    if attract_mode {
        if flow.attract_mode() {
            system.start_game();
        }
    }

    'running: loop {
        // Input generation
        let input = if bot_play {
            let mut bot_input = aurex::game::InputState::default();
            if flow.phase() == FlowPhase::AwaitStart {
                bot_input.accept = true;
            }
            if flow.phase() == FlowPhase::Game && system.game_runtime_ref().is_none() {
                bot_input.accept = true;
            }
            if let Some(game) = system.game_runtime_ref() {
                if let Some(game_bot) = game.bot_input() {
                    bot_input = game_bot;
                }
            }
            bot_input
        } else {
            aurex::game::InputState::default()
        };

        let start_pressed = bot_play && flow.phase() == FlowPhase::AwaitStart;
        if flow.tick(start_pressed) {
            system.start_game();
            println!("Library ready");
        }

        system.set_boot_waiting_for_start(flow.waiting_for_start());

        let audio_mode = match flow.phase() {
            FlowPhase::Boot | FlowPhase::AwaitStart => AudioMode::Boot,
            FlowPhase::Game => AudioMode::Game,
        };

        let boot_beat_step = if matches!(audio_mode, AudioMode::Boot) {
            synth.as_ref().map(|s| s.boot_beat_step())
        } else {
            None
        };

        system.run_frame(input, boot_beat_step);
        runtime_events.clear();
        system.drain_events(&mut runtime_events);

        // Audio rendering and capture
        if let Some(synth) = synth.as_mut() {
            dispatch_runtime_events(synth, &runtime_events);
            if let Some(rec) = audio_recorder.as_mut() {
                let mut block = [0i16; 4096];
                synth.render_block(audio_mode, &mut block);
                let _ = rec.write_block(&block);
            }
        }

        // Frame capture
        if let Some(ref rec_dir) = record_dir {
            let frame_path = format!("{}/frame_{:06}.png", rec_dir, system.ui_frame);
            let _ = save_screenshot(system.framebuffer(), &frame_path);
        }

        // Duration limit
        if let Some(max) = max_frames {
            if system.ui_frame >= max {
                println!("Max duration ({} frames) reached, exiting.", max);
                break 'running;
            }
        }

        pacer.wait_next_frame();
    }

    // MP4 assembly
    if let Some(ref rec_dir) = record_dir {
        let mp4_path = format!("{}/output.mp4", rec_dir);
        let wav_path = format!("{}/audio.wav", rec_dir);
        let status = Command::new("ffmpeg")
            .args(&[
                "-y",
                "-framerate", "60",
                "-i", &format!("{}/frame_%06d.png", rec_dir),
                "-i", &wav_path,
                "-c:v", "libx264",
                "-pix_fmt", "yuv420p",
                "-c:a", "aac",
                "-shortest",
                &mp4_path,
            ])
            .status();
        match status {
            Ok(s) if s.success() => {
                println!("MP4 created: {}", mp4_path);
            }
            _ => {
                eprintln!("ffmpeg not available or failed; raw frames and WAV remain in {}", rec_dir);
            }
        }
    }
}

#[cfg(feature = "sdl2")]
fn main() {
    interactive_main();
}
#[cfg(not(feature = "sdl2"))]
fn main() {
    headless_main();
}
