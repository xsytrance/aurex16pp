# AUREX-16++ AI HANDOFF — CANON

_Last synchronized: 2026-03-08._

This file is the canonical state contract for handoffs. It is normative.

## 1) Locked hardware profile

- Resolution: **426x240**
- Frame rate: **60 FPS fixed**
- Color format: **RGB555**
- CPU budget: **200,000 ops/frame**
- WRAM: **512 KB**
- VRAM: **1 MB**
- Audio RAM: **512 KB**
- Core rule: **integer-only deterministic simulation/render/audio**

## 2) DMA budgets (hard)

- Max DMA commands/frame: **4**
- Max VRAM bytes/frame: **64 KB**
- Max Audio bytes/frame: **16 KB**
- Over-budget/invalid operations are rejected.

## 3) Rendering order contract

Canonical compositing order per scanline/frame path:

1. BG layers
2. Sprites
3. Overlay/UI composition

Deterministic ordering is mandatory; no nondeterministic blending order.

## 4) Palette and tile semantics

- Palette address space supports up to **4096 RGB555 entries**.
- Legacy assumptions for lower palette banks remain supported.
- Tile/sprite palette interpretation must remain explicit and bounds checked.

## 5) Runtime mode contract

Runtime modes:

- `Boot` (PrimeAwakens overlay)
- `Game` (Library + launch flow)

Boot-to-game transition is explicitly controlled by flow state and start input edge behavior.

## 6) Audio contract (ASU-32 path)

`AudioEngine` canonical behavior:

- 48 kHz stereo deterministic synthesis
- 12 voices
- wavetable-backed voice sampling
- ADSR-style envelope states
- command API:
  - `PlayTrack(u8)`
  - `PlaySfx(AudioSfx)`
  - `StopTrack`

Boot and Game audio mode routing are distinct and deterministic.

## 7) Boot audio stability rule

To avoid fuzzy/clicky boot audio:

- Do **not** hard-retrigger envelope attack each tick for unchanged active note+instrument pairs.
- Use note-off transitions for release behavior.
- Reserve phase reset for off->on starts.
- Keep boot percussion/noise density below sustained fuzz threshold.

## 8) Launch lifecycle canon

Launch domain uses typed stages and validations:

- descriptor validation first
- request/cancel/stage/ready/resolved/rejected event model
- cartridge resolution gates readiness

No implicit launch side paths are allowed.

## 9) Cartridge contract

- Manifest identity and format validation are mandatory.
- Upload budgets and overlap checks are mandatory.
- Audit/analyze tooling must remain deterministic and JSON-capable.

## 10) Host diagnostics contract

`collect_runtime_diagnostics` is the canonical host interpretation surface for runtime events.

Any new runtime event type should include diagnostics impact analysis during handoff updates.

## 11) Handoff discipline

On every major runtime change, update at minimum:

- `docs/architecture.md`
- `docs/arch_index.md`
- `docs/dev_log.md`
- `docs/test_log.md`
- `docs/ai_handoff_history.md`

And add a dated handoff snapshot document when scope is broad.
