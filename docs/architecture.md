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
  - 10..13 palette select (16 banks)
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
- palette (u16 base index semantic)
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


---

## Library Runtime Domain (2026-03-08 update)

A dedicated title-profile domain now drives the library scene:

- `TitleProfile` = title text + audio track id + color theme + icon kind.
- Library selection is now a stateful domain event source.
- Selection changes emit runtime audio commands (`PlayTrack(track_id)`).
- Audio runtime resolves per-title songs by deterministic track id mapping.

This keeps UI theming and soundtrack policy data-driven instead of hardcoded across unrelated modules.


## Boot Gate Architecture (2026-03-08)

Boot flow is now strictly non-interruptible:
- Phase 1: Timed boot cinematic (`FlowPhase::Boot`).
- Phase 2: Gate screen on same boot scene (`FlowPhase::AwaitStart`).
- Phase 3: Library runtime (`FlowPhase::Game`).

Input is ignored for scene transitions during timed boot and only accepted in `AwaitStart` for an explicit Start press.


## Runtime Event Bus (2026-03-08)

A typed runtime event bus is now the handoff boundary between simulation and host runtime orchestration.

- `RuntimeEvent::Audio(RuntimeAudioCommand)` is emitted by core system logic.
- `RuntimeEvent::TitleLaunchRequested(LaunchDescriptor)` captures library launch intent as typed telemetry.
- Main loop drains events after `run_frame` and dispatches side effects (audio synth triggers).
- This removes direct audio-cue polling from the core API and prepares for additional event classes (UI, telemetry, cartridge).


## Event Queue Component (2026-03-08)

Runtime events now flow through a dedicated queue object (`RuntimeEventQueue`) instead of raw vectors in core state.

- Queue owns event buffering and drain semantics.
- `Aurex` emits intents to queue.
- Host loop drains queue and executes side effects.

This formalizes event transport as a reusable core component for future channels.


## Runtime Dispatch Primitive (2026-03-08)

A runtime-level dispatch helper now applies side effects from drained events:

- `dispatch_runtime_events(AudioEngine, &[RuntimeEvent])`

This centralizes host-side dispatch policy and keeps the main loop focused on lifecycle orchestration.


## Scene Transition Telemetry (2026-03-08)

Runtime now emits explicit scene transition events via the event bus:

- `RuntimeEvent::SceneChanged(SceneId)`
- Current scenes:
  - `SceneId::Boot`
  - `SceneId::Library`

Core emits transition intent; host loop may log/route diagnostics or future UI overlays without touching scene internals.

## Handoff-Ready Runtime Boundaries

Current boundaries are now explicit and suitable for team/agent handoff:

1. **Flow Policy** (`FlowController`)
   - Owns transition timing and gates.
2. **Scene Simulation** (`Aurex` core)
   - Owns deterministic frame update/render.
3. **Event Transport** (`RuntimeEventQueue`)
   - Owns event buffering and draining semantics.
4. **Host Dispatch** (`dispatch_runtime_events`)
   - Owns side effects from emitted runtime intents.


## Library AV Feedback Pass (2026-03-08 01:08:00Z)

Library runtime now includes explicit launch-feedback channels across visual, audio, and host diagnostics layers:

- Visual: deterministic footer audio meter + launch status pulse tint.
- Audio: launch intent triggers deterministic `PlaySfx(Launch)` runtime command.
- Architecture: `collect_runtime_diagnostics(&[RuntimeEvent]) -> RuntimeDiagnostics` centralizes non-audio event interpretation for host orchestration.

This keeps scene simulation deterministic while improving UX clarity and reducing host-loop branching noise.


## Launch Intent Lifecycle (2026-03-08 01:37:00Z)

Library launch flow now has bidirectional intent states:
- Request: `RuntimeEvent::TitleLaunchRequested(LaunchDescriptor)`
- Clear: `RuntimeEvent::TitleLaunchCanceled`

Audio runtime commands now distinguish user actions:
- `PlaySfx(Launch)`
- `PlaySfx(Cancel)`

Host loop consumes these via `RuntimeDiagnostics` to keep orchestration centralized.


## Launch Domain Controller (2026-03-08 02:02:00Z)

Launch orchestration now includes a dedicated runtime domain component:
- `LaunchIntentController`
- `LaunchStage::{Idle, Pending, Validating, Ready, Rejected}`

Core scene update emits `RuntimeEvent::LaunchStageChanged(LaunchStage)` on transitions, and host loop consumes this through `RuntimeDiagnostics`.


## LLM Cartridge SDK Contract (2026-03-08 02:28:00Z)

Aurex is explicitly designed for structured LLM cartridge generation.

Canonical authoring docs:
- `docs/llm_sdk_guide.md`
- `docs/llm_prompt_template.md`

Runtime launch descriptors now include both display and build identity:
- `title`
- `cartridge_id`

This keeps host/runtime launch orchestration aligned with deterministic, prompt-structured cartridge output.


## Launch Descriptor Validation (2026-03-08 02:56:00Z)

Launch requests now pass deterministic descriptor validation before stage transition:
- `validate_launch_descriptor(LaunchDescriptor)`
- rejection telemetry: `RuntimeEvent::TitleLaunchRejected(LaunchValidationError)`

This is a pre-validation guardrail for future cartridge loading stages.


## Launch Ready Stage (2026-03-08 03:22:00Z)

Runtime now emits explicit readiness telemetry:
- `RuntimeEvent::TitleLaunchReady(LaunchDescriptor)`

This event is intended as the future handoff trigger to cartridge boot/runtime attach policy.


## Launch Resolver Gate (2026-03-08 03:44:00Z)

After validation reaches `Ready`, runtime performs cartridge resolution by `cartridge_id`:
- success emits `RuntimeEvent::TitleLaunchResolved(LaunchDescriptor)`
- failure transitions to rejected state and emits launch rejection telemetry

This prevents host boot handoff from firing on unresolved cartridge IDs.


Resolver failures now distinguish missing manifests from invalid manifests; invalid manifests map to `CartridgeManifestInvalid` rejection telemetry.


## Human Authoring Guide (2026-03-08 04:22:00Z)

A human-facing cartridge authoring guide now complements the LLM SDK docs:
- `docs/human_game_creation_guide.md`

Purpose:
- help designers/producers request LLM-generated games using the same strict contract and hardware limits
- keep human workflow aligned with deterministic runtime constraints


---

## 2026-03-08 Canon Refresh (Palette + AV Direction)

### Graphics updates now in force
- Palette storage expanded to **4096 RGB555 entries**.
- Legacy compatibility preserved: the first 256 palette entries initialize exactly as before.
- Sprite palette reference is now a base index (`u16` domain behavior).
- BG tilemap palette select now uses bits **10..13** (16 banks).
- Deterministic scanline renderer model remains unchanged.
- Tile and sprite payload formats remain unchanged.

### Audio positioning vs Neo-Geo
Current Aurex audio is deterministic integer ASU-32 synthesis (48 kHz stereo, 12 voices, 512 KB audio RAM).

Near-term upgrade path (still within Aurex vision):
1. Deterministic 12-voice engine with fixed-point stereo pan/mix.
2. Pattern-instrument envelope table (attack/decay/sustain/release presets as integer lookups).
3. Track macro sequencing (per-title motif blocks) with zero runtime allocation.
4. Integer-only per-voice FX (delay/echo/bitcrush/distortion) under deterministic budgets.

### Vision discipline
Goal is “better than Neo-Geo in curated presentation consistency and deterministic tooling,” not by removing constraints.


## Audio Upgrade Increment (2026-03-08 Phase 2)

Implemented in runtime audio path:
- deterministic envelope shaping for major lanes
- explicit low-end support via sub lane
- zero-allocation per-sample synthesis path retained

This improves perceived production depth without changing core deterministic constraints or introducing floating-point math.


## Neo-Geo Comparison Planning

- Reference: `docs/aurex_vs_neo_geo.md`
- Planning rule: improvements should maintain or exceed Neo-Geo-class outcomes in each category (graphics richness, audio identity, runtime robustness, developer tooling) while preserving Aurex deterministic constraints.


## Documentation Reliability Notes (2026-03-08)

To reduce repeat regressions between docs and code:
- Prefer impl-scoped helpers for module-local math utilities to avoid duplicate symbol drift during merge stacking.
- Explicitly mark overflow-sensitive integer math paths as wrapping/saturating where required by deterministic intent.
- Keep event terminology synchronized with runtime code (`RuntimeAudioCommand`, typed launch events), and avoid stale cue-only wording in architecture summaries.
- Treat all rollout/testing statements as environment-specific; only claim full runtime execution when linker/runtime prerequisites are present.
