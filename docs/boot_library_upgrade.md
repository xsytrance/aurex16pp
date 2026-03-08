# Boot + Library Audiovisual Upgrade Notes

_Date: 2026-03-08 (PrimeAwakens era)._ 

## Overview

This document tracks the current AV direction after replacing the old boot scene and stabilizing ASU-32 boot audio behavior.

## Boot scene (`PrimeAwakens`)

File: `src/aurex/boot/prime_awakens.rs`

### Visual stack
- Quantum sky plasma backdrop.
- Rasterized neon sun banding.
- Perspective horizon grid with lane sway.
- Animated spectrum towers synchronized by deterministic phase logic.
- Layered title + prompt overlays with integer-only cadence blinking.

### Input/flow behavior
- Boot remains overlay-driven while flow controller manages start transition.
- Prompt alternates between warmup and start-call text based on `waiting_for_start`.

### Determinism constraints
- Integer-only math paths.
- No frame-time-dependent interpolation.
- No dynamic allocations in per-pixel draw loops.

## Library scene

File: `src/aurex/game/library.rs`

- Rich per-title metadata display (`cartridge_id`, `bpm`, `style`, `tag`).
- Deterministic animated backdrop and card shimmer.
- Launch request/cancel edge behavior feeds typed launch lifecycle events.
- Audio cue mapping routes to runtime audio commands (`PlayTrack`, `PlaySfx`).

## Audio alignment notes

Primary audio file: `src/aurex/runtime/audio.rs`

- Boot mode has dedicated sequencing branch.
- Recent stabilization removed boot fuzz by reducing unnecessary envelope retrigger + over-dense percussion grit.
- Game mode sequencing remains separate and track-driven.

## Next AV polish opportunities

1. Tie select visual pulses to deterministic boot sequencer step index (shared beat clock).
2. Add optional low-cost scanline glow pass in overlay layer only.
3. Introduce per-title launch micro-stingers keyed by `cartridge_id` tag groups.
