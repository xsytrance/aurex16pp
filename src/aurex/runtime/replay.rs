use crate::aurex::game::InputState;

#[derive(Default)]
pub struct ReplayCapture {
    input_hash: u64,
    event_hash: u64,
    framebuffer_hash: u64,
    pub frames: u64,
}

impl ReplayCapture {
    pub fn new() -> Self {
        Self {
            input_hash: 0x9E37_79B9_7F4A_7C15,
            event_hash: 0xC2B2_AE3D_27D4_EB4F,
            framebuffer_hash: 0x1656_67B1_9E37_9F9B,
            frames: 0,
        }
    }

    pub fn capture_input(&mut self, input: InputState) {
        let bits = (input.left as u64)
            | ((input.right as u64) << 1)
            | ((input.up as u64) << 2)
            | ((input.down as u64) << 3)
            | ((input.accept as u64) << 4)
            | ((input.cancel as u64) << 5);
        self.input_hash = mix(self.input_hash, bits);
    }

    pub fn capture_event_tag(&mut self, tag: u64) {
        self.event_hash = mix(self.event_hash, tag);
    }

    pub fn capture_framebuffer(&mut self, pixels: &[u16]) {
        let mut acc = self.framebuffer_hash;
        for px in pixels.iter().step_by(97) {
            acc = mix(acc, *px as u64);
        }
        self.framebuffer_hash = acc;
    }

    pub fn end_frame(&mut self) {
        self.frames = self.frames.wrapping_add(1);
        self.input_hash = mix(self.input_hash, self.frames);
        self.event_hash = mix(self.event_hash, self.frames.rotate_left(7));
        self.framebuffer_hash = mix(self.framebuffer_hash, self.frames.rotate_left(17));
    }

    pub fn summary_json(&self) -> String {
        format!(
            "{{\"frames\":{},\"input_hash\":{},\"event_hash\":{},\"framebuffer_hash\":{}}}",
            self.frames, self.input_hash, self.event_hash, self.framebuffer_hash
        )
    }
}

fn mix(seed: u64, v: u64) -> u64 {
    let mut x = seed ^ v.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 33;
    x = x.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    x ^= x >> 33;
    x = x.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    x ^ (x >> 33)
}

#[cfg(test)]
mod tests {
    use super::ReplayCapture;
    use crate::aurex::game::InputState;

    #[test]
    fn replay_capture_is_deterministic_for_same_sequence() {
        let mut a = ReplayCapture::new();
        let mut b = ReplayCapture::new();

        for f in 0..64u32 {
            let input = InputState {
                up: f % 2 == 0,
                down: f % 3 == 0,
                accept: f % 5 == 0,
                ..Default::default()
            };
            a.capture_input(input);
            b.capture_input(input);
            a.capture_event_tag(f as u64);
            b.capture_event_tag(f as u64);
            let fb = [f as u16; 128];
            a.capture_framebuffer(&fb);
            b.capture_framebuffer(&fb);
            a.end_frame();
            b.end_frame();
        }

        assert_eq!(a.summary_json(), b.summary_json());
    }

    #[test]
    fn replay_capture_changes_when_framebuffer_changes() {
        let mut a = ReplayCapture::new();
        let mut b = ReplayCapture::new();

        let input = InputState {
            accept: true,
            ..Default::default()
        };

        a.capture_input(input);
        b.capture_input(input);
        a.capture_event_tag(1);
        b.capture_event_tag(1);
        a.capture_framebuffer(&[1u16; 128]);
        b.capture_framebuffer(&[2u16; 128]);
        a.end_frame();
        b.end_frame();

        assert_ne!(a.summary_json(), b.summary_json());
    }
}
