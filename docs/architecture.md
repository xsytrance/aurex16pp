# AUREX-16++ Architecture

Aurex-16++ is a deterministic 2D fantasy console designed to be:

- Hardware-inspired
- Strictly constrained
- LLM-friendly
- Deterministic
- 60 FPS fixed
- Integer-only rendering
- 2D-only (no 3D pipeline)

---

# Core System Overview

## Frame Model

- 60 FPS fixed timestep
- Deterministic execution
- No floating point in core rendering
- Frame-based hardware-style pipeline

---

# Memory Layout

## WRAM

- 512 KB

## VRAM

- 1 MB total
- Partitioned into:
  - BG tile data
  - Sprite tile data
  - Tilemaps
  - Palettes

## Palette Format

- RGB555
- Little-endian
- First 256 entries active
- Deterministic integer math only

---

# Graphics Subsystem (PPU)

## Background Layer (BG0)

- 64x64 tilemap
- 8x8 tiles
- 4bpp packed format (32 bytes per tile)
- Tilemap entry (u16):
  - 0..9 tile_index
  - 10..11 palette select (4 banks)
  - 12 hflip
  - 13 vflip
  - 14..15 priority (reserved)

### BG Rendering Rules

- Rendered first
- Scroll registers supported (bg0_scroll_x / bg0_scroll_y)
- Deterministic scanline rendering
- No floating point

---

## Sprite System

### Sprite Format

Each sprite contains:

- x (u16)
- y (u16)
- tile_index (u16)
- palette (u8)
- priority (u8)
- visible (bool)
- blend (BlendMode)

### Sprite Tile Format

- 8x8
- 4bpp packed
- 32 bytes per tile
- Color index 0 = transparent

---

## Sprite Pipeline

### Scanline Evaluation

- Per-scanline sprite selection
- Maximum 8 sprites per scanline
- Additional sprites trigger overflow

### Overflow Tracking

- `sprite_overflow_latched` (per frame)
- `sprite_overflow_scanlines` (counter)

### Priority Sorting

- Sprites sorted by priority (low first)
- Composited after BG

---

## Blending System

### Supported Blend Modes

- Normal (overwrite)
- Additive

### Additive Blending

- RGB555 channel-wise add
- Clamp per channel (0–31)
- No overflow
- Deterministic
- Integer-only math

---

# Current Rendering Order

1. BG0 rendered
2. Sprites composited in priority order
3. Blend mode applied per pixel

---

# Current Limitations (Intentional)

- Only one background layer (BG0)
- No sprite flipping yet
- No sprite scaling
- No affine transforms
- No window layers
- No multi-layer priority system
- No alpha blending (only additive)
- No hardware register abstraction yet

---

# Development Status

PPU Phase 1: COMPLETE

The core 2D rendering pipeline is operational, deterministic, and stable.

Next milestones will expand capability without breaking determinism.

---

# Design Philosophy

Aurex-16++ aims to:

- Be superior to SNES/Genesis in flexibility
- Remain below PS1 complexity
- Encourage creative constraint
- Support LLM-generated cartridges under hardware-style limits
- Prioritize readability, determinism, and performance
