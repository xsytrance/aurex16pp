// ============================================================================
// PPU-A16 Framebuffer (Phase 4)
// ----------------------------------------------------------------------------
// Native resolution: 426×240 (16:9).
// Pixel format: 15-bit color stored in u16 as 0b0RRRRRGGGGGBBBBB (5:5:5).
//
// IMPORTANT:
// - This is internal console format, not host RGBA.
// - No rendering pipeline yet; only clear/fill operations.
// - SDL presentation comes later.
// ============================================================================

pub const FB_W: usize = 426;
pub const FB_H: usize = 240;
pub const FB_PIXELS: usize = FB_W * FB_H;

/// 15-bit color helper: 5 bits each for R, G, B.
/// Stored in u16 as: 0RRRRRGGGGGBBBBB (top bit unused).
#[inline]
pub fn rgb555(r: u8, g: u8, b: u8) -> u16 {
    let r5 = (r as u16) & 0x1F;
    let g5 = (g as u16) & 0x1F;
    let b5 = (b as u16) & 0x1F;
    (r5 << 10) | (g5 << 5) | b5
}

pub struct Framebuffer {
    // Heap allocated to avoid stack overflows on Windows.
    pixels: Box<[u16]>,
}

impl Framebuffer {
    pub fn new() -> Self {
        let fb = Self {
            pixels: vec![0u16; FB_PIXELS].into_boxed_slice(),
        };
        debug_assert_eq!(fb.pixels.len(), FB_PIXELS);
        fb
    }

    pub fn clear(&mut self, color: u16) {
        // Likely to evolve:
        // - fast clear paths
        // - partial clears / scissor
        for p in self.pixels.iter_mut() {
            *p = color;
        }
    }

    pub fn pixels(&self) -> &[u16] {
        &self.pixels
    }

    pub fn pixels_mut(&mut self) -> &mut [u16] {
        &mut self.pixels
    }
}
