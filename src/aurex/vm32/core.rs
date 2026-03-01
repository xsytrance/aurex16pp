use crate::aurex::pdu::Pdu;

pub struct Vm32;

impl Vm32 {
    pub fn new() -> Self {
        Self
    }

    pub fn run_frame(&mut self, pdu: &mut Pdu) {
        // Simulate CPU work
        for _ in 0..50_000 {
            if !pdu.consume(1) {
                break;
            }
        }
    }
}
