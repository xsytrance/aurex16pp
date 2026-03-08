# AUREX-16++ Technical Specification Report (2026-03-08)

This report consolidates current canonical hardware/runtime capabilities into one reference.

## 1. Core Platform Identity
- Platform: Aurex-16++
- Design model: deterministic 2D fantasy console
- Core simulation target: fixed 60 FPS
- Math policy (core): integer-only in VM/PPU paths

## 2. Display / Video Output
- Render resolution: **426 x 240** (16:9)
- Frame rate target: **60 FPS fixed timestep**
- Pixel format: **RGB555** (15-bit)
- Palette capacity: **4096 RGB555 entries** (8192 bytes)
- Legacy compatibility: first 256 entries retain prior initialization behavior

## 3. Processor / Compute Budget
- CPU model: **VM-32**
- Register width (model): **32-bit internal registers**
- Hard frame budget: **200,000 ops per frame**
- Equivalent throughput at 60 FPS: **12,000,000 ops/sec** (derived)
- CPU cap behavior: overages are rejected/tracked (no soft forgiveness)

## 4. Memory System
### WRAM
- Size: **512 KB** (locked)

### VRAM
- Size: **1 MB total** (0x100000 bytes)
- Regioned hardware layout with reserved protected region
- Used for BG/sprite/palette and cartridge VRAM usage domains

### Audio RAM
- Size: **256 KB**
- Strictly separated from VRAM policy

## 5. DMA / Transfer Constraints
- Max DMA commands per frame: **4**
- Max VRAM upload per frame: **64 KB**
- Max audio upload per frame: **16 KB**
- VRAM writes are VBlank-gated in current architecture policy
- Exceeding caps causes deterministic rejection

## 6. Rendering Pipeline Capabilities (Current)
- Deterministic scanline-oriented composition model
- BG0 and sprite-centric runtime flow in active path
- Sprite system supports priority/blend flags with overflow telemetry
- Launch/library UX overlays rendered in framebuffer stage

## 7. Audio Runtime Capabilities (Current Host Path + Positioning)
- Host audio queue sample rate: **44.1 kHz**
- Host channel config (current main loop): **mono (1 channel)**
- Runtime synthesis supports:
  - boot music mode
  - per-title library track selection
  - launch/cancel cue stingers
- Neo-Geo positioning:
  - current Aurex audio is deterministic and stylistically solid
  - does **not yet** match Neo-Geo multi-voice production depth
  - targeted improvement path is richer channel architecture under fixed deterministic budgets

## 8. Input / Control Model (Current)
- Keyboard + game controller polling
- Library lifecycle actions:
  - directional navigation
  - accept/launch intent
  - cancel/clear intent

## 9. Launch / Cartridge Readiness Pipeline
Current launch domain stages:
- `Idle`
- `Pending(LaunchDescriptor)`
- `Validating(LaunchDescriptor)`
- `Ready(LaunchDescriptor)`
- `Rejected(LaunchValidationError)`

Launch descriptor identity:
- `title`
- `cartridge_id`

Resolver gate:
- resolves `cartridge_id` against `cartridges/<cartridge_id>/manifest.txt`
- requires manifest `game_id=` field matching requested `cartridge_id`

## 10. Cartridge Authoring Contract (LLM + Human)
Authoring references:
- `docs/llm_sdk_guide.md`
- `docs/llm_prompt_template.md`
- `docs/human_game_creation_guide.md`

Required identity rule:
- `GAME_ID` / `cartridge_id` / manifest `game_id` must align
- ID format: `[a-z0-9_]+`

## 11. Determinism / Safety Guarantees
- No hidden dynamic budget scaling
- No deferred DMA policy
- Deterministic event-oriented runtime telemetry
- Typed rejection semantics for invalid launch/manifests

## 12. Current Practical Capability Summary
Aurex is currently capable of:
- deterministic boot -> library flow
- title-specific visual theming + music selection
- staged launch intent validation/resolution telemetry
- strict cartridge identity checks before future boot attach

Next capability unlock for “generated games fully runnable”:
- attach/load execution after `TitleLaunchResolved`
- manifest/schema expansion for richer cartridge metadata


## 13. Suggested Upgrade Stack (Constrained “Beyond Neo-Geo” Path)
- Graphics:
  - palette tooling (bank heatmap/debug view)
  - per-title palette animation utilities with deterministic frame scripts
  - richer boot/library scene shaders expressed as integer LUT effects
- Audio:
  - explicit 4-lane instrument engine with deterministic ADSR tables
  - motif sequencer + per-title macro patterns
  - optional host stereo widener while preserving mono simulation core
- Tooling:
  - cartridge lint tool enforcing budget + identity + manifest schema
  - golden-frame regression snapshots for deterministic render validation
