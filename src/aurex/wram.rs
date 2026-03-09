#![allow(dead_code)]
// ============================================================================
// WRAM (512 KB) - fixed-size system memory
// ----------------------------------------------------------------------------
// Stored on heap via Vec->Box<[u8]> to avoid stack issues on Windows.
// ============================================================================

pub const WRAM_SIZE: usize = 512 * 1024;

pub struct Wram {
    memory: Box<[u8]>,
}

impl Wram {
    pub fn new() -> Self {
        let w = Self {
            memory: vec![0u8; WRAM_SIZE].into_boxed_slice(),
        };
        debug_assert_eq!(w.memory.len(), WRAM_SIZE);
        w
    }

    pub fn len(&self) -> usize {
        self.memory.len()
    }

    pub fn memory(&self) -> &[u8] {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut [u8] {
        &mut self.memory
    }
}
