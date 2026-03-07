## 2026-03-07 20:09:06Z — Snake Retirement + System Sandbox Bring-up

### Summary
Per creative direction, snake-specific gameplay progress was retired and replaced with a system-focused sandbox scene to continue core platform development.

### Functional Changes
- Replaced snake loop with a lightweight system sandbox:
  - dark board/panel backdrop
  - movable cursor
  - animated status nodes
- Kept deterministic input-driven cursor movement with bounded grid constraints.
- Kept audio cue path alive (`AudioCue::Eat`) on cursor movement to preserve input/audio integration checks.

### Architecture Rationale
This shifts focus from game mechanics to platform validation loops (input → state update → render → audio cue), which is more aligned with system maturation goals.

### Progress Report
System architecture currently modularized into runtime subsystems:
- Flow control (`runtime::flow`)
- Input polling (`runtime::input`)
- Audio engine (`runtime::audio`)
- Render presenter (`runtime::render`)
- Frame pacing (`runtime::frame_pacer`)

Recommended next system steps:
1. Scene trait + SceneManager for explicit scene lifecycle boundaries.
2. Unified runtime event channel for typed cues (audio/UI/telemetry).
3. Hot-reloadable config blocks for visuals/audio parameters.

## 2026-03-07 19:26:20Z — Dark Theme Pass + Frame Pacer Architecture

### Summary
Applied requested visibility update by darkening the snake playfield theme and continued architecture separation by extracting frame pacing logic from `main`.

### Graphics Changes
- Reworked game palette to a darker near-black blue board for stronger contrast and less visual washout.
- Preserved bright snake/head/food accents against darker background for readability.

### Architecture Changes
- Added `runtime::frame_pacer` module with `FramePacer` helper.
- Replaced manual sleep/elapsed timing math in `main.rs` with `FramePacer::wait_next_frame()`.
- Keeps main loop focused on orchestrating runtime subsystems (flow/input/audio/render/pacing).

### Progress Report
Current subsystem status:
- ✅ Flow state machine extracted (`runtime::flow`).
- ✅ Audio synthesis extracted (`runtime::audio`).
- ✅ Input polling extracted (`runtime::input`).
- ✅ Render presentation extracted (`runtime::render`).
- ✅ Frame pacing extracted (`runtime::frame_pacer`).
- ✅ Snake AV polish stack active (glow body, corner glints, subtle drift, richer audio layers).

Next recommended architecture steps:
1. Add a `Scene` trait + `SceneManager` to formalize boot/game transitions.
2. Introduce a typed event bus for audio cues/telemetry.
3. Move hardcoded gameplay constants into config structs to support rapid theme/game swaps.

## 2026-03-07 18:58:48Z — Render Pipeline Extraction + Audio/Visual Motion Tune

### Summary
Continued architecture cleanup while improving presentation quality by extracting host presentation code and adding subtle motion/tonal character improvements.

### Architecture Changes
- Added `runtime::render` module with `present_frame(...)` for framebuffer conversion + SDL present path.
- Removed framebuffer conversion/present boilerplate from `main.rs` and switched to runtime render API.
- Keeps `main` focused on orchestration (flow, audio queue, input polling) rather than pixel conversion internals.

### Graphics/Sound Improvements
- Added subtle BG horizontal drift in gameplay for living-scene feel while preserving readability.
- Added light vibrato in game lead synthesis path for arcade character.
- Preserved border glints + snake glow animation stack.

### Notes
This pass further applies the anti-regression strategy: isolate subsystems to reduce conflicting edits in `main.rs`.

## 2026-03-07 18:53:29Z — Runtime Input Module Extraction + Motion FX Pass

### Summary
Continued architecture hardening and AV polish by extracting input polling logic from the main loop and adding subtle border/glint motion accents.

### Architecture Changes
- Added `runtime::input` module with a typed `poll_input(...)` interface returning `quit/start/gameplay` state.
- Removed duplicated keyboard/controller polling logic from `main.rs` and switched to runtime-owned input orchestration.
- Kept defensive SDL strategy (state-polling only; no panic-prone event/scancode iteration paths).

### Graphics/Sound Improvements
- Added animated corner glint sprites along arena border for clearer visual activity.
- Preserved extracted audio-engine path and deterministic synthesis behavior.

### Lessons Applied
This pass explicitly reduces large single-loop risk by moving input policy into a subsystem module, mirroring the prior flow/audio extraction pattern.

## 2026-03-07 18:04:07Z — Audio Architecture Extraction + Snake Visual FX Pass

### Summary
Continued architecture and AV polish by extracting synthesis logic from `main.rs` into a dedicated runtime audio module and adding richer in-game visual/audio motion cues.

### Architecture Changes
- Added `runtime::audio` module with `AudioEngine` + `AudioMode`.
- Moved music/SFX synthesis and cue handling out of `main.rs` to reduce loop complexity and improve subsystem boundaries.
- Updated runtime module exports so `main` consumes audio through a clean runtime API.

### Graphics/Sound Improvements
- Added snake body glow animation tile and alternating body-segment visual cadence.
- Added arpeggiated game layer to synth for a denser arcade mix while preserving deterministic generation.

### Notes
This follows a “separate policy from plumbing” approach to prevent repeat regressions from large monolithic loop logic.

## 2026-03-07 17:56:26Z — Keyboard Scancode Panic Mitigation

### Summary
Addressed another Windows panic source (`scancode.rs` invalid enum value) by removing dynamic scancode iteration from the input path.

### Technical Changes
- Replaced `keyboard_state().pressed_scancodes()` usage with an explicit, curated set of safe `is_scancode_pressed(...)` checks for start keys.
- Kept Escape handling and gameplay directional polling unchanged.
- Added inline note documenting the defensive strategy for rust-sdl2 enum conversion edge cases.

### Impact
- Eliminates the reported keyboard-scancode conversion panic path while preserving expected start/menu and gameplay controls.

## 2026-03-07 17:41:58Z — SDL Event Robustness Fix + Warning Cleanup

### Summary
Addressed a Windows runtime panic path from SDL event decoding (`invalid enum value 0x607`) and reduced warning noise for known intentional placeholders.

### Technical Changes
- Reworked main-loop input handling to avoid `poll_iter()` enum decoding on every event.
- Switched to `pump_events()` + keyboard/controller state polling for robust cross-device input handling.
- Added explicit comment documenting why raw event enum decoding was avoided.
- Renamed internal temporary PPU locals to underscore-prefixed forms to silence noisy unused-variable warnings.

### Impact
- Prevents crash path observed with certain controllers/drivers emitting event values not decoded by the current rust-sdl2 release.
- Keeps existing gameplay/input behavior while improving runtime resilience.

## 2026-03-07 17:29:13Z — Visual/Sound Polish Pass (Snake Scene)

### Summary
Continued forward with player-facing polish by improving in-game visual quality and audio texture while preserving deterministic behavior.

### Visual Changes
- Introduced a styled BG tilemap board (dark/light checker playfield + cyan border frame).
- Enabled BG rendering in game mode with fixed zero-scroll board presentation.
- Added animated/pulsing food visuals via alternating sprite tiles.
- Tightened playfield bounds to match visible framed arena and improved HUD pip spacing.

### Audio Changes
- Added deterministic hat/noise texture into music pattern synthesis for fuller mix.
- Refined eat SFX into a short descending chirp for clearer arcade feedback.

### Technical Note
This pass prioritized immediate presentation quality without architecture churn.

## 2026-03-07 17:12:20Z — Runtime Flow Controller Architecture Pass

### Summary
Advanced architecture by extracting boot/confirm/game transition policy from `main.rs` into a dedicated runtime controller module.

### Technical Changes
- Added `aurex::runtime` module with `FlowController` and `FlowPhase`.
- Moved phase-transition responsibilities (`Boot -> Confirming -> Game`) into the controller.
- Converted `main.rs` to consume controller APIs (`register_start_request`, `tick`, `phase`, `game_active`).
- Synced boot overlay confirmation state from central flow policy each frame.

### Why this helps
- Improves separation of concerns (input/audio loop no longer owns transition policy details).
- Provides a reusable control point for future scene/state expansion.
- Reduces duplicated transition condition logic across input pathways.

## 2026-03-07 17:00:47Z — Boot Prompt Centering + Transition Handoff + Snake Demo Pass

### Summary
Addressed final boot/demo UX polish requests: centered/fixed continue prompt text, explicit audio/state handoff from boot into game, and replaced the prior platformer demo with a compact snake-style clone.

### Technical Changes
- Centered bottom prompt using measured text width.
- Fixed missing glyph support in the boot pixel font (`I`, plus additional prompt/loading characters).
- Added a boot confirmation/loading handoff state so input triggers a short confirm phase before game start.
- Added explicit boot confirmation visual (`LOADING...`) while the handoff is active.
- Split audio behavior into flow-aware modes: boot music, confirmation sound, and separate game music.
- Added game SFX cue path for snake events (eat/fail) and wired it through core->main audio trigger handling.
- Replaced previous tech demo with a simple snake clone (grid movement, growth, food spawn, death/reset loop).

### Notes
Scope intentionally kept lightweight for iteration speed while fixing requested UX/audio transitions.

## 2026-03-07 16:26:34Z — Boot Visual/Flow Refinement

### Summary
Improved boot presentation and transition flow: larger/crisper logo treatment, explicit on-screen continue prompt, and retained boot music pipeline before entering the tech demo.

### Technical Changes
- Added a crisp 5x7 pixel-font overlay renderer for boot text, drawn directly onto the framebuffer for sharp edges.
- Increased perceived logo size by rendering `AUREX-16++` at scale 4 with drop-shadow.
- Added blinking `PRESS ANY BUTTON TO CONTINUE` prompt at scale 2.
- Kept the existing "press any key/button" transition path into `start_game()`.
- Preserved continuous audio queue feeding for boot music playback.

### Constraints Check
- Determinism preserved: yes.
- Hardware caps preserved: yes.
- No float usage introduced: yes.
- No architecture rewrite: yes.

## 2026-03-07 16:05:00Z — Boot-to-Game Start Gate and Input Flow Fix

### Summary
Resolved a boot flow issue where the program could appear stuck on the logo/boot scene by introducing an explicit run mode gate and a clear transition into gameplay.

### Technical Changes
- Added an explicit `RunMode` state machine (`Boot` and `Game`) in `Aurex`.
- Added `start_game()` on `Aurex` to perform an explicit mode transition.
- Routed `Aurex::tick(...)` through boot logic in `Boot` mode and gameplay logic in `Game` mode.
- Updated event/input handling to trigger start on keyboard and controller button events.
- Added controller state polling fallback so analog activity can also trigger the transition.
- Preserved per-frame audio queue feeding to avoid regressions in audio generation cadence.

### Constraints Check
- Determinism preserved: yes (mode transition is explicit and monotonic per start event).
- Hardware caps preserved: yes.
- No float usage introduced in core paths: yes.
- No architectural rewrite: yes (incremental state-gate fix only).

### Notes
This addresses the observed user-facing symptom of appearing to remain on boot/logo indefinitely when start input was not being accepted robustly across input paths.

AUREX-16++ DEVELOPMENT LOG

Reverse Chronological Engineering Record

Newest entries are always added at the top.
This file tracks engineering evolution, not canonical hardware state.
Refer to ai_handoff_canon.md for current hardware truth.

## [PPU Phase 6 / Boot Demo Recovery] Sprite pipeline bugfix + VBlank foundation

### What went wrong (root cause)

We hit a failure mode where 16x16 glyph/sprite rendering appeared “cut off” or garbage:

- Sprite scanline evaluation was still locked to 8x8 height (`sprite_bottom = sprite_top + 8`) even when sprites were actually rendered as 16x16.
- Sprite renderer temporarily forced `sprite_size = 16` unconditionally, creating a mismatch between evaluation and render rules.
- BG priority buffer (`bg_priority_line`) was accidentally re-declared inside the per-pixel loop, shadowing the intended scanline buffer and breaking its lifetime/scope.

Net effect:

- Sprites were only considered “present” on the first 8 scanlines, so the bottom half never rendered.
- Some experimental paths caused tile math to read the wrong rows/tiles, producing broken glyph shapes.

### Fix summary

- Sprite evaluation now uses the sprite’s configured size:
  - `sprite_size = if sprite.size_16 { 16 } else { 8 }`
  - `sprite_bottom = sprite_top + sprite_size`
- Sprite renderer now uses the same size logic (no hard-coded 16).
- Removed the accidental re-declaration of `bg_priority_line` inside the pixel loop so it persists for the entire scanline as intended.

### Phase 6 note (VBlank foundation)

PPU now simulates VBlank with a simple deterministic latch:

- `vblank = false` at frame start
- `vblank = true` after all scanlines render
  No mid-scanline timing yet (pre-VBlank simulation only), but this enables deterministic “VBlank-only VRAM write” enforcement.

### Outcome

- A clean 16x16 proof sprite renders correctly.
- PrimeIgnition boot demo glyphs now render legibly (AUREX-16 is visible and centered).
- This unblocks visual polish work (glow, easing, starfield, etc.) without fighting broken fundamentals.

## [YYYY-MM-DD] — Boot DMA + Sprite Format Validation

- Implemented PrimeIgnition boot module.
- Verified DMA request() → apply() → VBlank gating path.
- Confirmed sprite tiles use 4bpp linear nibble-packed format.
- Corrected earlier planar assumption.
- Successfully rendered first DMA-uploaded glyph tile.
- Identified palette initialization as next required visual foundation step.

2026-03-02
PPU Phase 6.5 — VBlank Gating for VRAM DMA

- DMA apply() now requires vblank=true
- Outside VBlank, writes are silently rejected
- No timing granularity added
- No IRQ added
- Determinism preserved

## 2026-03-02 — PPU VBlank Simulation Introduced

Added a hardware-style `vblank` boolean to the PPU.

- Cleared at start of `render_frame`
- Set true after scanline rendering completes
- No IRQ system yet
- No behavior change

This establishes future-safe DMA gating and proper console timing architecture.

Rendering pipeline remains unchanged and deterministic.

## 2026-03-02 — Hardware Register Bus + Mutation Isolation

Register system fully activated and enforced.

- Address-based PPU register writes live.
- Frame logic now mutates PPU via bus only.
- Direct field mutation removed.
- Debug register driver isolated.
- Rendering pipeline untouched.

System now reflects real hardware layering.

Stable milestone.

## 2026-03-02 — PPU Register Bus Activated (Address-Based Writes Live)

### Summary

PPU register system elevated from enum-only API to hardware-style address bus.

Rendering logic now mutates state exclusively through address-based register writes.

### What Changed

- Added PPU register address map.
- Implemented `write_addr(addr, value)` and `read_addr(addr)`.
- Frame logic now uses address-based writes instead of direct field mutation.
- Scroll auto-increment now reads from and writes to register bus.
- Window control now flows through register bus.

### Architectural Impact

Mutation hierarchy is now:

Frame Logic  
→ Address Bus  
→ write_reg  
→ Internal PPU fields

This prepares Aurex for:

- CPU bus emulation
- Cartridge-driven register writes
- Save-state stability
- Deterministic replay
- Proper hardware layering

No rendering behavior changed.

Pipeline remains deterministic and integer-only.

Stable checkpoint.

2026-03-02 — PPU Phase 5 — Global Sprite Flip + Layer Controls Stabilized
Summary

Completed full global flip logic for sprites and formalized layer enable controls. Rendering pipeline is now multi-layer capable, composite-safe, and fully deterministic under hardware constraints.

Major Additions

Global hflip and vflip support for:

8×8 sprites

16×16 composite sprites (2×2 tile layout)

Flip applied across full composite before tile selection

No tile memory duplication

Deterministic coordinate remapping

No OAM leakage — flip integrated into PPU API

API Change

write_sprite signature expanded:

write_sprite(
index,
x,
y,
tile_index,
palette,
priority,
size_16,
hflip,
vflip,
)

Sprite state mutation now occurs only through PPU interface.

Layer Control Stabilized

bg0_enable

bg1_enable

sprite_enable

Allows deterministic layer isolation and debug gating.

Rendering Integrity

RGB555 preserved

Integer-only compositing

8 sprites per scanline enforced

Overflow telemetry preserved

Scanline render order unchanged:

BG0

BG1 (window-masked)

Sprites

Additive blending during sprite pass

Architecture Status

Rendering pipeline is now:

Dual-layer capable

Window-masked

Per-scanline scroll capable

Multi-size sprite capable

Global flip correct

Deterministic under hardware caps

Stable checkpoint.

2026-03-02 — Rendering Elevation Tier Stabilized
Summary

Rendering pipeline expanded beyond initial baseline while preserving strict hardware constraints and determinism.

System now supports:

Dual background layers (BG0 + BG1)

Per-scanline scroll tables

Vertical window masking

Per-layer enable flags

16×16 sprites

Full sprite flipping (hflip / vflip)

Sprite ↔ BG priority interleave

Additive RGB555 blending

8 sprites per scanline enforcement

Overflow telemetry

Architecture remains deterministic and integer-only.

Dual Background System

Implemented:

BG0 (64×64 tilemap)

BG1 (64×64 tilemap)

Shared 4bpp pattern memory

Independent scroll registers

Per-tile priority bit (bit 14)

BG1 renders after BG0 and overwrites non-transparent pixels.

Per-Scanline Scroll Tables

Added:

bg0_scroll_x_line[FB_H]

bg1_scroll_x_line[FB_H]

Enables:

Raster distortion

Wave effects

Parallax motion

Integer-only math preserved.

Vertical Window Masking

Added:

window_enabled

window_top

window_bottom

BG1 can be vertically clipped per scanline.

Sprites unaffected.

Horizontal windowing not yet implemented.

Layer Enable Flags

Added:

bg0_enable

bg1_enable

sprite_enable

Purpose:

Debug isolation

Compositing validation

SDK readiness

Future register abstraction

No performance regression observed.

16×16 Sprite Support

Sprites now support:

8×8 (default)

16×16 (2×2 tile composite)

Layout:

[ base base+1 ]
[ base+2 base+3 ]

No new VRAM layout.
No tile duplication.
Fully deterministic decode.

Sprite Flipping

Implemented:

hflip

vflip

Flip applies across full sprite composite (8×8 or 16×16).

Global coordinate remapping used.
No additional memory usage.

Stability Verification

Confirmed:

Deterministic frame lifecycle

8-sprite-per-scanline enforcement

Overflow telemetry functional

RGB555 additive blending stable

No floating point contamination

No architectural regressions

Rendering core considered stable and expandable.

Template for Future Entries

When adding new entries, use this format:

## YYYY-MM-DD — Milestone Title

### Summary

Brief overview of change.

### Technical Changes

- Bullet list of implemented systems

### Constraints Check

- Determinism preserved?
- Hardware caps preserved?
- No float usage?
- No architectural rewrite?

### Notes

Optional engineering commentary.

Always insert new entries above older entries.


## 2026-03-08 00:00:00Z — Per-Title AV Profiles + Library Domain Refactor

### Summary
Implemented per-title song selection, per-title color themes, and tiny title graphics in the library UI while tightening system architecture boundaries.

### Changes
- Added `TitleProfile` domain model in library module (title, track id, theme, icon).
- Library selection now emits `AudioCue::SelectTrack(u8)` on change.
- Audio engine now supports 6 title-specific music patterns mapped 1:1 to library entries.
- Library cards now render per-title accent color + tiny icon graphic.
- `Aurex::start_game()` primes audio cue from currently selected title.

### Architecture Rationale
This starts a reusable “content profile” architecture for future cartridge metadata integration: one profile drives both visual and audio presentation from a single selection state.


## 2026-03-08 00:30:00Z — Boot Gating + Start Handshake Refactor

### Summary
Implemented a strict boot gate so the intro can never be interrupted, then added explicit `PRESS START TO CONTINUE` handshake before entering library mode.

### Architecture Changes
- Extended flow state machine to `Boot -> AwaitStart -> Game`.
- Added `waiting_for_start` propagation from runtime flow to boot renderer.
- Boot overlay now displays start prompt only in gate phase.
- Input-driven scene transition now only allowed in `AwaitStart`.

### Rationale
This creates a stable pre-runtime handshake point for future boot options/settings/debug menus while preserving deterministic boot timing.


## 2026-03-08 01:00:00Z — Typed Runtime Event Bus Slice

### Summary
Continued architecture hardening by introducing a typed runtime event bus and replacing direct audio cue polling with event draining.

### Changes
- Added `runtime::event::RuntimeEvent`.
- `Aurex` now buffers and emits runtime events (currently `Audio`).
- Main loop now drains event queue and dispatches by event type.

### Rationale
This creates a clean boundary where core frame simulation emits intent and host orchestration performs side effects. It enables faster feature growth without increasing cross-module coupling.


## 2026-03-08 01:20:00Z — Event Queue Component + Flow Tests

### Summary
Converted runtime event buffering to a dedicated queue component and added deterministic flow-state tests for boot/start gating behavior.

### Changes
- Added `RuntimeEventQueue` (`push`, `drain_to`) in runtime event module.
- Rewired `Aurex` to use queue component instead of raw `Vec<RuntimeEvent>`.
- Added `FlowController` unit tests:
  - boot non-skippable before timer end,
  - timer transition to `AwaitStart`,
  - explicit start requirement for `Game`.

### Rationale
Improves component boundaries and gives fast regression safety for flow semantics while architecture expands.
