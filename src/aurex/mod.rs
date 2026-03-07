pub mod boot;
pub mod clock;
pub mod dma;
pub mod game;
pub mod pdu;
pub mod ppu;
pub mod runtime;
pub mod vm32;
pub mod wram;

use crate::aurex::ppu::ppu::PPU_STATUS;
use crate::aurex::ppu::ppu::Ppu;
use crate::aurex::runtime::{RuntimeEvent, RuntimeEventQueue, SceneId};
use boot::prime_ignition::PrimeIgnition;
use clock::Clock;
use dma::controller::DmaController;
use game::{AudioCue, InputState, library::LibraryScreen};
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
    boot: PrimeIgnition,
    library: LibraryScreen,
    mode: RunMode,
    events: RuntimeEventQueue,
    ui_frame: u64,
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
            boot: PrimeIgnition::new(),
            library,
            mode: RunMode::Boot,
            events: RuntimeEventQueue::with_capacity(8),
            ui_frame: 0,
        }
    }

    pub fn start_game(&mut self) {
        self.mode = RunMode::Game;
        self.events
            .push(RuntimeEvent::SceneChanged(SceneId::Library));
        self.events
            .push(RuntimeEvent::Audio(self.library.current_audio_cue()));
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_frame(InputState::default());
        }
    }

    pub fn run_frame(&mut self, input: InputState) {
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
                let update = self.library.update(input);
                if !matches!(update.audio_cue, AudioCue::None) {
                    self.events.push(RuntimeEvent::Audio(update.audio_cue));
                }
                if update.launch_requested {
                    self.events.push(RuntimeEvent::TitleLaunchRequested(
                        self.library.current_title(),
                    ));
                }
            }
        }

        self.ppu.render_frame(&self.vram, &mut self.fb);

        match self.mode {
            RunMode::Boot => self.boot.draw_overlay(&mut self.fb),
            RunMode::Game => self.library.draw(&mut self.fb, self.ui_frame),
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
}
