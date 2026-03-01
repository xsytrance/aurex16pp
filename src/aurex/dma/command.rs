#[derive(Clone, Debug)]
pub enum DmaTarget {
    Vram,     // PPU-A16 upload path (later)
    AudioRam, // ASU-816 sample upload path (later)
}

#[derive(Clone, Debug)]
pub struct DmaCommand {
    pub target: DmaTarget,
    pub bytes: u32,
    // Later: src addr, dst addr, etc.
}

impl DmaCommand {
    pub fn vram_upload(bytes: u32) -> Self {
        Self {
            target: DmaTarget::Vram,
            bytes,
        }
    }

    pub fn audio_upload(bytes: u32) -> Self {
        Self {
            target: DmaTarget::AudioRam,
            bytes,
        }
    }
}
