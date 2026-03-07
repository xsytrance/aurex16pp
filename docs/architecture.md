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
  - 10..11 palette select (4 banks)
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
- palette (u8)
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
- Selection changes emit `AudioCue::SelectTrack(track_id)`.
- Audio runtime resolves per-title songs by `track_id` (6 title-specific patterns).

This keeps UI theming and soundtrack policy data-driven instead of hardcoded across unrelated modules.


## Boot Gate Architecture (2026-03-08)

Boot flow is now strictly non-interruptible:
- Phase 1: Timed boot cinematic (`FlowPhase::Boot`).
- Phase 2: Gate screen on same boot scene (`FlowPhase::AwaitStart`).
- Phase 3: Library runtime (`FlowPhase::Game`).

Input is ignored for scene transitions during timed boot and only accepted in `AwaitStart` for an explicit Start press.


## Runtime Event Bus (2026-03-08)

A typed runtime event bus is now the handoff boundary between simulation and host runtime orchestration.

- `RuntimeEvent::Audio(AudioCue)` is emitted by core system logic.
- `RuntimeEvent::TitleLaunchRequested(&'static str)` captures library launch intent as typed telemetry.
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
- Audio: `AudioCue::LaunchRequest` triggers dedicated launch stinger SFX.
- Architecture: `collect_runtime_diagnostics(&[RuntimeEvent]) -> RuntimeDiagnostics` centralizes non-audio event interpretation for host orchestration.

This keeps scene simulation deterministic while improving UX clarity and reducing host-loop branching noise.
