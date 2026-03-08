# AUREX-16++ LLM SDK GUIDE (v0.3)

## Purpose
Deterministic contract for generating Aurex cartridges that run inside hard hardware-style limits.

Companion: `docs/human_game_creation_guide.md`.

## Hard Platform Caps (must be respected)
- Resolution: `426x240 @ 60 FPS`
- CPU budget: `200,000 ops/frame`
- WRAM: `512 KB`
- VRAM: `1 MB`
- Audio RAM: `512 KB`
- DMA caps: `4 commands/frame`, `64 KB VRAM/frame`, `16 KB audio/frame`
- Palette store: `4096 RGB555 entries` (legacy-first behavior preserved)
- Core math: integer-only (no float simulation paths)

## Prompt Contract (required sections, in order)
1. `GAME_ID`
2. `TITLE`
3. `GENRE_TAG`
4. `LOOP_SPEC`
5. `INPUT_MAP`
6. `ASSET_BUDGET`
7. `DMA_PLAN`
8. `PALETTE_PLAN`
9. `AUDIO_PLAN`
10. `FAILSAFE_RULES`
11. `OUTPUT_FILES`

Missing section => invalid prompt.

## Cartridge Identity Rules
- `GAME_ID` and runtime `cartridge_id` must match regex: `[a-z0-9_]+`
- Files live at:
  - `cartridges/<cartridge_id>/manifest.txt`
  - `cartridges/<cartridge_id>/...assets...`
- Manifest minimum:
  - `name=<display name>`
  - `game_id=<cartridge_id>`
  - `upload=<Region,dst_offset,file>`

## Palette + Tile/Sprite Rules
- RGB555 remains unchanged.
- Palette address space supports `0..4095` entries.
- Sprite field `palette` is a **base palette index** (`u16` semantics).
  - Lookup model: `final = palette[sprite.palette + color_index]`
- BG tilemap palette select uses bits `10..13` (16 banks).
- Tile payload format remains unchanged (4bpp packed, 32 bytes/tile).
- Sprite tile payload format remains unchanged.

## Launch + Resolver Contract
Runtime launch stages are typed and deterministic:
- `Pending -> Validating -> Ready`
- failure path: `Rejected`

Resolver gate requires manifest `game_id` match before attach/load side effects.

## Audio Contract (ASU-32 current)
- Host mix: `48 kHz` stereo.
- Deterministic ASU-32 runtime command interface:
  - `RuntimeAudioCommand::PlayTrack(track_id)`
  - `RuntimeAudioCommand::PlaySfx(AudioSfx)`
  - `RuntimeAudioCommand::StopTrack`
- Audio engine profile: 12 voices, static instrument table, wavetable bank (sine/square/triangle/saw/noise), integer-only fixed-point mix.
- No floating point and no dynamic allocation inside per-sample loop.

## Validation & Anti-Regression Checklist (required)
Before declaring output valid:
- Confirm no duplicated helper/function names were introduced.
- Confirm no debug-overflow-sensitive arithmetic paths were added without wrapping/saturating intent.
- Confirm prompt and manifest identity fields remain aligned (`GAME_ID`, folder, `game_id`).
- Confirm deterministic event model language uses runtime commands/events, not stale cue-only terminology.

## Quality Target Guidance
Aurex targets **creative constraints with premium polish**, not unrestricted simulation complexity.
- Prefer stronger art direction, tighter motif design, and deterministic audio identity.
- Avoid asking runtime to violate fixed budgets for “next-gen” effects.

See `docs/llm_prompt_template.md` for a structured generation template.

## Comparison target reference
- See `docs/aurex_vs_neo_geo.md` for target-level capability comparison criteria used by prompt and cartridge planning.


## Tooling commands (recommended preflight)
- `cargo run -- --audit-cartridges --json`
- `cargo run -- --analyze-cartridges --json`
- `cargo run -- --audio-diagnostics --json --frames 48000`
- `cargo run -- --replay-capture-smoke`
