use std::time::{Duration, Instant};

pub struct FramePacer {
    target: Duration,
    last: Instant,
}

impl FramePacer {
    pub fn new(target: Duration) -> Self {
        Self {
            target,
            last: Instant::now(),
        }
    }

    pub fn wait_next_frame(&mut self) {
        let elapsed = self.last.elapsed();
        if elapsed < self.target {
            std::thread::sleep(self.target - elapsed);
        }
        self.last = Instant::now();
    }
}
