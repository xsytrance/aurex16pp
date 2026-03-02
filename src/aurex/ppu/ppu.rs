// ============================================================================
// PPU-A16 (v0.1)
// ----------------------------------------------------------------------------
// Hardware-accurate render entry point.
// Scanline-based deterministic pipeline.
// No blending, no priority yet.
// ============================================================================

use super::framebuffer::{FB_H, Framebuffer};

pub struct Ppu {
    frame_counter: u64,
}

impl Ppu {
    pub fn new() -> Self {
        Self { frame_counter: 0 }
    }

    // -------------------------------------------------------------------------
    // FRAME ENTRY
    // -------------------------------------------------------------------------
    pub fn render_frame(&mut self, fb: &mut Framebuffer) {
        for y in 0..FB_H {
            self.render_scanline(y, fb);
        }

        self.frame_counter += 1;
    }

    // -------------------------------------------------------------------------
    // SCANLINE STUB
    // -------------------------------------------------------------------------
    fn render_scanline(&mut self, _y: usize, _fb: &mut Framebuffer) {
        // Background + sprite pipeline will be implemented here.
        // Currently empty — debug draw still active.
    }
}
