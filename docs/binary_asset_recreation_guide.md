# Binary Asset Recreation Guide

This project now stores **no binary files in Git**. Use this guide to recreate the removed binary artifacts when needed.

## 1) Cartridge assets (`chrome_duo_boot`)

### `cartridges/chrome_duo_boot/palette.bin`
- **Purpose:** 16-color RGB555 palette upload used by `chrome_duo_boot` boot visuals.
- **Target location:** `cartridges/chrome_duo_boot/palette.bin`
- **Tech specs:**
  - Binary, little-endian `u16` words.
  - Exactly **32 bytes** (16 entries × 2 bytes).
  - RGB555 encoding (`0b0RRRRRGGGGGBBBBB`).
  - Intended upload region: `Palettes`, destination offset `0`.

### `cartridges/chrome_duo_boot/bg_tiles.bin`
- **Purpose:** Background tile payload for boot-themed pattern rendering.
- **Target location:** `cartridges/chrome_duo_boot/bg_tiles.bin`
- **Tech specs:**
  - Binary blob of tile bytes.
  - Exactly **4096 bytes**.
  - Intended upload region: `BgTiles`, destination offset `0`.

### `cartridges/chrome_duo_boot/bg0_map.bin`
- **Purpose:** BG0 tilemap data for deterministic cartridge visual layout.
- **Target location:** `cartridges/chrome_duo_boot/bg0_map.bin`
- **Tech specs:**
  - Binary, little-endian tilemap entries (`u16` per cell).
  - Exactly **8192 bytes** (64×64×2).
  - Intended upload region: `Bg0Tilemap`, destination offset `0`.

## 2) Snapshot artifacts

### `full_pub_snapshot.txt`
- **Purpose:** Workspace-wide public API snapshot artifact (generated, not source of truth).
- **Target location:** `full_pub_snapshot.txt` (repo root).
- **Tech specs:**
  - Plain text artifact.
  - UTF-8 with LF line endings recommended.
  - One symbol/location entry per line.

### `symbol_snapshot.txt`
- **Purpose:** Symbol inventory snapshot artifact for comparison/auditing.
- **Target location:** `symbol_snapshot.txt` (repo root).
- **Tech specs:**
  - Plain text artifact.
  - UTF-8 with LF line endings recommended.
  - One symbol/location entry per line.

## Manifest expectations

The `chrome_duo_boot` manifest references the three cartridge binaries above and expects them at runtime:
- `upload=Palettes,0,palette.bin`
- `upload=BgTiles,0,bg_tiles.bin`
- `upload=Bg0Tilemap,0,bg0_map.bin`
