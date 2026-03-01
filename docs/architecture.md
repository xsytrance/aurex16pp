# Aurex-16++ Architecture Progress

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