AUREX-16++ MASTER PROMPT (Rebuild v1.1)
Identity

Project: Aurex-16++ (aurex16pp)
Location: C:\Users\ageno\Apps\aurex16pp

Aurex-16++ is a fantasy 2D console inspired by the late 16-bit era, designed with modern AI-assisted development in mind.

It follows the RX car philosophy:

Lightweight

Balanced

Disciplined

Tuned

Deterministic

2D-only

It is not a 3D engine.
It is not Unity.
It is not a PS1 clone.
It is not a PC abstraction layer.

It is a constrained 2D hardware fantasy platform.

Hardware Canon (Locked)
VM-32

32-bit internal registers

200,000 ops per frame (hard cap)

60 FPS deterministic

512 KB WRAM (hard-locked constant)

No floats in core VM

PPU-A16

426×240 (16:9)

4 BG layers

BG2 = Mode 7 affine plane

256 sprites (64 per scanline)

256 on-screen colors

15-bit palette (5:5:5)

Color math supported

1 MB VRAM (partitioned)

Line table effects (tightly capped)

DMA

Max 4 commands per frame

Max 64 KB VRAM upload per frame

Max 16 KB audio upload per frame

Exceeding caps → rejected immediately

Reject counts tracked per frame

Audio RAM strictly separated from PPU VRAM

ASU-816

16 voices (8 PCM + 8 synth)

256 KB sample RAM

ADSR per voice

Echo + soft limiter

44.1 kHz stereo

SEQ-16 built-in sequencer

TCU

Frame counter

4 timers

Deterministic RNG

Sync clock

ECSU

256 entity slots

Standardized position/velocity/state layout

PDU

Tracks CPU ops

Tracks DMA usage

Tracks VRAM usage

Tracks audio usage

Tracks reject counts

Enforces 200,000 ops cap

CPU cap violations tracked (cpu_rejects)

AAS (Achievement Service)

Built-in trophy system

Unlock API

Persistent profile storage

GCU (Guided Creation Unit)

Visible in Library as first-party Game Maker tool

Powers LLM-assisted cartridge creation

Must remain constrained and hardware-aware

Engine Core Status (Milestone Locked)

The following systems are fully implemented and considered frozen:

WRAM hard-locked to 512 KB

CPU 200,000 ops/frame enforced

CPU reject telemetry active

DMA caps fully enforced (commands, VRAM, audio)

AudioRam separated from PPU; illegal routing panics

Deterministic frame lifecycle operational

Core engine tolerances must not be re-architected unless explicitly approved.

Development Order (Strict)

Core frame loop + VM stub

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
No architectural rewrites midstream.

Non-Negotiables

No 3D pipeline

No unlimited VRAM

No deferred DMA

No silent budget forgiveness

No silent hardware cap bypass

Violations must be rejected, not ignored.
