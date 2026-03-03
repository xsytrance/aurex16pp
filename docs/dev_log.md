AUREX-16++ DEVELOPMENT LOG

Reverse Chronological Engineering Record

Newest entries are always added at the top.
This file tracks engineering evolution, not canonical hardware state.
Refer to ai_handoff_canon.md for current hardware truth.

## [YYYY-MM-DD] — Boot DMA + Sprite Format Validation

- Implemented PrimeIgnition boot module.
- Verified DMA request() → apply() → VBlank gating path.
- Confirmed sprite tiles use 4bpp linear nibble-packed format.
- Corrected earlier planar assumption.
- Successfully rendered first DMA-uploaded glyph tile.
- Identified palette initialization as next required visual foundation step.

2026-03-02
PPU Phase 6.5 — VBlank Gating for VRAM DMA

- DMA apply() now requires vblank=true
- Outside VBlank, writes are silently rejected
- No timing granularity added
- No IRQ added
- Determinism preserved

## 2026-03-02 — PPU VBlank Simulation Introduced

Added a hardware-style `vblank` boolean to the PPU.

- Cleared at start of `render_frame`
- Set true after scanline rendering completes
- No IRQ system yet
- No behavior change

This establishes future-safe DMA gating and proper console timing architecture.

Rendering pipeline remains unchanged and deterministic.

## 2026-03-02 — Hardware Register Bus + Mutation Isolation

Register system fully activated and enforced.

- Address-based PPU register writes live.
- Frame logic now mutates PPU via bus only.
- Direct field mutation removed.
- Debug register driver isolated.
- Rendering pipeline untouched.

System now reflects real hardware layering.

Stable milestone.

## 2026-03-02 — PPU Register Bus Activated (Address-Based Writes Live)

### Summary

PPU register system elevated from enum-only API to hardware-style address bus.

Rendering logic now mutates state exclusively through address-based register writes.

### What Changed

- Added PPU register address map.
- Implemented `write_addr(addr, value)` and `read_addr(addr)`.
- Frame logic now uses address-based writes instead of direct field mutation.
- Scroll auto-increment now reads from and writes to register bus.
- Window control now flows through register bus.

### Architectural Impact

Mutation hierarchy is now:

Frame Logic  
→ Address Bus  
→ write_reg  
→ Internal PPU fields

This prepares Aurex for:

- CPU bus emulation
- Cartridge-driven register writes
- Save-state stability
- Deterministic replay
- Proper hardware layering

No rendering behavior changed.

Pipeline remains deterministic and integer-only.

Stable checkpoint.

2026-03-02 — PPU Phase 5 — Global Sprite Flip + Layer Controls Stabilized
Summary

Completed full global flip logic for sprites and formalized layer enable controls. Rendering pipeline is now multi-layer capable, composite-safe, and fully deterministic under hardware constraints.

Major Additions

Global hflip and vflip support for:

8×8 sprites

16×16 composite sprites (2×2 tile layout)

Flip applied across full composite before tile selection

No tile memory duplication

Deterministic coordinate remapping

No OAM leakage — flip integrated into PPU API

API Change

write_sprite signature expanded:

write_sprite(
index,
x,
y,
tile_index,
palette,
priority,
size_16,
hflip,
vflip,
)

Sprite state mutation now occurs only through PPU interface.

Layer Control Stabilized

bg0_enable

bg1_enable

sprite_enable

Allows deterministic layer isolation and debug gating.

Rendering Integrity

RGB555 preserved

Integer-only compositing

8 sprites per scanline enforced

Overflow telemetry preserved

Scanline render order unchanged:

BG0

BG1 (window-masked)

Sprites

Additive blending during sprite pass

Architecture Status

Rendering pipeline is now:

Dual-layer capable

Window-masked

Per-scanline scroll capable

Multi-size sprite capable

Global flip correct

Deterministic under hardware caps

Stable checkpoint.

2026-03-02 — Rendering Elevation Tier Stabilized
Summary

Rendering pipeline expanded beyond initial baseline while preserving strict hardware constraints and determinism.

System now supports:

Dual background layers (BG0 + BG1)

Per-scanline scroll tables

Vertical window masking

Per-layer enable flags

16×16 sprites

Full sprite flipping (hflip / vflip)

Sprite ↔ BG priority interleave

Additive RGB555 blending

8 sprites per scanline enforcement

Overflow telemetry

Architecture remains deterministic and integer-only.

Dual Background System

Implemented:

BG0 (64×64 tilemap)

BG1 (64×64 tilemap)

Shared 4bpp pattern memory

Independent scroll registers

Per-tile priority bit (bit 14)

BG1 renders after BG0 and overwrites non-transparent pixels.

Per-Scanline Scroll Tables

Added:

bg0_scroll_x_line[FB_H]

bg1_scroll_x_line[FB_H]

Enables:

Raster distortion

Wave effects

Parallax motion

Integer-only math preserved.

Vertical Window Masking

Added:

window_enabled

window_top

window_bottom

BG1 can be vertically clipped per scanline.

Sprites unaffected.

Horizontal windowing not yet implemented.

Layer Enable Flags

Added:

bg0_enable

bg1_enable

sprite_enable

Purpose:

Debug isolation

Compositing validation

SDK readiness

Future register abstraction

No performance regression observed.

16×16 Sprite Support

Sprites now support:

8×8 (default)

16×16 (2×2 tile composite)

Layout:

[ base base+1 ]
[ base+2 base+3 ]

No new VRAM layout.
No tile duplication.
Fully deterministic decode.

Sprite Flipping

Implemented:

hflip

vflip

Flip applies across full sprite composite (8×8 or 16×16).

Global coordinate remapping used.
No additional memory usage.

Stability Verification

Confirmed:

Deterministic frame lifecycle

8-sprite-per-scanline enforcement

Overflow telemetry functional

RGB555 additive blending stable

No floating point contamination

No architectural regressions

Rendering core considered stable and expandable.

Template for Future Entries

When adding new entries, use this format:

## YYYY-MM-DD — Milestone Title

### Summary

Brief overview of change.

### Technical Changes

- Bullet list of implemented systems

### Constraints Check

- Determinism preserved?
- Hardware caps preserved?
- No float usage?
- No architectural rewrite?

### Notes

Optional engineering commentary.

Always insert new entries above older entries.
