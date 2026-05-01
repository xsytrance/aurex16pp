#![allow(dead_code)]
pub mod boot;
pub mod cartridge;
pub mod clock;
pub mod dma;
pub mod game;
pub mod pdu;
pub mod ppu;
pub mod runtime;
pub mod vm32;
pub mod wram;

use crate::aurex::cartridge::CartridgeRuntime;
use crate::aurex::dma::controller::DmaController;
use crate::aurex::ppu::ppu::PPU_STATUS;
use crate::aurex::ppu::ppu::Ppu;
use crate::aurex::runtime::{
    validate_launch_descriptor, AudioSfx, GameOutcome, GameRuntime, LaunchIntentController,
    LaunchStage, LaunchValidationError, RuntimeAudioCommand, RuntimeEvent, RuntimeEventQueue,
    SceneId,
};
use crate::aurex::runtime::game_runtime::NoopGame;
use crate::aurex::game::blocks_and_bricks::BlocksAndBricks;
use boot::prime_awakens::PrimeAwakens;
use clock::Clock;
use game::{library::{LibraryScreen, LibraryUpdate}, AudioCue, InputState};

fn to_audio_command(cue: AudioCue) -> Option<RuntimeAudioCommand> {
    match cue {
        AudioCue::None => None,
        AudioCue::SelectTrack(track) => Some(RuntimeAudioCommand::PlayTrack(track)),
        AudioCue::LaunchRequest => Some(RuntimeAudioCommand::PlaySfx(AudioSfx::Launch)),
        AudioCue::Cancel => Some(RuntimeAudioCommand::PlaySfx(AudioSfx::Cancel)),
    }
}

use pdu::Pdu;
use ppu::vram::Vram;
use vm32::core::Vm32;
use wram::Wram;

enum RunMode {
    Boot,
    Game,
}

pub struct Aurex {
    clock: Clock,
    pdu: Pdu,
    wram: Wram,
    vm: Vm32,
    dma: DmaController,
    vram: Vram,
    fb: ppu::framebuffer::Framebuffer,
    ppu: Ppu,
    boot: PrimeAwakens,
    library: LibraryScreen,
    mode: RunMode,
    events: RuntimeEventQueue,
    launch: LaunchIntentController,
    pub ui_frame: u64,
    // Phase 1: Agent Console - cartridge execution
    game_runtime: Option<Box<dyn GameRuntime>>,
    current_cartridge_id: Option<&'static str>,
}

impl Aurex {
    pub fn new() -> Self {
        let vram = Vram::new();
        let library = LibraryScreen::new();

        Self {
            clock: Clock::new(),
            pdu: Pdu::new(),
            wram: Wram::new(),
            vm: Vm32::new(),
            dma: DmaController::new(),
            vram,
            fb: ppu::framebuffer::Framebuffer::new(),
            ppu: Ppu::new(),
            boot: PrimeAwakens::new(),
            library,
            mode: RunMode::Boot,
            events: RuntimeEventQueue::with_capacity(8),
            launch: LaunchIntentController::new(),
            ui_frame: 0,
            // Phase 1: Agent Console
            game_runtime: None,
            current_cartridge_id: None,
        }
    }

    pub fn start_game(&mut self) {
        self.mode = RunMode::Game;
        self.events
            .push(RuntimeEvent::SceneChanged(SceneId::Library));
        self.events
            .push(RuntimeEvent::Audio(RuntimeAudioCommand::PlayTrack(0)));
        self.events
            .push(RuntimeEvent::Audio(RuntimeAudioCommand::PlaySfx(
                AudioSfx::Confirm,
            )));
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_frame(InputState::default(), None);
        }
    }

    pub fn run_frame(&mut self, input: InputState, boot_beat_step: Option<u8>) {
        self.clock.begin_frame();
        self.pdu.begin_frame();

        use crate::aurex::ppu::framebuffer::rgb555;

        self.fb.clear(rgb555(0, 0, 0));
        self.dma.begin_frame();
        self.vm.run_frame(&mut self.pdu);

        match self.mode {
            RunMode::Boot => {
                self.boot
                    .update(&mut self.ppu, &mut self.dma, &mut self.wram, &self.vram);
            }
            RunMode::Game => {
                let update = if self.game_runtime.is_none() { self.library.update(input) } else { LibraryUpdate { audio_cue: AudioCue::None, launch_requested: false, launch_canceled: false } };
                if let Some(cmd) = to_audio_command(update.audio_cue) {
                    self.events.push(RuntimeEvent::Audio(cmd));
                }

                if update.launch_requested {
                    let req = self.library.current_launch_descriptor();
                    match validate_launch_descriptor(req) {
                        Ok(()) => {
                            if self.launch.request(req) {
                                self.events.push(RuntimeEvent::TitleLaunchRequested(req));
                                self.events
                                    .push(RuntimeEvent::LaunchStageChanged(self.launch.stage()));
                            }
                        }
                        Err(reason) => {
                            self.launch.reject(reason);
                            self.events.push(RuntimeEvent::TitleLaunchRejected(reason));
                            self.events
                                .push(RuntimeEvent::LaunchStageChanged(self.launch.stage()));
                        }
                    }
                }

                if update.launch_canceled && self.launch.cancel() {
                    self.events.push(RuntimeEvent::TitleLaunchCanceled);
                    self.events
                        .push(RuntimeEvent::LaunchStageChanged(self.launch.stage()));
                }

                if let Some(stage) = self.launch.tick() {
                    self.events.push(RuntimeEvent::LaunchStageChanged(stage));
                    if let LaunchStage::Ready(desc) = stage {
                        self.events.push(RuntimeEvent::TitleLaunchReady(desc));
                        match CartridgeRuntime::from_cartridge_id(desc.cartridge_id) {
                            Ok(cartridge) => {
                                self.events.push(RuntimeEvent::TitleLaunchResolved(desc));
                                
                                // Phase 2: Agent Console - Load BlocksAndBricks for blocks_and_bricks cartridge
                                // TODO: Add generic game loader based on cartridge_id
                                let game: Box<dyn GameRuntime> = if desc.cartridge_id == "blocks_and_bricks" {
                                    Box::new(BlocksAndBricks::new())
                                } else {
                                    // Fallback to NoopGame for other cartridges
                                    Box::new(NoopGame)
                                };
                                
                                if self.game_runtime.is_none() {
                                    self.game_runtime = Some(game);
                                    self.current_cartridge_id = Some(desc.cartridge_id);
                                    self.events
                                        .push(RuntimeEvent::GameStarted(desc.cartridge_id));
                                    
                                    // Initialize game runtime with cartridge and VRAM
                                    self.game_runtime.as_mut().unwrap().initialize(&cartridge, &mut self.vram, &mut self.ppu);
                                }
                            }
                            Err(
                                crate::aurex::cartridge::CartridgeResolveError::MissingManifest,
                            ) => {
                                self.launch.reject(LaunchValidationError::CartridgeMissing);
                                self.events.push(RuntimeEvent::TitleLaunchRejected(
                                    LaunchValidationError::CartridgeMissing,
                                ));
                                self.events
                                    .push(RuntimeEvent::LaunchStageChanged(self.launch.stage()));
                            }
                            Err(
                                crate::aurex::cartridge::CartridgeResolveError::InvalidManifest(_),
                            ) => {
                                self.launch
                                    .reject(LaunchValidationError::CartridgeManifestInvalid);
                                self.events.push(RuntimeEvent::TitleLaunchRejected(
                                    LaunchValidationError::CartridgeManifestInvalid,
                                ));
                                self.events
                                    .push(RuntimeEvent::LaunchStageChanged(self.launch.stage()));
                            }
                        }
                    }
                }

                self.library.set_launch_stage(self.launch.stage());
                
                // Phase 1: Agent Console - Execute game runtime if attached
                if let Some(game) = self.game_runtime.as_mut() {
                    let ops_budget = 200_000 - self.pdu.cpu_consumed();
                    
                    let outcome = game.update(input, ops_budget);
                    
                    // Handle game outcome
                    match outcome {
                        GameOutcome::Running => {
                            // Check for CPU rejects
                            let cpu_rejects = self.dma.rejects_this_frame();
                            if cpu_rejects > 0 {
                                self.events.push(RuntimeEvent::GameCpuRejects(cpu_rejects));
                            }
                        }
                        GameOutcome::Paused => {
                            self.events.push(RuntimeEvent::GamePaused);
                        }
                        GameOutcome::Completed { score } => {
                            self.events.push(RuntimeEvent::GameCompleted(score));
                        }
                        GameOutcome::Failed { reason } => {
                            self.events.push(RuntimeEvent::GameFailed(reason));
                        }
                    }
                    
                    // Render game frame
                    game.render(&mut self.ppu, &mut self.dma);
                }
            }
        }

        self.ppu.render_frame(&self.vram, &mut self.fb);

        match self.mode {
            RunMode::Boot => self.boot.draw_overlay(&mut self.fb, boot_beat_step),
            RunMode::Game => {
                // Draw library UI only when no active game runtime is present.
                // When a game is running, the library overlay is suppressed to avoid obscuring game graphics.
                if self.game_runtime.is_none() {
                    self.library.draw(&mut self.fb, self.ui_frame);
                }
            }
        }

        self.pdu.ingest_ppu(
            self.ppu.sprite_overflow_latched(),
            self.ppu.sprite_overflow_scanlines(),
        );

        let vblank = self.ppu.read_addr(PPU_STATUS) & 0x1 != 0;
        self.dma.apply(&self.wram, &mut self.vram, vblank);

        self.pdu.ingest_dma(
            self.dma.commands_used(),
            self.dma.vram_bytes_used(),
            0,
            self.dma.rejects_this_frame(),
        );

        self.pdu.end_frame();
        self.clock.end_frame();
        self.ui_frame = self.ui_frame.wrapping_add(1);
    }

    pub fn current_scene(&self) -> SceneId {
        match self.mode {
            RunMode::Boot => SceneId::Boot,
            RunMode::Game => SceneId::Library,
        }
    }

    pub fn set_boot_waiting_for_start(&mut self, waiting: bool) {
        self.boot.set_waiting_for_start(waiting);
    }

    pub fn drain_events(&mut self, out: &mut Vec<RuntimeEvent>) {
        self.events.drain_to(out);
    }

    pub fn framebuffer(&self) -> &crate::aurex::ppu::framebuffer::Framebuffer {
        &self.fb
    }

    /// Get a reference to the current game runtime (if a cartridge is loaded)
    pub fn game_runtime_ref(&self) -> Option<&dyn GameRuntime> {
        self.game_runtime.as_deref()
    }
}
