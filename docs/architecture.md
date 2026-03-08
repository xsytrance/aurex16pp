# AUREX-16++ Architecture

_Last updated: 2026-03-08 (post PrimeAwakens + ASU-32 boot-audio stabilization)._

## 1) System intent

Aurex-16++ is a deterministic 2D fantasy console with hardware-style constraints:

- Fixed timestep: 60 FPS.
- Integer-only core simulation/rendering/audio math.
- Strict memory and DMA budgets.
- Host-facing typed runtime events for scene/audio/launch diagnostics.
- Cartridge-facing deterministic contracts for authoring and validation.

The runtime is intentionally "fantasy hardware," not a general-purpose engine.

---

## 2) Runtime frame pipeline

Per frame (`Aurex::run_frame`):

1. Begin frame clocks (`Clock`, `Pdu`).
2. Clear framebuffer.
3. Begin DMA frame and run VM frame.
4. Execute scene update:
   - `Boot` mode: `PrimeAwakens::update`.
   - `Game` mode: `LibraryScreen::update` + launch intent validation/lifecycle updates.
5. Render PPU frame into framebuffer.
6. Draw scene overlay:
   - `PrimeAwakens` visuals in boot.
   - Library UI in game.
7. Ingest PPU and DMA telemetry into PDU.
8. End frame and advance UI frame counter.

This order is deterministic and stable across runs.

---

## 3) Memory and addressing

### WRAM
- 512 KB.

### VRAM
- 1 MB total.
- Regioned layout with hard bounds checks in VRAM/DMA paths.
- Palette capacity expanded to 4096 RGB555 entries while preserving legacy compatibility assumptions for older content.

### Audio RAM
- 512 KB, isolated from VRAM domains.
- Used by ASU-32 wavetable data.

---

## 4) Graphics architecture (PPU)

### Resolution and format
- Native framebuffer: 426x240.
- Pixel format: RGB555.

### BG pipeline
- Tile-based BG layers.
- Integer-only address + palette lookup paths.
- Palette-bank handling updated for expanded palette space.

### Sprite pipeline
- Deterministic scanline evaluation.
- Overflow latching/telemetry exposed to runtime/PDU.
- Priority and blend semantics are explicit in PPU codepaths.

---

## 5) Audio architecture (ASU-32 runtime path)

`AudioEngine` provides deterministic, integer-only synthesis:

- 48 kHz stereo output.
- 12 voices.
- Wavetable generation at startup in Audio RAM.
- Envelope stages: Attack / Decay / Sustain / Release.
- Runtime commands:
  - `PlayTrack(track_id)`
  - `PlaySfx(Launch|Cancel|Confirm)`
  - `StopTrack`

### Boot audio mode
Boot now uses a dedicated sequencer branch (`advance_boot_sequencer`) with curated voice placement and FX, separate from game track sequencing.

### Boot fuzz regression root cause + fix
After the ASU-32 upgrade, fuzziness came from envelope hard-retrigger behavior on stable notes plus over-dense noise/percussion layering. The fix:

- Skip envelope retrigger when note+instrument remain unchanged on active voices.
- Preserve release behavior on note-off.
- Keep phase reset only on true off->on transitions.
- Reduce harsh boot percussion/fx density.

This produced smoother sustained timbre without removing boot energy.

---

## 6) Typed runtime events and host diagnostics

Runtime emits typed events consumed by host loop diagnostics, including:

- Scene transitions.
- Audio command events.
- Launch lifecycle events:
  - request
  - canceled
  - stage changed
  - ready
  - resolved
  - rejected

`collect_runtime_diagnostics` converts event streams into host-readable snapshots.

---

## 7) Launch lifecycle domain

Launch flow is explicit and validated:

- `LaunchDescriptor` identity payload (`title`, `cartridge_id`, `track_id`, etc.).
- `validate_launch_descriptor` enforces structural checks.
- `LaunchIntentController` drives deterministic stage transitions.

Cartridge resolution failures map to typed rejection reasons (missing/invalid manifest).

---


### Launch telemetry formatting
Launch-stage logs are now normalized for host integration rather than Rust debug enums:

- `Launch stage: pending title=<title> cartridge=<id>`
- `Launch stage: validating title=<title> cartridge=<id>`
- `Launch stage: ready title=<title> cartridge=<id>`
- `Launch rejected: reason=<snake_case_reason>`

This keeps telemetry parsing stable for downstream UI/event ingest.

## 8) Cartridge tooling and validation

`CartridgeRuntime` now includes:

- Manifest parsing/validation.
- Identity checks (e.g. `game_id` consistency).
- Upload budget checks.
- Overlap/bounds analysis.
- Audit and analyze report generation.

CLI flags in `main.rs` expose deterministic tooling entrypoints for local/CI diagnostics.

---

## 9) Tooling entrypoints

Key commands:

- `--audit-cartridges [--json]`
- `--analyze-cartridges [--json]`
- `--audio-diagnostics [--boot] [--frames N] [--json]`
- `--generate-runtime-baseline [--frames N] [--out PATH]`
- `--docs-sync-check`
- `--palette-heatmap`
- `--replay-capture-smoke`

Plus `scripts/preflight.sh` for formatting/check + docs-sync + cartridge audit gate.

---

## 10) Current known environment caveat

In minimal Linux containers without system SDL2, `cargo check` succeeds but link-dependent `cargo test`/`cargo run` can fail at link stage. This is an environment dependency issue, not a deterministic core logic issue.

---

## 11) Handoff checklist

Before further feature work:

1. Run preflight (`scripts/preflight.sh`).
2. Run `--audio-diagnostics --boot --frames 48000 --json` and compare against prior baselines.
3. Run cartridge audit/analyze in JSON mode and diff outputs.
4. Verify launch event telemetry in logs for request->stage->ready->resolved path.
5. Keep canonical docs synchronized (`ai_handoff_canon.md`, `arch_index.md`, `dev_log.md`, `test_log.md`).
