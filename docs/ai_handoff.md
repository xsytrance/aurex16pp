AUREX-16++ AI HANDOFF DOCUMENT

Canonical Hardware Snapshot

1. Project Identity

Aurex-16++ is a deterministic 2D fantasy console inspired by late-era 16-bit hardware.

It is:

Hardware-constrained

Deterministic

Integer-only in core systems

Designed for AI-assisted cartridge creation

It is not:

A modern game engine

A PC abstraction layer

A free-form rendering system

All development must respect hardware canon.

2. Rendering System (Current State)

The rendering pipeline is stable and considered canonical.

2.1 Display

Resolution: 426×240 (16:9)

Color format: RGB555

256 on-screen colors max

Deterministic integer compositing

Locked 60 FPS

2.2 Background System
Shared Pattern Memory

4bpp packed

32 bytes per 8×8 tile

Stored in bg_tiles

Shared by BG0 and BG1

BG0

64×64 tilemap (wrap)

8×8 tiles

u16 little-endian tile entry

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

Per-scanline scroll table:

bg0_scroll_x_line[FB_H]

BG0 renders first.

BG1

64×64 tilemap (wrap)

Same format as BG0

Independent scroll registers:

bg1_scroll_x

bg1_scroll_y

Per-scanline scroll table:

bg1_scroll_x_line[FB_H]

BG1 renders after BG0.

Non-transparent BG1 pixels overwrite BG0.

BG priority bit interleaves with sprites.

2.3 Window System

Vertical window masking supported.

Registers:

window_enabled

window_top

window_bottom

Behavior:

BG1 can be masked per scanline

BG0 unaffected

Sprites unaffected

Deterministic

No horizontal windowing yet

2.4 Layer Enable Flags

Per-layer control:

bg0_enable

bg1_enable

sprite_enable

These flags gate rendering blocks inside render_scanline.

Purpose:

Debug isolation

Deterministic compositing control

Future register abstraction

SDK readiness

2.5 Sprite System

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

16×16 Sprite Support

2×2 tile composition

4 consecutive tiles

Layout:

[ base base+1 ]
[ base+2 base+3 ]

Flip logic applies across the full composite.

No tile duplication.
No new VRAM layout.
Deterministic integer-only decode.

2.6 Sprite Scanline Rules

Evaluated per scanline

Maximum 8 sprites per scanline

Overflow telemetry tracked

Overflow telemetry:

sprite_overflow_latched

sprite_overflow_scanlines

Deterministic ordering enforced.

2.7 Priority Rules
BG Priority

Per-tile priority bit (bit 14)

Transparent BG pixels do not block sprites

Sprite Priority

Resolution rules:

High-priority BG blocks low-priority sprite

High-priority sprite always wins

Transparent BG never blocks sprite

2.8 Blending

Supported modes:

Normal

Additive

Additive blending:

Channel-wise RGB555 addition

Per-channel clamp (0–31)

Integer-only

Implemented via add_rgb555

2.9 Rendering Order

Per scanline:

BG0 (if enabled)

BG1 (if enabled + window pass)

Sprites (if enabled)

Additive blending applied during sprite pass

All compositing is deterministic and integer-only.

3. Core Hardware Constraints (Locked)
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

4. Systems Operational

Deterministic frame lifecycle

PDU telemetry

CPU cap enforcement

DMA enforcement

Dual BG layering

Per-scanline scroll effects

Vertical window masking

Layer enable gating

Sprite flipping

16×16 sprite decode

BG ↔ Sprite priority interleave

Additive blending

Rendering pipeline is stable.

5. Known Limitations (Intentional)

Not yet implemented:

Horizontal window clipping

Sprite window masking

Mode 7

Alpha blending

Per-layer blending modes

Sub-scanline effects

Mosaic / distortion hardware

Register abstraction layer (scroll/window via CPU)

All future features must preserve determinism.

6. Architectural Guardrails

Do NOT:

Introduce floating point into PPU core

Break 60 FPS determinism

Remove sprite scanline cap

Remove overflow telemetry

Introduce unlimited VRAM access

Add deferred DMA

Ignore hardware limits silently

All expansions must feel like hardware.

7. Development Status

Rendering architecture is:

Clean

Deterministic

Layered

Composable

Hardware-constrained

This is a stable checkpoint.

Future work must extend capability without destabilizing core systems.
