// ============================================================================
// OAM (Object Attribute Memory)
// ----------------------------------------------------------------------------
// Sprite descriptor memory for AUREX-16++
//
// No rendering yet.
// Hardware limits enforced.
// ============================================================================

pub const MAX_SPRITES: usize = 128;

// Sprite size is fixed to 8x8 for Phase 1.
// Later we add size flags.
#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    pub x: u16,
    pub y: u16,
    pub tile_index: u16,
    pub palette: u8,
    pub priority: u8,
    pub visible: bool,
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
