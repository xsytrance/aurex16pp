## 2026-03-08 — AV Upgrade Execution (Phase 1–3)

### Summary
Executed the proposed AV architecture upgrade: audio (16 voices, anti-click, bitcrush), graphics (16 sprites/scanline, OAM size_16 fix), boot beat sync, and documentation alignment.

### Phase 1 — Audio
- **Tech spec:** Audio RAM corrected to **512 KB** (was 256 KB).
- **ASU-32:** Voice count increased from 12 to **16**; removed duplicate `AudioDiagnosticsBaseline` and duplicate unit tests.
- **Anti-click:** Per-voice `prev_env_gain` smoothing at envelope boundaries (integer blend of current and previous env_level).
- **Bitcrush:** New FX bit `0x10` in `apply_effects` (6-bit quantize, integer-only).
- Docs: `tech_spec_report.md`, `architecture.md`, `ai_handoff_canon.md` updated for 16 voices and FX.

### Phase 2 — Graphics
- **PPU:** Sprites per scanline increased from 8 to **16** (`evaluate_sprites_for_scanline` and overflow semantics).
- **Sprite size:** Render path now uses OAM `size_16` (8×8 vs 16×16) instead of hardcoded 16.
- **Docs:** Max colors on screen (4096) and sprite limits (16/scanline, 8×8/16×16) documented in tech_spec, architecture, canon.

### Phase 3 — Boot
- **Beat sync:** `AudioEngine::pattern_step()` added; `run_frame(input, boot_beat_step: Option<u8>)` added; PrimeAwakens `draw_overlay(fb, boot_beat_step)` with beat-aligned pulse in status panels (step % 4 == 0).
- **Main:** Passes `boot_beat_step` from synth when in Boot/AwaitStart.

### Phase 4
- Library UI / diagnostics polish deferred (no code change).

### Documentation
- Canon, architecture, tech_spec, symbol_registry (`run_frame` signature) updated. `dev_log` and handoff docs aligned.

### Progress
Build passes (`cargo check`). Determinism and integer-only constraints preserved.

## 2026-03-08 16:20:00Z — Handoff Documentation Synchronization Pass

### Summary
Completed a full documentation synchronization pass for handoff readiness after the runtime/audio/boot upgrades.

### Documentation updates
- Rewrote `architecture.md` to reflect the current frame pipeline, ASU-32 contracts, launch lifecycle, cartridge tooling, and known environment caveats.
- Replaced `ai_handoff_canon.md` with a concise normative contract reflecting the current deterministic runtime and boot audio stability rules.
- Refreshed `arch_index.md` to map all active handoff/governance/runtime docs.
- Added a new handoff snapshot: `ai_handoff_2026-03-08_runtime_av_stage3.md`.
- Updated `ai_handoff_history.md` with latest snapshot ordering and canon-precedence reminder.
- Refreshed AV upgrade notes (`boot_library_upgrade.md`) and master prompt text (`master_prompt.md`) to align with PrimeAwakens and ASU-32 stabilization.
- Updated `test_log.md` with latest environment-accurate command outcomes.

### Progress
Documentation is now aligned with implementation and ready for operator-to-operator handoff continuation.

## 2026-03-08 13:40:00Z — Boot+Library EDM/Hypnotic Upgrade

### Summary
Upgraded boot intro and library presentation to use denser hypnotic visuals and per-title EDM identity using the upgraded runtime/audio contracts.

### Runtime/Tooling
- Reworked `PrimeIgnition` overlay into a tunnel/equalizer-style intro sequence with updated prompt cadence (`PRESS START // GO STRAIGHT`).
- Expanded library title profiles with BPM/style/tag metadata and upgraded card/header/footer rendering for stronger per-title art direction.
- Expanded ASU-32 track routing from 4 to 6 patterns so each dummy title can drive a unique music track id.
- Added `docs/boot_library_upgrade.md` documenting the visuals, track mapping, and deterministic constraints.

### Progress
Boot and library now better reflect the upgraded hardware/audio stack and provide clearer content authoring hooks for style + music identity.

## 2026-03-08 12:45:00Z — Suggestion Stack Deepening (Replay+Analyzer)

### Summary
Extended the initial tooling scaffolds with deterministic replay framebuffer hashing and stronger cartridge analyzer diagnostics.

### Runtime/Tooling
- Replay smoke now runs real `Aurex` frames and hashes deterministic framebuffer samples in addition to input/event tags.
- Cartridge analyzer v2 now flags overlapping upload ranges within the same VRAM region.
- Added analyzer overlap test coverage for deterministic regression protection.

### Progress
Tooling path moved from placeholder smoke metrics toward actionable deterministic regression signals.

## 2026-03-08 12:20:00Z — Suggestion Stack Execution Pass (Tooling Scaffolds)

### Summary
Executed the suggested-upgrade stack with deterministic tooling scaffolds for replay capture, audio diagnostics, cartridge analyzer v2, and palette bank heatmap reporting.

### Runtime/Tooling
- Added cartridge analyzer v2 report path with per-cartridge upload metrics and JSON output.
- Added ASU-32 audio diagnostics report path (peak/average absolute channel metrics over deterministic frame windows).
- Added deterministic replay capture scaffold with hash-based summary output for input/event streams.
- Added BG0 palette bank heatmap JSON utility.
- Added CLI switches in `main.rs`:
  - `--analyze-cartridges [--json]`
  - `--audio-diagnostics [--json] --frames <N> [--boot]`
  - `--palette-heatmap`
  - `--replay-capture-smoke`

### Progress
This completes first-pass implementation for all documented suggestions and sets up next-phase regression tooling without altering core render/CPU/DMA budgets.

## 2026-03-08 11:50:00Z — Neo-Geo Comparison + SDK/LLM Instruction Refresh (Post-Upgrade)

### Summary
Updated comparison and authoring docs to reflect current ASU-32/runtime launch upgrades and to strengthen LLM/human prompt reliability after recent regressions.

### Documentation
- Rewrote `docs/aurex_vs_neo_geo.md` with updated current-state verdicts and explicit next upgrade stack.
- Updated `docs/llm_sdk_guide.md`/`docs/llm_prompt_template.md` alignment usage through canon and index cross-links.
- Updated architecture index audio/library sections to runtime-command-based terminology.
- Updated tech spec suggested upgrade stack to ASU-32 refinement wording instead of stale mono/4-lane direction.
- Added canon-level LLM instruction reliability section and cleaned launch resolve bullet formatting.

### Suggested Upgrades (ready for next pass)
- Deterministic replay/golden capture for launch+audio+input.
- Audio diagnostics report (clipping, per-track level, preset usage).
- Cartridge static analyzer v2 with CI-oriented JSON output.
- Palette bank heatmap + script validator tooling.

## 2026-03-08 11:20:00Z — Documentation Reliability Pass (Architecture/Handoff/SDK/Prompt)

### Summary
Performed a thorough docs synchronization pass to align architecture, handoff canon, SDK, and prompt templates with the current ASU-32/runtime event model and to capture concrete anti-regression guidance.

### Documentation
- Updated architecture to use current palette semantics (`10..13` bank bits, sprite palette base index) and runtime audio command terminology.
- Updated handoff canon library/audio references from legacy cue wording to runtime command wording.
- Rewrote LLM SDK guide to v0.3 with 512 KB audio RAM, ASU-32 command contract, and explicit validation checklist.
- Rewrote LLM prompt template to v0.3 with updated budgets/contracts and final self-check items.
- Updated master prompt hardware memory section (Audio RAM 512 KB) and added documentation reliability guardrails.

### Lessons Learned / Anti-Regression
- Avoid global helper symbol drift in fast iteration; prefer impl-scoped helpers when utility scope is local.
- Make integer overflow intent explicit in deterministic arithmetic paths.
- Keep docs terminology synchronized with actual runtime APIs before shipping claims.
- Keep test/run claims strictly environment-verified.

## 2026-03-08 10:55:00Z — ASU-32 Sine Helper De-dup Hardening

### Summary
Eliminated remaining risk of duplicate `sine_approx` symbol collisions by moving sine shaping to an impl-scoped helper used only by ASU-32 wavetable generation.

### Runtime/Tooling
- Replaced free function usage with `AudioEngine::sine_from_phase(...)`.
- Removed standalone global sine helper symbol from runtime audio module.

### Progress
This prevents duplicate global helper definitions during merge drift while preserving deterministic integer-only synthesis behavior.

## 2026-03-08 10:35:00Z — ASU-32 Wavetable Overflow Fix (Deterministic Noise Seed)

### Summary
Fixed a runtime panic in ASU-32 wavetable generation caused by debug overflow during integer noise seed synthesis.

### Runtime/Tooling
- Replaced overflowing `i32` multiply in noise seed generation with explicit wrapping `u32` arithmetic.
- Added an audio unit test to ensure ASU-32 initialization/render path does not panic under debug overflow checks.

### Progress
`cargo run` panic source in ASU-32 wavetable init is resolved while preserving deterministic integer-only behavior.

## 2026-03-08 10:10:00Z — ASU-32 Audio Engine (48k Stereo / 12 Voices)

### Summary
Implemented ASU-32 runtime audio path with deterministic 48 kHz stereo synthesis, 12 voices, fixed-point mixing, static instruments, wavetable RAM, and runtime audio command dispatch.

### Runtime/Tooling
- Replaced legacy cue-driven mono synth internals with ASU-32 voice engine (`voice[12]`) + ADSR/vibrato instrument table.
- Added wavetable bank generation for sine/square/triangle/saw/noise stored in 512 KB audio RAM.
- Added deterministic fixed-tick pattern sequencer and integer-only stereo mixer with pan weights.
- Added integer-only per-voice optional FX path (delay/echo/bitcrush/distortion).
- Extended runtime audio event model from `AudioCue` intent to `RuntimeAudioCommand` (`PlayTrack`, `PlaySfx`, `StopTrack`) and wired dispatch path.
- Updated host audio queue path to 48 kHz stereo output blocks.
- Updated canon/architecture/tech-spec docs for ASU-32 semantics.

### Progress
Audio runtime now matches ASU-32 implementation target while preserving deterministic execution and integer-only synthesis in the sample loop.

## 2026-03-08 09:05:00Z — CI/Local Preflight Entrypoint Script

### Summary
Added a single script entrypoint for deterministic preflight checks so local and CI invocation paths stay aligned.

### Runtime/Tooling
- Added `scripts/preflight.sh` to run:
  - `cargo fmt -- --check`
  - `cargo check`
  - `cargo run -- --audit-cartridges --json`
- Added `AUREX_SKIP_AUDIT_LINK=1` escape hatch for environments without native SDL2 linking support.
- Updated architecture index and technical spec docs to reference the preflight script.

### Progress
This reduces drift between ad-hoc local checks and CI expectations and makes cartridge audit gating easier to operationalize.

## 2026-03-08 08:35:00Z — Manifest Schema Gate + Upload Budget Validation

### Summary
Extended cartridge preflight validation with an explicit manifest key schema gate and upload budget checks so CI/runtime reject malformed or out-of-bounds content before attach.

### Runtime/Tooling
- Added manifest key registry semantics (`name` optional single, `game_id` required single, `upload` required repeat).
- Added duplicate-key checks for singleton fields (`name`, `game_id`).
- Added upload budget checks for per-upload size cap, region capacity bounds, and palette alignment constraints.
- Added unit coverage for palette-region budget/alignment rejection.

### Progress
This strengthens deterministic preflight by moving structural + memory-safety checks into manifest parsing rather than relying on downstream runtime behavior.

## 2026-03-08 08:10:00Z — Cartridge Audit CLI (Deterministic Preflight)

### Summary
Added a deterministic cartridge audit mode to preflight cartridge manifests and identity coherence before launch/runtime attach attempts.

### Runtime/Tooling
- Added cartridge audit report model and root-scan API in cartridge runtime module.
- Added host CLI switch: `--audit-cartridges` to print per-cartridge status and return non-zero on invalid entries.
- Added `--json` output mode for machine-readable CI/preflight integration.
- Added unit coverage for mixed valid/invalid/missing-manifest cartridge trees.

### Progress
This closes a tooling gap between authoring docs and runtime launch validation by providing a fast preflight gate.

## 2026-03-08 07:40:00Z — Boot Screen + Boot Music Refresh for Expanded AV Capability

### Summary
Since palette/audio capability has increased versus the original boot implementation, the boot presentation was upgraded to use richer deterministic visuals and a stronger boot theme.

### Graphics
- Added deterministic accent rails and a compact boot meter visual to the boot overlay.
- Enhanced backdrop with sparse deterministic star-glint highlights.

### Audio
- Updated boot track motif/BPM and amplitudes for a more assertive startup identity.
- Preserved integer-only deterministic synthesis and existing runtime constraints.

### Progress
Boot now better reflects current platform capability without changing frame timing, determinism, or core constraints.

## 2026-03-08 07:10:00Z — Neo-Geo Comparison Canon + Documentation/SDK/Prompt Alignment

### Summary
Completed a full documentation and instruction alignment pass and introduced an explicit Aurex-vs-Neo-Geo target comparison document to guide future phases.

### Documentation / SDK / LLM Instruction Updates
- Updated SDK guidance and prompt template to reflect current palette/audio semantics and hard constraints.
- Updated architecture/handoff/tech-spec/index docs to reference Neo-Geo comparison planning.
- Added `docs/aurex_vs_neo_geo.md` with category-by-category target verdicts requiring Aurex >= Neo-Geo outcomes while preserving deterministic creative constraints.

### Progress
This gives a unified planning reference for future upgrades and keeps both human and LLM authoring paths aligned with the project vision.

## 2026-03-08 06:40:00Z — Phase Upgrade: Deterministic Audio Lane Depth + Neo-Geo Gap Closure Plan

### Summary
Executed next-phase upgrades by implementing deterministic envelope-shaped lane mixing in runtime audio and updating canon documentation with concrete “beat Neo-Geo while constrained” direction.

### Runtime/Audio
- Added deterministic envelope shaping to core music lane generation.
- Kept explicit sub lane to thicken low-end response without non-deterministic processing.
- Preserved integer-only synthesis and zero-allocation inner-loop behavior.

### Documentation
- Updated SDK, architecture, handoff canon, and tech spec with phase-2 audio capability and roadmap.
- Added additional suggested upgrade list for deterministic tooling, graphics profiling, and runtime replay/lint systems.

### Progress
Aurex now has a stronger production-style audio identity while staying inside strict fantasy-console constraints and deterministic architecture rules.

## 2026-03-08 06:05:00Z — Palette 4096 Canon + AV Direction Refresh + Neo-Geo Positioning

### Summary
Consolidated documentation around the expanded 4096-entry palette system, refreshed architecture/handoff/sdk guidance, and clarified audio positioning versus Neo-Geo while preserving deterministic constraints.

### Runtime/Graphics
- Palette storage now documented as 4096 RGB555 entries.
- Sprite palette semantics documented as base-index lookup behavior.
- BG tilemap palette selection documented as 4-bit bank field (bits 10..13).
- Boot/library visual tone refreshed toward stronger contrast and richer color accents.

### Audio
- Canon docs now explicitly state that current audio quality is stylistically strong but not yet equivalent to Neo-Geo production depth.
- Added constrained upgrade path: deterministic voice lanes, envelope tables, and motif sequencing.

### Documentation
- Updated:
  - `docs/llm_sdk_guide.md`
  - `docs/architecture.md`
  - `docs/ai_handoff_canon.md`
  - `docs/tech_spec_report.md`
  - `docs/dev_log.md`

### Progress
This aligns implementation + docs around a single “premium but constrained” vision: better tooling and deterministic polish, not unconstrained complexity.

## 2026-03-08 04:48:00Z — Consolidated Technical Specification Report

### Summary
Added a single consolidated technical specification report documenting current system capabilities and operational limits.

### Documentation
- Added `docs/tech_spec_report.md` covering:
  - display/FPS
  - CPU/ops budget
  - memory and VRAM budgets
  - DMA limits
  - audio/input/runtime launch pipeline
  - cartridge authoring constraints (LLM + human)
- Added index linkage in `docs/arch_index.md` for discoverability.

### Progress
This gives a single operator-friendly source for “what the system can do now” while preserving canonical constraints.

## 2026-03-08 04:22:00Z — Human Game Creation Guide Aligned to LLM SDK Contract

### Summary
Added a human-facing game creation guide that mirrors LLM prompt contract requirements and deterministic hardware constraints.

### Documentation
- Added `docs/human_game_creation_guide.md` with:
  - hard-limit summary
  - step-by-step human workflow for directing an LLM
  - identity consistency checks (`GAME_ID` / folder / manifest `game_id`)
  - deterministic authoring checklist and DoD criteria
- Linked this guide into architecture/index/master/canon docs to keep human and LLM instructions synchronized.

### Progress
Human-to-LLM authoring now has a canonical, constraint-safe path that matches runtime validation behavior.

## 2026-03-08 04:02:00Z — Manifest Identity Enforcement for LLM Cartridges

### Summary
Added strict manifest identity enforcement so runtime cartridge resolution validates `game_id` against requested `cartridge_id`.

### Runtime / Architecture
- `CartridgeRuntime::from_cartridge_id` now returns typed resolve errors:
  - `MissingManifest`
  - `InvalidManifest(String)`
- Resolver now requires `game_id=` in manifest and checks equality with launch `cartridge_id`.
- Launch pipeline now maps invalid manifest to `LaunchValidationError::CartridgeManifestInvalid`.

### SDK
- LLM SDK guide now documents required `game_id` manifest field.
- Prompt template now includes a manifest snippet with `game_id`.

### Progress
This closes a key identity loophole and makes generated cartridge folders + manifests verifiable before boot attach.

## 2026-03-08 03:44:00Z — Launch Resolve Gate + Library Stage HUD Messaging

### Summary
Continued launch pipeline development by adding cartridge resolution at ready stage and upgrading library HUD messaging for staged launch lifecycle.

### Runtime / Architecture
- Added `CartridgeRuntime::from_cartridge_id(...)` resolver hook.
- When launch reaches `Ready`, runtime now attempts cartridge resolution and emits:
  - `RuntimeEvent::TitleLaunchResolved(LaunchDescriptor)` on success
  - `RuntimeEvent::TitleLaunchRejected(CartridgeMissing)` + `LaunchStage::Rejected` on failure
- Added `RuntimeDiagnostics::launch_resolved` for host orchestration/logging.

### Library Screen
- Library status messaging now reflects full stage state (`Pending`, `Validating`, `Ready`, `Rejected`) rather than simple pending-only text.

### Progress
The launch pipeline now has explicit resolver gating before future boot attach, reducing ambiguity between “ready” and “actually loadable”.

## 2026-03-08 03:22:00Z — Multi-Stage Launch Lifecycle (Validating/Ready/Rejected)

### Summary
Extended launch orchestration from a binary pending state to a deterministic staged lifecycle suitable for cartridge boot attachment.

### Runtime / Architecture
- `LaunchStage` now models `Idle`, `Pending`, `Validating`, `Ready`, and `Rejected`.
- `LaunchIntentController::tick()` advances deterministic validation timing.
- Runtime now emits `RuntimeEvent::TitleLaunchReady(LaunchDescriptor)` when stage reaches `Ready`.

### Validation & Telemetry
- Invalid descriptors now drive both rejection event and stage transition to `Rejected`.
- Runtime diagnostics now surfaces `launch_ready` for host-side orchestration.

### Progress
Launch flow is now close to boot handoff readiness: validation and readiness are explicit states/events rather than implicit timing assumptions.

## 2026-03-08 02:56:00Z — Launch Validation Telemetry + SDK Contract Tightening

### Summary
Added deterministic launch-request validation with explicit reject telemetry and tightened LLM SDK authoring rules around cartridge identity.

### Runtime / Architecture
- Added `validate_launch_descriptor(...)` and `LaunchValidationError` to launch domain.
- Core now rejects invalid launch descriptors and emits `RuntimeEvent::TitleLaunchRejected(LaunchValidationError)`.
- Runtime diagnostics now surfaces `launch_rejected` for host logging/dispatch policy.

### LLM SDK
- SDK guide now documents cartridge ID format constraints (`[a-z0-9_]+`).
- Prompt template now explicitly requires a valid `GAME_ID` that matches runtime `cartridge_id` usage.

### Progress
Launch pipeline now has explicit deterministic rejection semantics, which is required before adding multi-stage cartridge validation/loading.

## 2026-03-08 02:28:00Z — LLM SDK Groundwork + Launch Descriptor Identity

### Summary
Started the explicit LLM authoring SDK path and aligned runtime launch identity with cartridge build identity.

### Runtime / Architecture
- Launch descriptor now carries `{ title, cartridge_id }`.
- `RuntimeEvent::TitleLaunchRequested` now transports full launch descriptor identity.
- Host diagnostics logging now prints both display title and cartridge ID.

### LLM SDK
- Added `docs/llm_sdk_guide.md` with required prompt contract sections and deterministic output rules.
- Added `docs/llm_prompt_template.md` as the baseline prompt skeleton for cartridge generation.

### Progress
This pass establishes deterministic prompt-structure governance and begins bridging runtime launch to cartridge folder identity.

## 2026-03-08 02:02:00Z — Launch Controller Integration + Pending Stage HUD

### Summary
Integrated a dedicated launch-intent controller and stage-change telemetry while extending library HUD to reflect pending launch stage.

### Architecture
- Added runtime launch domain component: `LaunchIntentController` with `LaunchStage::{Idle, Pending(&'static str)}`.
- Core now routes request/cancel through controller and emits `RuntimeEvent::LaunchStageChanged(LaunchStage)` on transitions.
- Runtime diagnostics now includes `launch_stage_changed` for centralized host interpretation.

### Graphics
- Library footer now shows a `PENDING` indicator when launch intent is armed.
- Audio meter bars gain additional height while pending to make stage state visible without relying on logs.

### Progress
Launch intent has moved from ad-hoc booleans toward explicit runtime domain state, preparing clean attachment of cartridge validation/boot steps.

## 2026-03-08 01:37:00Z — Launch Intent Lifecycle Pass (Cancel Path + Cue Split)

### Summary
Fast follow-up pass on library UX + runtime architecture to complete launch intent lifecycle signaling (request + clear) with explicit AV feedback.

### Graphics
- Library footer control hint now includes cancel/clear action (`B/ESC: CLEAR`).
- Existing launch pulse behavior now resets immediately when launch intent is cleared.

### Sound
- Added `AudioCue::Cancel` and a dedicated cancel stinger path in `AudioEngine`.
- Launch and cancel cues are now distinct intents in the audio contract.

### Architecture
- Added `RuntimeEvent::TitleLaunchCanceled`.
- `LibraryUpdate` now carries `launch_canceled` in addition to `launch_requested`.
- `RuntimeDiagnostics` now includes `launch_canceled` and main loop logs clear events.

### Progress
This closes the launch-intent loop from a runtime signaling perspective and keeps host-side logic data-driven through typed events/diagnostics.

## 2026-03-08 01:08:00Z — Library AV Pass: Launch Stinger + Meter + Runtime Diagnostics

### Summary
Continued polish on graphics, sound, and runtime architecture around the library scene while preserving deterministic frame behavior.

### Graphics
- Added a footer audio meter visualization that animates per frame and picks up selected-title color themes.
- Added launch-status pulse tinting for the footer status text after an accept press.

### Sound
- Added `AudioCue::LaunchRequest` and wired an explicit launch stinger synthesis path in `AudioEngine::sfx_sample`.
- Launch stinger now has priority over short confirm beeps so title-open intent has clear audible feedback.

### Architecture
- Added `collect_runtime_diagnostics(events)` and `RuntimeDiagnostics` to centralize host-side interpretation of non-audio runtime events.
- Main loop now uses diagnostics extraction instead of ad-hoc direct event matching.

### Validation
- Updated library tests to assert launch cue emission when launch is requested.

## 2026-03-08 00:22:00Z — Library Launch Request Event + Accept Input Path

### Summary
Implemented the next library milestone: explicit launch intent capture for selected titles with typed runtime telemetry, while keeping deterministic flow and existing track-selection behavior.

### Functional Changes
- Added gameplay-level `accept`/`cancel` input fields to support menu intent expansion beyond directional navigation.
- Library selection update now returns a typed `LibraryUpdate` payload (`audio_cue`, `launch_requested`) instead of only an audio cue.
- Pressing accept on a library card now edge-triggers a launch request intent and updates footer status messaging to show launch pipeline progress text.
- Added runtime event `RuntimeEvent::TitleLaunchRequested(&'static str)` and host logging hook in `main`.

### Validation
- Added unit tests for:
  - selection wrap + deterministic track cue emission
  - edge-triggered launch request behavior (press/hold/release/press)

### Architecture Rationale
This keeps scene simulation deterministic while making “open title” a first-class event boundary. The host can now route launch requests (future cartridge boot flow) without coupling launch side effects into the library scene.

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


## 2026-03-08 01:40:00Z — Runtime Event Dispatch Helper

### Summary
Extracted host-side runtime event dispatch into a dedicated helper to keep `main` orchestration slimmer and reduce repeated event matching boilerplate.

### Changes
- Added `dispatch_runtime_events(engine, events)` to runtime module.
- Main loop now delegates runtime event handling through this helper.

### Rationale
Improves iteration speed by making side-effect dispatch a reusable runtime primitive as additional event types are introduced.


## 2026-03-08 02:00:00Z — Scene Transition Telemetry + Handoff Contract Pass

### Summary
Extended the runtime event model with explicit scene transition telemetry and documented a formal handoff contract across flow/simulation/event/dispatch boundaries.

### Changes
- Added `SceneId` and `RuntimeEvent::SceneChanged`.
- Emitted `SceneChanged(Library)` on boot->library transition.
- Added `Aurex::current_scene()` for runtime introspection.
- Main loop now logs scene transition events before dispatch.
- Updated architecture and handoff documents with contract-level detail.

### Rationale
Improves observability and handoff-readiness without increasing coupling. Future options/menus/router features can consume transition telemetry through existing event channels.


## 2026-03-08 10:20:00Z — Runtime AV Stage 3 follow-through (baseline artifact + regressions + docs-sync + telemetry polish)

### Summary
Implemented the next handoff focus items in order: deterministic audio/replay baseline artifact generation path, regression tests for retrigger and boot voice density policies, preflight docs-sync gate, and launch telemetry formatting normalization.

### Runtime/Tooling
- Added `--generate-runtime-baseline [--frames N] [--out PATH]` to write deterministic JSON containing audio diagnostics baseline + replay-capture smoke summary.
- Added `--docs-sync-check` and wired preflight to run docs-sync before cartridge audit (unless link-limited skip mode is enabled).
- Normalized launch telemetry strings for stage/rejection lines into parser-friendly lower-case formats.

### Audio regression tests
- Added regression coverage asserting unchanged active note+instrument pairs do not retrigger envelope attack.
- Added boot sequencer density guard asserting active boot voices stay under defined threshold.

### Validation notes
- `cargo check` and `cargo check --tests` pass in this container.
- `cargo test` remains blocked by missing system `SDL2` linker dependency (`-lSDL2`).


## 2026-03-08 11:05:00Z — Audio diagnostics depth + preflight docs-sync fallback hardening

### Summary
Continued planned runtime AV/tooling development by strengthening audio quality diagnostics and hardening preflight docs-sync behavior in SDL2-link-limited environments.

### Runtime/Audio
- Extended `AudioDiagnostics` with deterministic quality counters: `crest_l_q10`, `crest_r_q10`, `clipped_l`, `clipped_r`.
- Updated `--audio-diagnostics` human-readable output to include crest/clipping metrics for quick quality triage.
- Added regression assertions that boot/game diagnostics retain non-trivial crest and zero clipping in baseline path.

### Tooling/Preflight
- Fixed `scripts/preflight.sh` behavior so `AUREX_SKIP_AUDIT_LINK=1` still runs docs-sync checks via shell marker fallback before skipping audit execution.
- This closes prior drift risk where skip mode bypassed docs-sync entirely (or required link-dependent `cargo run`).

### Validation
- `cargo fmt` / `cargo check --all-targets` / `cargo check --tests` pass (warnings only).
- `AUREX_SKIP_AUDIT_LINK=1 scripts/preflight.sh` passes and now explicitly reports shell docs-sync fallback pass.
- `cargo test -q` remains SDL2-link-limited in this container.


## 2026-03-08 11:40:00Z — Core AV architecture continuation: deterministic mix profiles

### Summary
Continued core architecture + sound/graphics development with a deterministic audio-mix profile system and host tooling exposure for profile-aware diagnostics/baselines.

### Runtime/Audio
- Added `MixProfile` (`soft`, `default`, `arcade`) with fixed integer coefficients for gain, LP smoothing, and HP decay.
- Added `AudioEngine::new_with_profile(...)` and carried profile through diagnostics simulation path for deterministic parity.
- Wired profile coefficients into master mix shaping path in `render_block(...)`.

### Tooling/CLI
- Added `--audio-profile soft|default|arcade` handling for `--audio-diagnostics`, `--generate-runtime-baseline`, and runtime playback initialization.
- Non-JSON diagnostics output now reports selected profile alongside crest/clipping metrics.

### Progress report
- Core architecture now has a profile dimension for deterministic AV tuning rather than hard-coded one-size output shaping.
- This makes upcoming quality calibration (Neo-Geo-target voicing and cabinet/consumer presets) much easier without contract drift.

### Next planned work
1. Add profile-aware regression assertions (crest/clipping deltas across profiles).
2. Add deterministic boot palette profile path for graphics hardware tone calibration.
3. Reduce module-wide dead-code allow usage by moving to targeted item-level allowances as implementation catches up.


## 2026-03-08 12:10:00Z — Core AV continuation: beat-step API compatibility + envelope smoothing state

### Summary
Continued core architecture/sound/graphics integration by aligning runtime signatures for boot beat-step plumbing and adding envelope gain smoothing state to remove compile drift seen in downstream environments.

### Runtime/Integration
- Updated `Aurex::run_frame` signature to accept `boot_beat_step: Option<u8>` and threaded that parameter through boot overlay rendering path.
- Updated host callsites (main loop + replay smoke helper + `run()`) to pass explicit `None` for now, preserving current behavior while unlocking deterministic beat-step wiring later.

### Audio
- Added `prev_env_gain` voice state and applied per-voice envelope gain smoothing during voice sampling to reduce transient harshness and prevent missing-field compile mismatches in mixed-branch environments.

### Graphics
- Boot overlay now consumes beat-step input parameter and applies a small beat bias to wordmark animation timing (no behavior change when beat-step is absent).

### Progress report
- Core AV architecture is now better aligned for future shared beat-clock synchronization between boot graphics and audio without breaking existing deterministic flow.

### Next planned work
1. Implement actual boot beat-step source from sequencer diagnostics and feed it into `run_frame(...)` instead of `None`.
2. Add profile-aware audio regression assertions and expose profile in baseline artifact metadata.
3. Add deterministic boot palette profiles to continue graphics hardware tone calibration.


## 2026-03-08 17:35:00Z — Fast follow: boot beat-step wiring from audio sequencer

### Summary
Completed the next integration step by wiring live audio sequencer beat-step data into boot frame rendering so boot overlay timing can track runtime musical step state instead of using a placeholder `None` path.

### Runtime/Integration
- Added `AudioEngine::boot_beat_step()` accessor to expose the deterministic sequencer step as a compact `u8`.
- Updated main loop to pass `Some(synth.boot_beat_step())` to `run_frame(...)` during boot-phase audio mode.
- Kept game-phase behavior unchanged (`None`) to avoid coupling game visuals to boot-only timing semantics.

### Progress report
- Boot overlay beat bias is now driven by real runtime sequencer state, closing the immediate API plumbing loop and reducing AV drift risk.

### Next planned work
1. Add a tiny deterministic integration test that verifies boot beat-step propagation influences overlay timing path without changing game path.
2. Surface beat-step in runtime diagnostics output to aid host-side debugging and sync validation.
3. Continue profile-aware regression checks (crest/clipping deltas) and metadata enrichment in baseline artifacts.


## 2026-03-08 17:50:00Z — Fast follow: deterministic beat-step progression test

### Summary
Added a focused deterministic audio unit test to lock in boot beat-step progression semantics and improve confidence while iterating quickly.

### Validation/Tests
- Added `boot_beat_step_tracks_sequencer_progression` in `runtime/audio` tests.
- Test advances exactly `tick_samples` per sequencer step and verifies `boot_beat_step()` increments deterministically.

### Progress report
- Beat-step runtime wiring now has a regression guard, reducing risk of silent API drift during future sequencer refactors.

### Next planned work
1. Add a compact runtime diagnostics field exposing current boot beat-step for host telemetry.
2. Add profile-aware assertion coverage for crest/clipping deltas in diagnostics baseline generation.
3. Continue reducing broad `dead_code` allowances as modules stabilize.


## 2026-03-08 18:05:00Z — Fast follow: expose boot beat-step in audio diagnostics

### Summary
Implemented the next telemetry step by exposing deterministic sequencer beat-step in `AudioDiagnostics` outputs (JSON + human-readable CLI), improving host-side AV sync visibility.

### Runtime/Tooling
- Extended `AudioDiagnostics` with `boot_beat_step` and included it in `to_json()` serialization.
- Updated `--audio-diagnostics` text output to print `boot_beat_step`.
- Added assertions in diagnostics regression test to validate deterministic beat-step value after a fixed frame window.

### Progress report
- Beat-step telemetry is now visible in diagnostics artifacts and command-line triage output, reducing debugging turnaround for sync issues.

### Next planned work
1. Include profile metadata at top-level runtime baseline JSON payload for faster host-side grouping.
2. Add a tiny parser test for diagnostics JSON field presence/shape to avoid accidental telemetry regressions.
3. Continue tightening warning hygiene by replacing broad `dead_code` allows with targeted item-level allowances.


## 2026-03-08 18:20:00Z — Fast follow: baseline payload profile metadata

### Summary
Added top-level `audio_profile` metadata to runtime baseline JSON generation for faster host-side grouping and cross-profile diff workflows.

### Runtime/Tooling
- Updated `--generate-runtime-baseline` payload shape to include `"audio_profile":"soft|default|arcade"` at the top level.
- Keeps existing baseline and replay-smoke payloads unchanged for backward-compatible content use.

### Progress report
- Baseline artifacts now self-describe profile context, reducing triage and indexing overhead in CI/host pipelines.

### Next planned work
1. Add a tiny JSON shape regression test to assert baseline payload includes `audio_profile` and diagnostics beat-step fields.
2. Add profile-aware crest/clipping delta assertions to baseline validation flow.
3. Continue dead-code allowance cleanup in stabilized modules.


## 2026-03-08 18:45:00Z — Fast follow: baseline JSON shape regression guard

### Summary
Added a focused regression guard for runtime baseline JSON shape so profile metadata and diagnostics beat-step telemetry cannot silently drift.

### Runtime/Tooling
- Added `runtime_baseline_json(...)` helper in `main.rs` to centralize baseline payload assembly.
- `--generate-runtime-baseline` now uses the helper, keeping behavior equivalent while improving testability.
- Added unit test `runtime_baseline_json_contains_profile_and_diagnostics_fields` asserting payload includes top-level `audio_profile` and nested diagnostics `boot_beat_step` markers.

### Progress report
- Baseline artifact contract now has explicit automated shape protection for host/CI consumers.

### Next planned work
1. Add profile-aware crest/clipping delta assertions to baseline validation flow.
2. Add first minimal cartridge scaffold (`cartridges/<game_id>/manifest.txt`) and run analyze/audit gates.
3. Continue dead-code allowance cleanup in stabilized modules.


## 2026-03-08 19:05:00Z — Fast follow: profile-aware diagnostics delta assertions

### Summary
Continued planned hardening by adding profile-aware regression assertions to verify deterministic output level ordering across `soft/default/arcade` mix profiles.

### Validation/Tests
- Added `mix_profiles_produce_ordered_level_deltas` in `runtime/audio` tests.
- Test checks monotonic ordering of `avg_abs_{l,r}` and `peak_{l,r}` for `Game` diagnostics over a fixed frame window.
- Test also enforces zero clipping across all three profiles in this deterministic baseline path.

### Progress report
- Mix profile tuning now has explicit regression protection against accidental coefficient inversions or unintended loudness drift.

### Next planned work
1. Add first minimal cartridge scaffold (`cartridges/<game_id>/manifest.txt`) and run analyze/audit gates end-to-end.
2. Add a narrow JSON parser/shape assertion for diagnostics payloads beyond string-contains checks.
3. Continue dead-code allowance cleanup in stabilized modules.


## 2026-03-08 19:25:00Z — Fast follow: new boot cartridge scaffold (`chrome_duo_boot`)

### Summary
Added a brand-new cartridge scaffold inspired by French-touch robot-disco aesthetics for boot-era vibe testing and launch-pipeline validation.

### Cartridge content
- Added `cartridges/chrome_duo_boot/manifest.txt` with valid `game_id` and three uploads (`Palettes`, `BgTiles`, `Bg0Tilemap`).
- Added deterministic binary assets:
  - `palette.bin` (16 RGB555 entries)
  - `bg_tiles.bin` (patterned tile payload)
  - `bg0_map.bin` (64x64 tilemap payload)
- Added a new library profile card (`CHROME DUO BOOT`) wired to `cartridge_id=chrome_duo_boot` so it can be selected and launched from title flow.

### Validation/Tests
- Added `includes_chrome_duo_boot_profile` unit test in `game/library`.
- Verified compile and test-target compile remain green in this environment.

### Progress report
- We now have a concrete first-style cartridge artifact in-repo, closing part of the gap from tooling readiness to real generated-game launch exercises.

### Next planned work
1. Add a minimal cartridge runtime smoke assertion (non-linking path) for manifest+upload parse shape.
2. Add one more cartridge with contrasting style to validate analyzer/audit multi-entry reporting ergonomics.
3. Continue dead-code allowance cleanup in stabilized modules.


## 2026-03-08 19:45:00Z — Fast follow: cartridge asset extension migration (`.bin` -> `.bin.md`)

### Summary
Migrated cartridge binary asset filenames from `.bin` to `.bin.md` for the `chrome_duo_boot` scaffold and updated manifest upload references accordingly.

### Cartridge changes
- Renamed asset files:
  - `palette.bin` -> `palette.bin.md`
  - `bg_tiles.bin` -> `bg_tiles.bin.md`
  - `bg0_map.bin` -> `bg0_map.bin.md`
- Updated `manifest.txt` `upload=` lines to point at the new `.bin.md` filenames.

### Progress report
- Cartridge loader remains extension-agnostic (reads bytes from manifest paths), so this migration keeps runtime behavior while aligning naming with the requested packaging convention.

### Next planned work
1. Add a cartridge analyzer regression asserting `.bin.md` assets load and report expected upload counts/bytes.
2. Add a second content cartridge to exercise multi-cartridge audit/analyze JSON output in CI.
3. Continue dead-code allowance cleanup in stabilized modules.


## 2026-03-08 20:05:00Z — Fast follow: remove tracked binaries and publish recreation spec

### Summary
Removed all tracked binary files from the repository and replaced them with a single markdown recreation guide documenting location, purpose, and technical specs for each artifact.

### Changes
- Deleted tracked cartridge binary assets from `cartridges/chrome_duo_boot`.
- Deleted root snapshot binaries with embedded NUL-byte encoding (`full_pub_snapshot.txt`, `symbol_snapshot.txt`).
- Added `docs/binary_asset_recreation_guide.md` with per-file recreation details and manifest expectations.
- Updated `cartridges/chrome_duo_boot/manifest.txt` to reference canonical recreated asset names (`*.bin`).

### Progress report
- Repository now contains no tracked binary blobs while preserving a clear specification for deterministic artifact regeneration.

### Next planned work
1. Add a script to regenerate `chrome_duo_boot` assets from the markdown spec.
2. Add CI check to reject future tracked binary files unless explicitly allowed.
3. Continue dead-code allowance cleanup in stabilized modules.


## 2026-03-08 20:25:00Z — Sound fix: boot tempo, library BPM ticks, instant track switch, per-track FX

### Summary
Implemented the requested audio timing fix set so boot and library playback use correct musical timing, track changes start immediately, and library tracks have distinct timbral character.

### Audio runtime changes
- Added dedicated boot tick constant: `BOOT_TICK_HZ = 8`.
- Added `TRACK_BPM` and mode-dependent tick scheduling via `tick_samples_for_mode(...)`.
- Updated sequencer advance logic to use dynamic tick interval per mode (`Boot` vs `Game`).
- Updated `PlayTrack` handling to force immediate next-step trigger by setting:
  - `pattern_step = PATTERN_STEPS - 1`
  - `tick_counter = tick_samples_for_mode(Game)`
- Added `TRACK_FX` and applied `base_fx | track_fx` in `note_on(...)` for game tracks.

### Main loop ordering
- Moved audio block rendering to occur **after** `run_frame -> drain_events -> dispatch_runtime_events`, so track changes affect the same-frame queued block.

### Validation/Tests
- Added test `play_track_starts_on_next_tick_immediately` to guard immediate-start behavior.
- Existing diagnostics/beat-step/profile tests remain compiling.

### Progress report
- Boot melody speed and library speed now map to intended timing conventions, and selection changes audibly respond without extra frame/tick lag.

### Next planned work
1. Add a tiny deterministic test for BPM-derived step-rate sanity across all tracks.
2. Add an A/B diagnostics helper for profile+track timing telemetry snapshots.
3. Continue dead-code allowance cleanup in stabilized modules.


## 2026-03-08 21:05:00Z — Audio engine sync to provided reference implementation

### Summary
Replaced `src/aurex/runtime/audio.rs` behavior to match the provided reference implementation for boot/library timing, track character, and boot mix behavior.

### Audio/runtime changes
- Synced `MixProfile` to include `Boot` profile with dedicated LP/HP settings for boot rendering path.
- Synced timing constants and scheduling model (`BOOT_TICK_HZ = 4`, BPM-based game ticks with `sample_rate * 15 / bpm`).
- Synced track content model to melody/bass/arp + optional percussion layers with expanded voice budget and per-track instrument routing.
- Synced `PlayTrack` immediate-start behavior and per-track FX overlay model (`TRACK_FX`).
- Synced boot sequencer to the provided single-voice dramatic melody path.
- Added PCM channel model and `PlayPcm` command handling paths from the provided implementation.

### Compatibility glue
- Preserved `boot_beat_step` in diagnostics/output and `boot_beat_step()` accessor for existing host/overlay integration points.
- Extended runtime event enums to include new provided audio commands/SFX variants.

### Progress report
- Audio engine now follows the provided known-good reference structure rather than incremental local patching, reducing drift risk versus the target build.

### Next planned work
1. Validate in an SDL2-enabled environment for audible crackle/distortion confirmation.
2. Add focused unit assertions for `PlayPcm` command state transitions.
3. Continue dead-code allowance cleanup once AV behavior stabilizes.
