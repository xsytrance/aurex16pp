1. Project Identity

Aurex-16++ is a deterministic 2D fantasy console inspired by late 16-bit hardware.

It is:

Hardware-constrained

Deterministic

Integer-only in core systems

Designed for AI-assisted cartridge creation

It is not:

A modern engine

A PC abstraction layer

A free-form rendering system

All development must respect hardware canon.

2. Current Stable Milestone
   ✅ PPU Phase 2 — COMPLETE

Rendering pipeline is stable.

The following systems are considered functional and safe:

Dual background layers

Sprite flipping

16×16 sprites

Sprite ↔ BG priority

Deterministic compositing

Architecture must not be rewritten.

3. Graphics System
   3.1 Display

Resolution: 426×240 (16:9)

Color format: RGB555

256 on-screen colors max

Deterministic integer math only

60 FPS locked

3.2 Background Layers
Shared Pattern Memory

4bpp packed

32 bytes per tile

Stored in bg_tiles

Shared by BG0 and BG1

BG0

64×64 tilemap (wrap)

8×8 tiles

Tilemap entry: u16 little-endian

Bit layout:

bits 0–9: tile index

bits 10–11: palette select

bit 12: hflip

bit 13: vflip

bit 14: priority bit

bit 15: reserved

Scroll registers:

bg0_scroll_x

bg0_scroll_y

Rendered first.

BG1

64×64 tilemap (wrap)

Same format as BG0

Independent scroll registers:

bg1_scroll_x

bg1_scroll_y

Rendered after BG0.

Overwrites BG0 where non-transparent.

Priority bit respected in sprite interleave.

3.3 Sprite System

Each sprite contains:

x (u16)

y (u16)

tile_index (u16)

palette (u8)

priority (u8)

visible (bool)

hflip (bool)

vflip (bool)

size_16 (bool)

blend (BlendMode)

Sprite Tile Format

8×8

4bpp packed

32 bytes per tile

Color index 0 = transparent

16×16 Support

2×2 tile composition

4 consecutive tiles

Flip logic applies across full composite

No duplication of tile memory

Deterministic integer-only decode

4. Rendering Pipeline
   4.1 Per-Scanline Order

BG0

BG1

Sprites

4.2 Sprite Scanline Evaluation

Evaluated per scanline

Max 8 sprites per scanline

Overflow triggers telemetry

Overflow Tracking

sprite_overflow_latched (bool per frame)

sprite_overflow_scanlines (u32 per frame)

4.3 Priority Rules
BG Priority

Per-tile priority bit (bit 14)

Transparent BG pixels do not block sprites

Sprite Priority

Resolution rule:

High-priority BG blocks low-priority sprite

High-priority sprite always wins

Transparent BG never blocks sprite

4.4 Blending

Supported blend modes:

Normal

Additive

Additive blending:

Channel-wise RGB555 addition

Per-channel clamp (0–31)

Integer-only

Implemented via add_rgb555

5. Core Hardware Constraints (Locked)
   VM-32

200,000 ops per frame (hard cap)

CPU reject tracking active

Memory

512 KB WRAM (locked)

1 MB VRAM (partitioned, canonical layout)

256 KB Audio RAM

No cross-routing

DMA

4 commands per frame max

64 KB VRAM upload per frame max

16 KB audio upload per frame max

Immediate rejection if exceeded

No silent forgiveness

6. Systems Operational

Deterministic frame lifecycle

PDU telemetry

CPU cap enforcement

DMA enforcement

Sprite overflow telemetry

Dual BG layering

Sprite flipping

16×16 sprite decode

BG priority interleave

7. Known Limitations (Intentional)

Not yet implemented:

Mode 7

Window layers

Alpha blending

Per-layer blending modes

Sub-scanline effects

Mosaic / distortion

Parallax registers

All future features must preserve determinism.

8. Architectural Guardrails

Do NOT:

Introduce floating point into PPU core

Break 60 FPS determinism

Remove sprite scanline cap

Remove overflow tracking

Introduce unlimited VRAM access

Add deferred DMA

Ignore hardware limits silently

All expansions must feel like hardware.

9. Next Recommended Milestones

BG1 actual render duplication (if not completed)

Parallax per scanline

Window masking

Mode 7 deterministic implementation

Performance validation passes

VRAM DMA stress testing

All upgrades must be incremental.

No sweeping rewrites.

10. Development Status

Rendering pipeline is stable.

Architecture is clean.

This is a safe checkpoint.

Future work must extend capability without destabilizing core systems.
