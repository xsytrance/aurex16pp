pub mod clock;
pub mod dma;
pub mod pdu;
pub mod ppu;
pub mod vm32;
pub mod wram;

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
        }
    }

    pub fn run(&mut self) {
        loop {
            self.clock.begin_frame();
            self.pdu.begin_frame();

            self.dma.begin_frame();

            // DMA smoke test (temporary): accept 4 commands, reject the 5th
            use crate::aurex::dma::command::DmaCommand;

            let _ = self.dma.request(DmaCommand::vram_upload(1024));
            let _ = self.dma.request(DmaCommand::vram_upload(1024));
            let _ = self.dma.request(DmaCommand::audio_upload(1024));
            let _ = self.dma.request(DmaCommand::audio_upload(1024));
            let fifth = self.dma.request(DmaCommand::vram_upload(1024));

            self.vm.run_frame(&mut self.pdu);

            self.pdu.ingest_dma(
                self.dma.commands_used(),
                self.dma.vram_bytes_used(),
                self.dma.audio_bytes_used(),
                self.dma.rejects_this_frame(),
            );

            self.pdu.end_frame();

            self.clock.end_frame();
        }
    }
}
