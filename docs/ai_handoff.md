Project Overview

Aurex-16++ is a deterministic 2D fantasy console platform built in Rust.

It is designed to:

Be hardware-inspired

Be LLM-friendly

Enforce strict constraints

Support trophies

Support guided game creation

Remain strictly 2D

It follows the RX philosophy:
Lightweight. Balanced. Constrained. Tuned. Deterministic.

Current Engine Status
Core Engine — LOCKED

The following systems are fully implemented and frozen:

CPU (VM-32)

200,000 ops per frame cap (hard enforced)

Deterministic 60 FPS model

No floating point in core VM

CPU cap violations tracked (cpu_rejects)

WRAM

512 KB hard-locked constant

Heap-backed allocation

Size enforced via constant

DMA

Max 4 commands per frame

Max 64 KB VRAM upload per frame

Max 16 KB audio upload per frame

Immediate rejection when caps exceeded

Reject telemetry tracked per frame

AudioRam explicitly separated from PPU

Any AudioRam access through PPU panics (debug enforcement)

Core tolerances must not be re-architected without explicit approval.

Hardware Canon
Memory

512 KB WRAM

1 MB VRAM (partitioned — formal layout pending finalization)

256 KB ASU sample RAM

Graphics (PPU-A16)

426×240 (16:9)

4 BG layers

BG2 reserved for Mode 7 affine plane

256 sprites (64 per scanline)

256 on-screen colors

15-bit palette (5:5:5)

Line effects tightly capped

Audio (ASU-816)

16 voices (8 PCM + 8 synth)

ADSR per voice

Built-in SEQ-16 sequencer

Echo + soft limiter

44.1 kHz stereo

ECSU

256 entity slots

Standardized layout

PDU

Tracks CPU usage

Tracks DMA usage

Tracks VRAM usage

Tracks audio usage

Tracks reject counts

AAS

Built-in achievement system

Unlock API

Persistent profile storage

GCU

Built-in Guided Creation Unit

Visible in Library

LLM-assisted cartridge creation

Architectural Rule

If new features contradict:

Determinism

2D-only philosophy

Hard hardware constraints

Canon caps (CPU, DMA, memory)

They must be rejected.

No silent budget forgiveness.
No deferred DMA.
No implicit scaling.

Current Development Phase

Core Engine + DMA complete and locked.

Next major milestone:

VRAM memory map formalization and PPU-A16 partition locking.

Development continues in strict build order.

---

PHASE: VRAM PARTITION LOCK
Status: COMPLETE

---

Total VRAM: 1 MB (0x100000 bytes)

Region A (0x00000–0x4FFFF) – BG Tile Patterns (BG0/BG1/BG3)
Region B (0x50000–0x5FFFF) – BG Tilemaps
Region C (0x60000–0x8FFFF) – Sprite Tile Patterns
Region D (0x90000–0x93FFF) – Sprite Tables
Region E (0x94000–0xA3FFF) – Mode 7 Map (BG2 only)
Region F (0xA4000–0xD3FFF) – Mode 7 Texture (BG2 only)
Region G (0xD4000–0xDBFFF) – Line Tables
Region H (0xDC000–0xFBFFF) – Cartridge General VRAM
Region I (0xFC000–0xFFFFF) – Reserved (inaccessible)

DMA transfers are region-bound.
A single DMA command may not exceed its region.
Reserved region writes are rejected.
Mode 7 (BG2) is restricted to its dedicated regions.

---

PHASE: PPU-A16 PIPELINE INIT
Status: IN PROGRESS

---

PPU device instantiated and integrated into Aurex motherboard.

Debug framebuffer renderer removed from frame loop.

New hardware render entry:
Aurex -> PPU::render_frame()

Current behavior:
PPU renders empty scanlines (black frame).

Next step:
Background fetch discipline (BG0 only).

---

PHASE: PPU-A16 BG0 ONLINE
Status: COMPLETE

---

BG0 scanline fetch implemented.

Pipeline:
Tilemap (64x64) ->
4bpp tile pattern fetch ->
Palette bank select ->
RGB555 framebuffer write

SDL host loop restored.
Console now renders via PPU device.

Temporary VRAM seed active (debug only).
To be removed when cartridge DMA uploads real assets.

PPU-A16 Formats (LOCKED)

BG tile encoding: 8×8, 4bpp packed, 32 bytes/tile, row = 4 bytes, each byte stores 2 pixels (hi nibble then lo nibble, left→right)

Tilemap entry: u16 little-endian

bits 0–9: tile index (0..1023)

bits 10–11: palette select (0..3) → bank\*16

bit 12: hflip

bit 13: vflip

bits 14–15: priority (reserved; ignored in v0.1)

Palette: first 256 entries = RGB555 u16 little-endian (2 bytes each)

Also note:

BG0 currently treats tilemap as 64×64 (wrap).

TEMP TEST VRAM seed exists in debug builds only.

PPU scroll registers introduced (BG0 only).
Scroll now controlled via CPU-side register write (temporary).
PPU no longer self-mutates state.

Phase 4 — OAM (Object Attribute Memory) Introduced

Sprite hardware memory layer has been implemented.

128 hardware sprite slots (MAX_SPRITES = 128)

Fixed 8x8 sprite assumption (Phase 1)

Sprite struct fields:

x

y

tile_index

palette

priority

visible

CPU-accessible write API via:

Ppu::write_sprite(...)

No rendering yet

No per-scanline sprite limits yet

No priority sorting yet

Architectural rule:
Memory exists before behavior.

Next milestone:
Scanline sprite renderer with strict 8-sprite-per-line hardware limit.

Phase 5 — Sprite Rendering + Priority

Sprite rendering is now functional.

Implemented:

8x8 sprite rendering (4bpp decode)

Palette lookup (RGB555)

Transparent pixel skip (color index 0)

Per-scanline sprite evaluation

Hard 8-sprite-per-line cap

Priority-based sprite sorting (low first, high last)

Current behavior:

Sprites overwrite background pixels

Sprite priority determines layering between sprites

Background currently treated as implicit lowest layer

No sprite/background priority bits yet

No blending

No overflow telemetry wired yet

Status:

PPU now supports:
BG0 + Sprites + Priority (basic)

Next planned milestone:
Sprite overflow flag exposure OR BG1 layer introduction.

Phase 6 — Sprite Overflow Telemetry

Implemented:

8 sprites per scanline limit

Overflow detection during scanline evaluation

Per-frame latch of overflow state

Count of overflowed scanlines

PPU → PDU telemetry bridge

Frame reset of telemetry

Hardware Behavior:

Sprites are evaluated per scanline.

Only first 8 visible sprites render.

If >8 detected, overflow flag is set.

Overflow persists for entire frame.

Overflow scanline count recorded.

Architecture:

PPU:

Owns overflow detection + latch

Resets at start of frame

Exposes read-only getters

PDU:

Ingests PPU telemetry per frame

Holds status for debugging / SDK / future register exposure

Temporary overflow validation test:

Seeded 12 sprites on one scanline

Confirmed 8-scanline overflow (sprite height = 8)

Test removed

Status:
Sprite pipeline now hardware-accurate.

Next milestone candidate:

Sprite vs BG priority interaction

BG1 layer

Sprite size modes

Hardware register mapping

OAM DMA pipeline

Graphics Pipeline Status

BG0: 8x8 tiles, 4bpp, 64x64 map

Sprites: 8x8, 4bpp

8 sprites per scanline

Overflow flag + per-frame counter

Priority sorting (low first)

Blend modes:

Normal

Additive (RGB555 channel clamp)

Then list:

Known limitations

No sprite flipping yet

No sprite scaling

No window layers

No per-layer priority

No transparency blend modes other than additive
