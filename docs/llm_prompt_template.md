# AUREX Cartridge Prompt Template (v0.3)

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
- AUDIO_RAM <= 512 KB
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
- sprite palette field is base index (`u16` semantic)
- BG tilemap palette select uses bits 10..13 (16 banks)

AUDIO_PLAN:
- `PlayTrack(track_id)` mapping declared
- optional UI SFX usage declared (`PlaySfx(Launch|Cancel|Confirm)`)
- deterministic lane/voice intent declared (12-voice ASU-32 budget)
- no float/no heap in per-sample path

FAILSAFE_RULES:
- reject on cap violation
- reject writes to reserved region
- no deferred DMA
- no float in core simulation
- reject if any identity mismatch (`GAME_ID`, folder, `manifest game_id`)

OUTPUT_FILES:
- cartridges/neon_circuit/manifest.txt
- cartridges/neon_circuit/*.bin

Manifest snippet:
name=NEON CIRCUIT
game_id=neon_circuit
upload=BgTiles,0,bg_tiles.bin

Final self-check before response:
- confirm no duplicate helper/function names in generated code
- confirm arithmetic overflow behavior is explicit where needed (wrapping/saturating)
- confirm runtime terminology uses `RuntimeAudioCommand`/launch events consistently
