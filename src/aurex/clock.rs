// ============================================================================
// Deterministic Frame Clock
// ----------------------------------------------------------------------------
// Maintains fixed 60 FPS pacing without runaway drift.
//
// IMPORTANT:
// - Frame time is constant (16.666667 ms)
// - Uses real time only for pacing
// - Does NOT accumulate unbounded frame targets
// ============================================================================

use std::time::{Duration, Instant};

pub struct Clock {
    frame_duration: Duration,
    frame_start: Instant,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            frame_duration: Duration::from_nanos(16_666_667),
            frame_start: Instant::now(),
        }
    }

    pub fn begin_frame(&mut self) {
        self.frame_start = Instant::now();
    }

    pub fn end_frame(&self) {
        let target = self.frame_start + self.frame_duration;

        while Instant::now() < target {
            std::hint::spin_loop();
        }
    }
}
