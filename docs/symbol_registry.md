AUREX-16++ SYMBOL REGISTRY
Baseline Lock v1.1 (Full Public Snapshot)

Source: full_pub_snapshot.txt
This document locks the complete public symbol surface of Aurex-16++.

All symbols listed here are ABI-level stable.
Renaming requires explicit breaking-change approval.

---

1️⃣ Public Module Exports

From src/aurex/mod.rs:

boot

clock

dma

pdu

ppu

vm32

wram

These module names are locked.

2️⃣ Public Constants
wram.rs

WRAM_SIZE: usize

3️⃣ Public Structs

Clock

Aurex

Pdu

Wram

RenderProbe

These struct names are locked.

4️⃣ Public Constructors

Clock::new()

Aurex::new()

Pdu::new()

Wram::new()

RenderProbe::new()

Constructor naming is locked.

5️⃣ Public System Control APIs
Aurex

run(&mut self) -> !

run_frame(&mut self)

framebuffer(&self) -> &Framebuffer

Clock

begin_frame(&mut self)

end_frame(&self)

Pdu

frame_index(&self) -> u64

begin_frame(&mut self)

consume(&mut self, ops: u32) -> bool

ingest_dma(&mut self, commands: u32, vram_bytes: u32, audio_bytes: u32, rejects: u32)

end_frame(&mut self)

ingest_ppu(&mut self, sprite_overflow: bool, overflow_scanlines: u32)

Wram

len(&self) -> usize

memory(&self) -> &[u8]

memory_mut(&mut self) -> &mut [u8]

RenderProbe

update(&mut self, ppu: &mut Ppu, dma: &mut DmaController, wram: &mut Wram)

6️⃣ Notes on Current Public Surface

No public enums detected.

No public traits detected.

No public type aliases detected.

No public re-exports detected.

Public API surface is intentionally small and controlled.

This is desirable for hardware-style stability.

END OF REGISTRY
