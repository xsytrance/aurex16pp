AUREX-16++ MASTER PROMPT
Canonical Architecture Document (v2.0)

1. Identity

Project: Aurex-16++ (aurex16pp)
Location: C:\Users\ageno\Apps\aurex16pp

Aurex-16++ is a deterministic 2D fantasy console inspired by the late 16-bit era and designed for AI-assisted cartridge creation.

It is:

Lightweight

Balanced

Disciplined

Tuned

Deterministic

Strictly 2D

It is not:

A 3D engine

Unity

A PS1 clone

A PC abstraction layer

It is a constrained 2D hardware fantasy platform.

2. Core Philosophy (Locked)

Aurex follows RX-car philosophy:

Tight tolerances

Hard caps

No silent forgiveness

Deterministic behavior

Integer math in core systems

Hardware-style rejection, not soft warnings

If hardware limits are exceeded:

Reject immediately.
Track the violation.
Never silently allow it.

3. Hardware Canon

These values are locked unless explicitly approved.

VM-32 (CPU Core)

32-bit internal registers

200,000 ops per frame (hard cap)

60 FPS deterministic

No floating point in core VM

CPU cap violations tracked (cpu_rejects)

Memory
WRAM

512 KB (hard-locked constant)

VRAM

1 MB total

Partitioned into:

BG tile data

Sprite tile data

Tilemaps

Palettes

Audio RAM

256 KB

Strictly separated from VRAM

Illegal routing panics immediately

4. PPU-A16 (Graphics Unit)
   Display

426×240 (16:9)

256 on-screen colors max

RGB555 (5:5:5)

Integer-only color math

Background System (Current Status)
BG0 (Implemented – Phase 1 Complete)

64×64 tilemap (wrap)

8×8 tiles

4bpp packed format

32 bytes per tile

4 bytes per row

Each byte = 2 pixels (hi nibble → lo nibble)

Tilemap Entry (u16, little-endian)

Bits 0–9: tile index (0..1023)

Bits 10–11: palette select (0..3 → bank ×16)

Bit 12: hflip

Bit 13: vflip

Bits 14–15: priority (reserved)

Palette

First 256 entries = RGB555 (u16 little-endian)

Deterministic integer math only

Sprite System (Phase 1 Complete)
Sprite Format

Each sprite contains:

x (u16)

y (u16)

tile_index (u16)

palette (u8)

priority (u8)

visible (bool)

blend (BlendMode)

Sprite Rendering Rules

8×8 tiles

4bpp packed

Color index 0 = transparent

Sorted by priority (low first)

Composited after BG0

Scanline Rules

8 sprites max per scanline

Additional sprites trigger overflow

Overflow tracking:

sprite_overflow_latched

sprite_overflow_scanlines

Blending System (Implemented)
Blend Modes

Normal

Additive

Additive Blending

RGB555 channel-wise add

Clamp per channel (0–31)

No overflow

No floats

Fully deterministic

Future PPU Expansion (Planned, Not Yet Implemented)

BG1–BG3

Mode 7 affine plane (BG2)

Sprite flipping

16×16 sprites

Layer priority mixing

Window layers

Extended blend modes

Line table effects (tightly capped)

These must preserve determinism and hardware caps.

5. DMA Controller

Max 4 commands per frame

Max 64 KB VRAM upload per frame

Max 16 KB audio upload per frame

Reject immediately if exceeded

Reject counts tracked per frame

No deferred DMA.
No silent forgiveness.

6. ASU-816 (Audio System)

16 voices (8 PCM + 8 synth)

256 KB sample RAM

ADSR per voice

Echo + soft limiter

44.1 kHz stereo

SEQ-16 built-in sequencer

Fully deterministic

7. TCU (Timing & Control Unit)

Frame counter

4 hardware timers

Deterministic RNG

Sync clock

8. ECSU (Entity System)

256 entity slots

Standardized position/velocity/state layout

Deterministic update cycle

9. PDU (Performance & Diagnostics Unit)

Tracks per frame:

CPU ops

DMA usage

VRAM usage

Audio usage

Reject counts

Sprite overflow

CPU cap violations

Enforces:

200,000 ops cap

10. AAS (Achievement Service)

Built-in trophy system

Unlock API

Persistent profile storage

11. GCU (Guided Creation Unit)

First-party Game Maker tool

Powers LLM-assisted cartridge creation

Must respect hardware limits

Cannot bypass caps

12. Development Order (Strict)

Core frame loop

PDU

WRAM scaffold

DMA controller

PPU-A16

ASU-816 + SEQ-16

ECSU + TCU

Cartridge system

Achievement Service

GCU

No skipping.
No midstream architectural rewrites.

13. Non-Negotiables

No 3D pipeline

No unlimited VRAM

No deferred DMA

No floating point in core rendering

No silent budget forgiveness

No hidden hardware bypasses

Violations must be rejected.

14. Current Milestone

PPU Phase 1: COMPLETE

The rendering pipeline is stable, deterministic, and hardware-consistent.

Future work must extend capability without breaking architectural constraints.
