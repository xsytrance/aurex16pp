> **Addendum (2026-03-08):** PrimeAwakens is now the canonical boot visual module and ASU-32 boot audio includes anti-fuzz stabilization (non-retrigger policy for unchanged active notes + reduced boot percussion harshness). Refer to `docs/architecture.md` and `docs/ai_handoff_2026-03-08_runtime_av_stage3.md` for current operational details.

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
- Max colors on screen: **4096** (all palette entries addressable; 4bpp tiles/sprites use 16 colors per bank)
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
- Size: **512 KB**
- Strictly separated from VRAM policy

## 5. DMA / Transfer Constraints
- Max DMA commands per frame: **4**
- Max VRAM upload per frame: **64 KB**
- Max audio upload per frame: **16 KB**
- VRAM writes are VBlank-gated in current architecture policy
- Exceeding caps causes deterministic rejection

## 6. Rendering Pipeline Capabilities (Current)
- Deterministic scanline-oriented composition model
- BG0, BG1, and sprite layers; **16 sprites per scanline** (overflow if more; 128 sprites total)
- Sprite sizes: 8×8 and 16×16 (OAM `size_16`); priority and blend (Normal/Additive) with overflow telemetry
- Launch/library UX overlays rendered in framebuffer stage

## 7. Audio Runtime Capabilities (Current Host Path + Positioning)
- Host audio queue sample rate: **48 kHz**
- Host channel config (current main loop): **stereo (2 channels)**
- Runtime synthesis supports:
  - ASU-32 16-voice deterministic engine
  - static instrument table (ADSR + vibrato)
  - wavetable bank in 512 KB audio RAM (sine/square/triangle/saw/noise)
  - fixed-tick deterministic pattern sequencer
  - fixed-point stereo mixer + integer-only optional per-voice effects (delay/echo/bitcrush/distortion)
  - runtime audio commands: `PlayTrack`, `PlaySfx`, `StopTrack`
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
  - ASU-32 refinement pass (instrument authoring table tooling + validation)
  - deterministic motif sequencer improvements and track macro authoring checks
  - fixed-point stereo scene balancing diagnostics (no float)
- Tooling:
  - cartridge lint tool enforcing budget + identity + manifest schema
  - cartridge audit CLI mode: `cargo run -- --audit-cartridges` (or `--json`) for manifest/identity sweep
  - golden-frame regression snapshots for deterministic render validation
  - preflight entrypoint: `scripts/preflight.sh` (supports `AUREX_SKIP_AUDIT_LINK=1` for SDL-missing environments)


## 14. Additional Suggested Upgrades (Agent Proposals)

Implemented scaffolds:
- cartridge static analyzer v2 CLI path: `cargo run -- --analyze-cartridges --json`
- audio diagnostics CLI path: `cargo run -- --audio-diagnostics --json --frames <N>`
- replay capture smoke path: `cargo run -- --replay-capture-smoke`
- palette bank heatmap JSON path: `cargo run -- --palette-heatmap`


- Deterministic audio
  - add per-track instrument preset tables (wave mix + envelope profile IDs)
  - add fixed-point tempo swing table (no random timing drift)
  - add track-loudness safety normalization pass using integer peak clamps
- Graphics
  - add palette bank usage profiler (frame telemetry counters)
  - add boot/library authored palette scripts (keyframe LUT swaps per frame)
- Runtime
  - add deterministic capture/replay for launch + audio + input events
  - add cartridge static analyzer command that fails builds on budget/schema violations


## 15. Neo-Geo Comparison Reference
- See: `docs/aurex_vs_neo_geo.md`
- This comparison is used as target guidance for phase planning and authoring policy.

