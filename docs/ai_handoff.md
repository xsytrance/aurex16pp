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
