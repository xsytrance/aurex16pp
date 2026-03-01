# Aurex-16++ Architecture Progress

## Phase 4.5 — Framebuffer Debug Test

- Added TEMP TEST debug draw module
- Framebuffer is populated each frame with a deterministic test pattern
- Used to validate pixel pipeline before SDL integration
- Marked clearly as non-production logic

## Phase 4 — Framebuffer Skeleton
- Added PPU-A16 framebuffer at 426×240
- Internal pixel format is u16 RGB555 (0RRRRRGGGGGBBBBB)
- Framebuffer clears to black each frame (no rendering yet)

## Phase 3.6 — Real DMA (WRAM → VRAM)

- DMA now copies real bytes from WRAM into specific VRAM partitions
- Transfers validated at request time (Option A discipline)
- No clipping, no partial writes
- Max 4 commands per frame
- Max 64 KB VRAM upload per frame
- All large memory blocks allocated via Vec → Box<[u8]> to avoid stack overflow
- Audio DMA temporarily disabled (reserved for ASU-816 phase)

## Phase 3.5
- DMA now queues accepted transfers
- DMA apply stage executes at frame end
- VRAM partitions implemented as separate heap allocations
- Placeholder DMA writes currently mark BG tile memory
- No WRAM source copying yet

### Technical Notes
- All large memory blocks (WRAM, VRAM partitions) allocated via Vec -> Box<[u8]>
  to prevent Windows stack overflow during initialization.
- Core loop contains no temporary smoke tests.
- Frame timing uses anchored frame_start approach (no drift accumulation).

## Phase 3
- PPU-A16 VRAM skeleton implemented as separate fixed partitions (Option B)
- Total VRAM = 1 MiB split into:
  - 384 KB BG tiles
  - 128 KB tilemaps
  - 384 KB sprite tiles
  - 64 KB Mode7 texture
  - 16 KB palettes
  - 64 KB reserved/system
- No rendering yet (memory only)

## Phase 1
- Deterministic 60 FPS clock
- 200,000 ops per frame CPU cap
- 512 KB WRAM (heap allocated)
- VM-32 stub

## Phase 2
- DMA Controller
  - Max 4 commands per frame
  - Max 64 KB VRAM upload
  - Max 16 KB audio upload
  - Reject tracking

## Phase 2.5
- PDU now ingests DMA telemetry
- CPU and DMA budgets unified under frame diagnostics