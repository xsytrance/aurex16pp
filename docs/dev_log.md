AUREX-16++ DEVELOPMENT LOG

Reverse Chronological Engineering Record

Newest entries are always added at the top.
This file tracks engineering evolution, not canonical hardware state.
Refer to ai_handoff.md for current hardware truth.

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
