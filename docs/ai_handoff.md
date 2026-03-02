AUREX-16++ AI HANDOFF DOCUMENT
Milestone: PPU Phase 1 Complete
Project Identity

Project: Aurex-16++
Repository: aurex16pp
Location: C:\Users\ageno\Apps\aurex16pp

Aurex-16++ is a deterministic 2D fantasy console inspired by late 16-bit hardware, built with strict hardware-style constraints and AI-assisted cartridge creation in mind.

It is not a modern engine.
It is not a PC abstraction.
It is a constrained fantasy console.

All work must respect hardware canon.

Current Stable Milestone
PPU Phase 1 — COMPLETE

The rendering pipeline is stable and considered frozen at this milestone.

Graphics System — Current State
Display

Resolution: 426×240 (16:9)

RGB555 (5:5:5)

256 on-screen colors max

Deterministic integer math only

BG0

64×64 tilemap (wrap)

8×8 tiles

4bpp packed (32 bytes per tile)

Tilemap entry: u16 little-endian

bits 0–9: tile index

bits 10–11: palette select

bit 12: hflip

bit 13: vflip

bits 14–15: reserved

Rendered first each scanline.

Scroll registers:

bg0_scroll_x

bg0_scroll_y

Sprite System

Each sprite contains:

x (u16)

y (u16)

tile_index (u16)

palette (u8)

priority (u8)

visible (bool)

blend (BlendMode)

Sprite tile format:

8×8

4bpp packed

32 bytes per tile

Color index 0 = transparent

Sprite Pipeline
Scanline Evaluation

Evaluated per scanline

Max 8 sprites per scanline

Additional sprites trigger overflow

Overflow Tracking

sprite_overflow_latched (bool per frame)

sprite_overflow_scanlines (u32 per frame)

Sorting

Sprites sorted by priority (low first)

Composited after BG0

Blending

Supported blend modes:

Normal

Additive

Additive blending:

Channel-wise RGB555 add

Clamp per channel (0–31)

Deterministic

Integer-only

No floating point

Implemented via add_rgb555

Engine Core Constraints (Locked)

VM-32:

200,000 ops per frame

Hard cap

CPU reject tracking active

Memory:

512 KB WRAM (locked)

1 MB VRAM (partitioned)

256 KB Audio RAM

No cross-routing allowed

DMA:

4 commands per frame max

64 KB VRAM upload per frame max

16 KB audio upload per frame max

Reject immediately if exceeded

No silent forgiveness.

Systems Operational

Deterministic frame lifecycle

PDU telemetry

CPU cap enforcement

DMA enforcement

Sprite overflow telemetry

Additive blending compositing

Known Limitations (Intentional)

Not yet implemented:

Sprite flipping

16×16 sprites

Multi-layer BG

Mode 7

Window layers

Alpha blending

Per-layer priority mixing

These are future expansions and must preserve determinism.

Architectural Guardrails

Do NOT:

Introduce floating point into PPU core

Break 60 FPS determinism

Remove sprite scanline cap

Remove overflow tracking

Introduce unlimited VRAM access

Add deferred DMA

Silently ignore hardware limits

All expansions must respect hardware fantasy constraints.

Next Recommended Milestones

Sprite flipping (hflip/vflip)

16×16 sprite support

Register abstraction for scroll

Additional BG layers

Performance validation passes

VRAM DMA correctness testing

Each should be incremental.
No sweeping rewrites.

Development Status

Rendering pipeline is stable.

This is a safe checkpoint.

Future work must extend capability without destabilizing architecture.
