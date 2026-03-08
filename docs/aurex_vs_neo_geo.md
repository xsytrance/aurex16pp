# AUREX-16++ vs Neo-Geo — Capability & Vision Comparison (Updated)

Date: 2026-03-08  
Purpose: keep Aurex creatively constrained while targeting Neo-Geo-class or better outcomes in deterministic tooling, orchestration clarity, and authoring velocity.

## Comparison Method
This table compares:
- Neo-Geo historical baseline (high level)
- Aurex current implementation state
- Verdict from Aurex project direction (`>=` preferred; `<` means active upgrade target)

## Core Comparison Table
| Category | Neo-Geo Baseline (reference) | Aurex-16++ Current | Verdict |
|---|---|---|---|
| Determinism / replay posture | Hardware-deterministic but not modern typed telemetry workflow | Explicit deterministic runtime with typed launch + audio commands and rejection semantics | **Aurex >** |
| Authoring workflow | Specialist/manual pipeline | Human + LLM SDK contracts, prompt templates, manifest identity enforcement, preflight script | **Aurex >** |
| Launch orchestration | Cartridge boot flow without modern app-layer stage telemetry | Typed `Pending -> Validating -> Ready/Rejected` launch lifecycle with host diagnostics | **Aurex >** |
| Palette capacity policy | Strong era palette capability | 4096 RGB555 entries + base-index sprite lookup + 16 BG palette banks | **Aurex >=** |
| 2D composition discipline | Strong sprite/tile hardware identity | Deterministic scanline/tile/sprite pipeline under explicit lock rules | **Aurex >=** |
| Audio engine architecture | Iconic multi-voice chip personality | ASU-32 deterministic engine (48 kHz stereo, 12 voices, static instruments, wavetable bank, integer FX) | **Aurex >=** |
| Runtime observability | Limited host-level diagnostics model | Typed runtime events + diagnostics collector + stage/resolve telemetry | **Aurex >** |
| Constraint governance | Hardware constraints implicitly enforced by platform | Hardware-style constraints + explicit structural contract + doc governance | **Aurex >** |

## Practical Interpretation
Aurex already exceeds Neo-Geo-style workflows in **tooling + authoring + host observability** while retaining a constrained hardware identity.  
The remaining “beat Neo-Geo” work is primarily about **content quality depth** (asset richness, composition quality, deterministic replay QA tooling), not abandoning constraints.

## Guardrails (non-negotiable)
- No floating point in core simulation paths.
- Fixed frame cadence and deterministic update model.
- Hard-cap rejection policy (no soft hidden expansion).
- Integer-only core rendering/audio math.
- No tile/sprite/palette memory contract drift without explicit canon update.

## Suggested Upgrade Stack (Next)
1. Deterministic replay + golden capture
   - launch/input/audio event capture
   - golden-frame and golden-audio regression snapshots
2. Audio content quality tooling
   - per-track preset authoring table validation
   - deterministic loudness and clipping diagnostics report
3. Cartridge static analyzer v2
   - schema + budget + region overlap + naming consistency checks
   - JSON output suitable for CI gates
4. Palette pipeline quality tools
   - bank usage heatmaps
   - deterministic palette keyframe script validator
5. Host diagnostics dashboard layer
   - summarized stage/audio command counts per frame window
   - deterministic anomaly flags (reject spikes, overflow counters)

## Decision Rule
If a proposed feature reduces determinism, weakens explicit constraints, or introduces hidden budget scaling, it fails—even if it appears more powerful.

Aurex wins by combining **premium deterministic polish** with **strict, explainable hardware-style limits**.
