pub mod boot;
pub mod clock;
pub mod dma;
pub mod game;
pub mod pdu;
pub mod ppu;
pub mod vm32;
pub mod wram;

use crate::aurex::ppu::ppu::PPU_STATUS;
use crate::aurex::ppu::ppu::Ppu;
use boot::prime_ignition::PrimeIgnition;
use clock::Clock;
use dma::controller::DmaController;
use game::{AudioCue, InputState, tech_demo::TechDemo};
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
    game: TechDemo,
    mode: RunMode,
    audio_cue: AudioCue,
}

impl Aurex {
    pub fn new() -> Self {
        let mut vram = Vram::new();
        let game = TechDemo::new(&mut vram);

        let s = Self {
            clock: Clock::new(),
            pdu: Pdu::new(),
            wram: Wram::new(),
            vm: Vm32::new(),
            dma: DmaController::new(),
            vram,
            fb: ppu::framebuffer::Framebuffer::new(),
            ppu: Ppu::new(),
            boot: PrimeIgnition::new(),
            game,
            mode: RunMode::Boot,
            audio_cue: AudioCue::None,
        };

        s
    }

    pub fn start_game(&mut self) {
        self.mode = RunMode::Game;
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

        // Clear to black each frame (v0.1)
        self.fb.clear(rgb555(0, 0, 0));

        // ---------------------------------------------------------------------
        // DMA + CPU + GAME UPDATE
        // ---------------------------------------------------------------------
        self.dma.begin_frame();

        // CPU execution for this frame
        self.vm.run_frame(&mut self.pdu);

        match self.mode {
            RunMode::Boot => {
                self.boot
                    .update(&mut self.ppu, &mut self.dma, &mut self.wram, &self.vram);
            }
            RunMode::Game => {
                // Tech demo gameplay update
                self.audio_cue = self.game.update(&mut self.ppu, input);
            }
        }

        // ---------------------------------------------------------------------
        // PPU FRAME RENDER
        // ---------------------------------------------------------------------
        self.ppu.render_frame(&self.vram, &mut self.fb);

        if let RunMode::Boot = self.mode {
            self.boot.draw_overlay(&mut self.fb);
        }

        // =====================================================================
        // PPU → PDU TELEMETRY BRIDGE
        // ---------------------------------------------------------------------
        // The PPU latches hardware events during rendering (e.g. sprite overflow).
        // The PDU collects per-frame telemetry for debugging / future SDK hooks.
        // This keeps rendering logic isolated from diagnostics logic.
        // =====================================================================
        self.pdu.ingest_ppu(
            self.ppu.sprite_overflow_latched(),
            self.ppu.sprite_overflow_scanlines(),
        );

        // Apply accepted DMA transfers to hardware memory
        let vblank = self.ppu.read_addr(PPU_STATUS) & 0x1 != 0;
        self.dma.apply(&self.wram, &mut self.vram, vblank);

        // Aggregate telemetry into PDU
        self.pdu.ingest_dma(
            self.dma.commands_used(),
            self.dma.vram_bytes_used(),
            0, // audio not implemented yet
            self.dma.rejects_this_frame(),
        );

        self.pdu.end_frame();
        self.clock.end_frame();
    }

    pub fn set_boot_confirming(&mut self, confirming: bool) {
        self.boot.set_confirming(confirming);
    }

    pub fn take_audio_cue(&mut self) -> AudioCue {
        let cue = self.audio_cue;
        self.audio_cue = AudioCue::None;
        cue
    }
    pub fn framebuffer(&self) -> &crate::aurex::ppu::framebuffer::Framebuffer {
        &self.fb
    }
}
