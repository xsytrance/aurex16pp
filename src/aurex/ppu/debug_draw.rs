// ============================================================================
// PPU Debug Draw (Phase 4.5)
// ----------------------------------------------------------------------------
// Temporary internal drawing utilities for framebuffer validation.
//
// IMPORTANT:
// - This is NOT final rendering logic.
// - Used only to validate framebuffer pipeline before tile engine exists.
// - May be removed or gated behind debug config later.
// ============================================================================

use super::framebuffer::{FB_H, FB_W, Framebuffer};

pub fn draw_test_pattern(fb: &mut Framebuffer, frame: u64) {
    let pixels = fb.pixels_mut();

    // Moving vertical bar
    let bar_x = (frame as usize / 4) % FB_W;

    for y in 0..FB_H {
        for x in 0..FB_W {
            let idx = y * FB_W + x;

            if x == bar_x {
                pixels[idx] = 0b0_11111_00000_00000; // red
            } else if y % 40 == 0 {
                pixels[idx] = 0b0_00000_11111_00000; // green horizontal lines
            } else {
                // subtle blue gradient
                let blue = ((x * 31) / FB_W) as u16;
                pixels[idx] = blue;
            }
        }
    }
}
