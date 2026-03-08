# AUREX Cartridge Prompt Template (v0.2)

GAME_ID: neon_circuit  # must match [a-z0-9_]+
TITLE: NEON CIRCUIT
GENRE_TAG: racer

LOOP_SPEC:
- 60 FPS deterministic update
- integer-only state transitions
- no hidden frame-order dependencies

INPUT_MAP:
- LEFT/RIGHT: steer
- A/START: accelerate/confirm
- B/ESC: cancel/back

ASSET_BUDGET:
- WRAM <= 512 KB
- VRAM <= 1 MB
- VRAM upload/frame <= 64 KB
- DMA commands/frame <= 4
- Audio upload/frame <= 16 KB

DMA_PLAN:
- BG tiles in `BgTiles`
- tilemap in `Bg0Tilemap`
- palettes in `Palettes`
- chunked writes max 4096 bytes staged through WRAM

PALETTE_PLAN:
- RGB555 palette entries in range 0..4095
- sprite palette field is base index
- BG tilemap palette select uses bits 10..13 (16 banks)

AUDIO_PLAN:
- track_id mapping declared
- lane intent declared: bass / sub / lead / arp / percussion accents
- launch/cancel cues reserved for runtime UX

FAILSAFE_RULES:
- reject on cap violation
- reject writes to reserved region
- no deferred DMA
- no float in core simulation

OUTPUT_FILES:
- cartridges/neon_circuit/manifest.txt
- cartridges/neon_circuit/*.bin

Manifest snippet:
name=NEON CIRCUIT
game_id=neon_circuit
upload=BgTiles,0,bg_tiles.bin
