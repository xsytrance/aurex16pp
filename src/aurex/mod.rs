pub mod clock;
pub mod dma;
pub mod pdu;
pub mod ppu;
pub mod vm32;
pub mod wram;

use crate::aurex::ppu::ppu::Ppu;
use clock::Clock;
use dma::controller::DmaController;
use pdu::Pdu;
use ppu::vram::Vram;
use vm32::core::Vm32;
use wram::Wram;

pub struct Aurex {
    clock: Clock,
    pdu: Pdu,
    wram: Wram,
    vm: Vm32,
    dma: DmaController,
    vram: Vram,
    fb: ppu::framebuffer::Framebuffer,
    ppu: Ppu,
}

impl Aurex {
    pub fn new() -> Self {
        Self {
            clock: Clock::new(),
            pdu: Pdu::new(),
            wram: Wram::new(),
            vm: Vm32::new(),
            dma: DmaController::new(),
            vram: Vram::new(),
            fb: ppu::framebuffer::Framebuffer::new(),
            ppu: Ppu::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            self.clock.begin_frame();
            self.pdu.begin_frame();

            use crate::aurex::ppu::framebuffer::rgb555;

            self.fb.clear(rgb555(0, 0, 0)); // black

            // =====================================================================
            // TEMP TEST: Debug framebuffer pattern
            // ---------------------------------------------------------------------
            // Enabled only in debug builds.
            // Remove or replace when real PPU rendering exists.
            // =====================================================================
            // =====================================================================
            // PPU FRAME RENDER
            // =====================================================================
            self.ppu.render_frame(&mut self.fb);

            self.dma.begin_frame();

            // CPU execution for this frame
            self.vm.run_frame(&mut self.pdu);

            // Apply accepted DMA transfers to hardware memory
            self.dma.apply(&self.wram, &mut self.vram);

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
    }
}
