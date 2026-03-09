#![allow(dead_code)]
// ============================================================================
// OAM (Object Attribute Memory)
// ----------------------------------------------------------------------------
// Sprite descriptor memory for AUREX-16++
//
// No rendering yet.
// Hardware limits enforced.
// ============================================================================

pub const MAX_SPRITES: usize = 128;

// ============================================================================
// Sprite Blend Mode
// ----------------------------------------------------------------------------
// Normal   : Standard overwrite rendering
// Additive : RGB555 channel-wise additive blending (clamped)
// ============================================================================
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Additive,
}

// Sprite size is fixed to 8x8 for Phase 1.
// Later we add size flags.
#[derive(Clone, Copy, Debug)]
// -----------------------------------------------------------------------------
// Sprite (Phase 2)
// Now supports 8x8 and 16x16 using 2x2 tile composition
// -----------------------------------------------------------------------------
pub struct Sprite {
    pub x: u16,
    pub y: u16,
    pub tile_index: u16,
    pub palette: u16,
    pub priority: u8,
    pub visible: bool,

    // Flip flags (Phase 2)
    pub hflip: bool,
    pub vflip: bool,
    // Size flag (false = 8x8, true = 16x16)
    pub size_16: bool,

    // ------------------------------------------------------------------------
    // Blend mode (default: Normal)
    // ------------------------------------------------------------------------
    pub blend: BlendMode,
    // ------------------------------------------------------------------------
    // Sprite size flag (Phase 3)
    // false = 8x8
    // true  = 16x16 (4 tiles)
    // ------------------------------------------------------------------------
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            tile_index: 0,
            palette: 0,
            priority: 0,
            visible: false,
            hflip: false,
            vflip: false,
            size_16: false,
            blend: BlendMode::Normal,
        }
    }
}

pub struct Oam {
    sprites: [Sprite; MAX_SPRITES],
}

impl Oam {
    pub fn new() -> Self {
        Self {
            sprites: [Sprite::default(); MAX_SPRITES],
        }
    }

    pub fn sprite(&self, index: usize) -> Option<&Sprite> {
        self.sprites.get(index)
    }

    pub fn sprite_mut(&mut self, index: usize) -> Option<&mut Sprite> {
        self.sprites.get_mut(index)
    }

    pub fn len(&self) -> usize {
        MAX_SPRITES
    }
}
