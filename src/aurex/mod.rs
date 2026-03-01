pub mod clock;
pub mod pdu;
pub mod vm32;
pub mod wram;

use clock::Clock;
use pdu::Pdu;
use vm32::core::Vm32;
use wram::Wram;

pub struct Aurex {
    clock: Clock,
    pdu: Pdu,
    wram: Wram,
    vm: Vm32,
}

impl Aurex {
    pub fn new() -> Self {
        Self {
            clock: Clock::new(),
            pdu: Pdu::new(),
            wram: Wram::new(),
            vm: Vm32::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            self.clock.begin_frame();
            self.pdu.begin_frame();

            self.vm.run_frame(&mut self.pdu);

            self.pdu.end_frame();

            self.clock.end_frame();
        }
    }
}
