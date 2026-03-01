// ============================================================================
// WRAM (512 KB) - fixed-size system memory
// ----------------------------------------------------------------------------
// Stored on heap via Vec->Box<[u8]> to avoid stack issues on Windows.
// ============================================================================

const WRAM_BYTES: usize = 512 * 1024;

pub struct Wram {
    memory: Box<[u8]>,
}

impl Wram {
    pub fn new() -> Self {
        let w = Self {
            memory: vec![0u8; WRAM_BYTES].into_boxed_slice(),
        };
        debug_assert_eq!(w.memory.len(), WRAM_BYTES);
        w
    }
}
