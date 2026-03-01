use std::time::{Duration, Instant};

pub struct Clock {
    frame_duration: Duration,
    next_frame: Instant,
}

impl Clock {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            frame_duration: Duration::from_nanos(16_666_667),
            next_frame: now,
        }
    }

    pub fn begin_frame(&mut self) {
        self.next_frame += self.frame_duration;
    }

    pub fn end_frame(&self) {
        while Instant::now() < self.next_frame {
            std::hint::spin_loop();
        }
    }
}
