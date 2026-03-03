# AUREX-16++ ARCHITECTURE INDEX

Purpose:
Fast navigation to critical engine systems.
Updated only when structural changes occur.

---

## CORE FRAME LOOP

File:

- src/aurex/mod.rs

Important Functions:

- run_frame()
- tick()
- render_frame()

---

## PPU SYSTEM

File:

- src/aurex/ppu/ppu.rs

Core Struct:

- struct Ppu

Hot Functions:

- render_frame()
- render_scanline()
- write_sprite()

Mutation Points:

- BG scroll registers
- Window registers
- Layer enable flags
- Sprite flip logic

---

## VRAM SYSTEM

File:

- src/aurex/ppu/vram.rs

Core Struct:

- struct Vram

Locked Regions:

- BG pattern memory
- BG0 tilemap
- BG1 tilemap
- Sprite tiles
- Palettes

Do NOT re-architect layout.

---

## OAM SYSTEM

File:

- src/aurex/ppu/oam.rs

Core Struct:

- struct Sprite
- struct Oam

Likely to evolve:

- Sprite attributes
- Priority rules
- Blending modes

---

## DMA SYSTEM

File:

- src/aurex/dma/

Core:

- DmaController
- VramRegion

Hardware limits locked.

---

## AUDIO SYSTEM (ASU)

File:

- src/aurex/asu/

Core:

- Voice
- Envelope
- Sequencer

Integer-only audio logic required.

---

## HARDWARE LOCK POINTS

- VRAM_TOTAL_BYTES
- 200k ops cap
- 8 sprites per scanline
- 60 FPS deterministic loop
- RGB555 compositing
- Integer-only PPU core

## FILE RESPONSIBILITY LOCK

src/aurex/mod.rs

- Owns frame loop
- Owns system orchestration
- Does NOT mutate PPU internals

src/aurex/ppu/ppu.rs

- Owns all PPU state
- Owns render_frame
- Owns register writes
- Owns scanline pipeline

src/aurex/ppu/oam.rs

- Owns sprite storage
- No rendering logic

src/aurex/ppu/vram.rs

- Owns VRAM layout
- No rendering logic
- Canonical memory partition

END OF INDEX
