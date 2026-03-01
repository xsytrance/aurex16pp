pub struct Pdu {
    ops_used: u32,
    frame_index: u64,
}

const OPS_CAP: u32 = 200_000;

impl Pdu {
    pub fn new() -> Self {
        Self {
            ops_used: 0,
            frame_index: 0,
        }
    }

    pub fn begin_frame(&mut self) {
        self.ops_used = 0;
    }

    pub fn consume(&mut self, ops: u32) -> bool {
        if self.ops_used + ops > OPS_CAP {
            return false;
        }
        self.ops_used += ops;
        true
    }

    pub fn end_frame(&mut self) {
        self.frame_index += 1;

        if self.frame_index % 60 == 0 {
            println!("Frame: {}", self.frame_index);
        }
    }
}
