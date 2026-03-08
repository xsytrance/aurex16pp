# Boot + Library Audiovisual Upgrade (EDM/Hypnotic Pass)

Date: 2026-03-08

## Goals

- Upgrade boot into a hypnotic, rhythm-forward intro inspired by arcade-era EDM energy.
- Upgrade the library so each dummy title has a distinct visual/music identity.
- Keep all behavior deterministic and integer-only in the render/audio loops.

## Boot Intro Changes (`PrimeIgnition`)

File: `src/aurex/boot/prime_ignition.rs`

### Visual choreography

- Replaced the old static rail/meter look with a layered sequence:
  - Animated radial/tunnel backdrop with stripe/ring modulation.
  - Expanding rectangular tunnel overlays to simulate forward motion.
  - Wider equalizer band with 24 bars and phase-shifted height pulses.
  - Updated text stack:
    - `AUREX-16++`
    - `NEON IGNITION`
    - `EDM CORE ONLINE`
  - Waiting prompt now flashes as: `PRESS START // GO STRAIGHT`.

### Constraints honored

- Deterministic integer math only.
- No architecture changes to PPU core or DMA rules.
- Boot remains an overlay-driven visualization while runtime flow controls scene transition.

## Library Upgrade (`LibraryScreen`)

File: `src/aurex/game/library.rs`

### Title profile expansion

Each profile now includes:

- `track_id`
- `bpm`
- `style` (music/identity descriptor)
- `tag` (art direction phrase)
- per-title color theme and icon

### Visual upgrades

- New animated backdrop blending phase/cross modulation for a denser “club grid” look.
- Header now displays selected title’s `style` and `tag`.
- Cards now show:
  - stronger cover plate shimmer
  - icon block
  - title
  - style label
  - right-side pulse bars
- Footer meter now reads the selected title BPM and drives bar movement with BPM-influenced phase.
- Idle status text upgraded to: `PROFILE READY // EDM BANK ARMED`.

## Per-title Music Upgrade (ASU-32)

File: `src/aurex/runtime/audio.rs`

### Track bank changes

- Expanded selectable track bank from 4 to 6 tracks.
- Added new deterministic patterns:
  - `TRACK4`
  - `TRACK5`
- Updated track selection command handling:
  - `PlayTrack(track_id)` now maps with `% 6`.

This allows each of the six library entries to route to a unique track id without collapsing onto only four patterns.

## Notes for future polish

- If we want stronger “drum machine” identity, add a dedicated kick/snare lane driven by a compact step-mask table.
- Add optional per-title intro SFX tags so confirm/launch can be timbre-variant by cartridge profile.
