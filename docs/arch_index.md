# AUREX-16++ ARCHITECTURE INDEX

Purpose:
Fast navigation to critical engine systems.
Updated only when structural changes occur.

---

## CORE FRAME LOOP

File:

- src/aurex/mod.rs

Important Functions:

- run_frame()
- tick()
- render_frame()

---

## PPU SYSTEM

File:

- src/aurex/ppu/ppu.rs

Core Struct:

- struct Ppu

Hot Functions:

- render_frame()
- render_scanline()
- write_sprite()

Mutation Points:

- BG scroll registers
- Window registers
- Layer enable flags
- Sprite flip logic

---

## VRAM SYSTEM

File:

- src/aurex/ppu/vram.rs

Core Struct:

- struct Vram

Locked Regions:

- BG pattern memory
- BG0 tilemap
- BG1 tilemap
- Sprite tiles
- Palettes

Do NOT re-architect layout.

---

## OAM SYSTEM

File:

- src/aurex/ppu/oam.rs

Core Struct:

- struct Sprite
- struct Oam

Likely to evolve:

- Sprite attributes
- Priority rules
- Blending modes

---

## DMA SYSTEM

File:

- src/aurex/dma/

Core:

- DmaController
- VramRegion

Hardware limits locked.

---

## AUDIO SYSTEM (ASU)

File:

- src/aurex/asu/

Core:

- Voice
- Envelope
- Sequencer

Integer-only audio logic required.

---

## HARDWARE LOCK POINTS

- VRAM_TOTAL_BYTES
- 200k ops cap
- 8 sprites per scanline
- 60 FPS deterministic loop
- RGB555 compositing
- Integer-only PPU core

## FILE RESPONSIBILITY LOCK

src/aurex/mod.rs

- Owns frame loop
- Owns system orchestration
- Does NOT mutate PPU internals

src/aurex/ppu/ppu.rs

- Owns all PPU state
- Owns render_frame
- Owns register writes
- Owns scanline pipeline

src/aurex/ppu/oam.rs

- Owns sprite storage
- No rendering logic

src/aurex/ppu/vram.rs

- Owns VRAM layout
- No rendering logic
- Canonical memory partition

boot/
prime_ignition.rs - Boot cinematic sequence - DMA glyph upload - Temporary visual validation logic

## Boot / Demo Modules

- `src/aurex/boot/prime_ignition.rs`
  - Owner: Boot demo (PrimeIgnition)
  - Responsibility:
    - Deterministic boot-time visuals (logo/glyph staging)
    - Asset staging into WRAM + DMA requests into VRAM (when enabled)
    - No architecture mutation; must obey Phase 6 VBlank gating rules
  - TEMP status: Yes (boot demo), but intentionally retained as a system-level integration test.

- `src/aurex/boot/render_probe.rs`
  - Owner: Render probe (diagnostic)
  - Responsibility:
    - Minimal known-good sprite/tile output for pipeline validation
    - Used to isolate issues in evaluation/render/tile indexing
  - TEMP status: Yes (diagnostic tool). May be disabled when PrimeIgnition is stable.

END OF INDEX


## LIBRARY DOMAIN

Files:
- `src/aurex/game/library.rs`
- `src/aurex/game/mod.rs`

Core model:
- `TitleProfile`
- `AudioCue::SelectTrack`

Responsibilities:
- Own title metadata (theme/icon/track)
- Emit title-selection cues to runtime audio
- Render placeholder library UI with per-title styling


## RUNTIME EVENT BUS

Files:
- `src/aurex/runtime/event.rs`
- `src/aurex/mod.rs`
- `src/main.rs`

Core objects:
- `RuntimeEvent`
- `Aurex::drain_events(...)`

Responsibilities:
- Decouple simulation output from host side effects
- Provide typed dispatch boundary for future runtime channels


## FLOW CONTROLLER TESTS

File:
- `src/aurex/runtime/flow.rs`

Coverage:
- Boot non-skip guard
- Boot timer expiry transition
- Await-start handshake transition


## SCENE TRANSITION EVENTS

Files:
- `src/aurex/runtime/event.rs`
- `src/aurex/mod.rs`
- `src/main.rs`

Core objects:
- `SceneId`
- `RuntimeEvent::SceneChanged`
- `Aurex::current_scene()`

Responsibilities:
- Emit explicit scene transition telemetry
- Keep scene lifecycle observable at host/runtime layer


## LLM SDK / Cartridge Authoring

Files:
- `docs/llm_sdk_guide.md`
- `docs/llm_prompt_template.md`
- `src/aurex/runtime/launch.rs`

Core objects:
- `LaunchDescriptor { title, cartridge_id }`
- `LaunchStage`
- `LaunchIntentController`

Responsibilities:
- enforce prompt-structured cartridge authoring expectations
- bridge library selection to cartridge identity (`cartridge_id`)
- provide deterministic launch lifecycle domain state

- Launch validation: `validate_launch_descriptor` + `TitleLaunchRejected` telemetry prior to pending stage.

- Launch readiness telemetry: `TitleLaunchReady(LaunchDescriptor)` emitted after deterministic validation stage completion.
