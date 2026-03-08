# AUREX-16++ vs Neo-Geo — Capability & Vision Comparison (Target Canon)

Date: 2026-03-08
Purpose: define a **target comparison contract** so Aurex remains creatively constrained but at least equal to (preferably greater than) Neo-Geo-class outcomes.

## Comparison Method
This table compares:
- Neo-Geo historical baseline (high level)
- Aurex current/target policy
- Target verdict for this program (`>=` required)

## Core Comparison Table
| Category | Neo-Geo Baseline (reference) | Aurex-16++ Position | Verdict |
|---|---|---|---|
| Determinism model | hardware-deterministic but not modern replay-oriented tooling | explicit deterministic runtime + typed events + strict reject semantics | **Aurex >** |
| Authoring workflow | traditional low-level pipelines | human + LLM SDK contracts + manifest identity enforcement | **Aurex >** |
| Runtime observability | limited native telemetry abstraction | launch/event diagnostics + stage telemetry | **Aurex >** |
| Palette capacity policy | high era-typical palette richness | 4096 RGB555 entries + banked selection + deterministic lookup safety | **Aurex >=** |
| Graphics composition policy | strong 2D sprite heritage | deterministic scanline renderer + expandable palette tooling plan | **Aurex >=** |
| Audio composition framework | iconic multi-voice chip character | deterministic lane synth (bass/sub/lead/arp/percussion), envelope shaping, cue model, roadmap to richer instrument tables | **Aurex >= (target)** |
| Development velocity | manual specialist tooling | structured docs, templates, canon handoff, machine-assisted generation | **Aurex >** |
| Constraint discipline | hardware constraints | hardware-style constraints + explicit modern policy docs and validation gates | **Aurex >** |

## Creative Constraint Guardrails (must remain)
- No floating point in core simulation paths.
- Fixed frame cadence, deterministic update model.
- Hard-cap rejection policy (no soft hidden expansion).
- Integer-only rendering/audio core math.
- No tile/sprite format drift without explicit canon revision.

## Priority Upgrades to Keep Aurex >= Neo-Geo
1. Audio instrument table system
   - deterministic instrument preset IDs
   - envelope/wave recipes per track without runtime allocation
2. Palette analytics + animation scripts
   - per-bank usage counters
   - deterministic keyframed palette modulation utilities
3. Deterministic replay/capture
   - golden-frame + golden-audio snapshots
   - launch/input/audio event replay harness
4. Cartridge static analyzer
   - enforce manifest/schema/budget correctness pre-runtime

## Decision Rule
If a proposed feature reduces determinism, weakens constraints, or introduces hidden budget scaling, it fails even if “more powerful.”

Aurex wins by combining **premium output quality** with **strict, explainable constraints**.
