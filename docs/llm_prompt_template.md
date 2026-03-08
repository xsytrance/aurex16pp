# AUREX Cartridge Prompt Template

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
- VRAM upload/frame <= 64 KB
- DMA commands/frame <= 4
- Audio upload/frame <= 16 KB

DMA_PLAN:
- BG tiles in `BgTiles`
- tilemap in `Bg0Tilemap`
- palettes in `Palettes`
- chunked writes max 4096 bytes staged through WRAM

AUDIO_PLAN:
- track_id mapping declared
- launch/cancel cues reserved for runtime UX

FAILSAFE_RULES:
- reject on cap violation
- reject writes to reserved region
- no deferred DMA

OUTPUT_FILES:
- cartridges/neon_circuit/manifest.txt
- cartridges/neon_circuit/*.bin
