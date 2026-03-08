# AUREX-16++ MASTER PROMPT (Handoff-Oriented)

Use this prompt framing when continuing implementation across agent/operator handoffs.

## Identity

Aurex-16++ is a deterministic 2D fantasy console runtime.

It is:
- hardware-inspired
- constrained
- integer-only in core loops
- strict about caps and validation

It is not:
- a general-purpose game engine
- a floating-point-first renderer
- a soft-fail permissive runtime

## Hard constraints

- 60 FPS fixed
- 426x240 framebuffer
- RGB555 color
- WRAM 512 KB
- VRAM 1 MB
- Audio RAM 512 KB
- DMA command/data caps enforced

## Architectural priorities

1. Determinism over novelty.
2. Typed contracts over ad-hoc side channels.
3. Explicit validation over implicit tolerance.
4. Handoff readability over cleverness.

## Current implementation anchors

- Boot visual module: `PrimeAwakens`
- Runtime audio path: ASU-32 `AudioEngine`
- Launch domain: `LaunchDescriptor` + `LaunchIntentController`
- Host diagnostics: `collect_runtime_diagnostics`
- Cartridge verification: audit/analyze + manifest validation

## Coding expectations

- Preserve deterministic ordering.
- Avoid hidden state mutations across subsystems.
- Keep runtime event payloads typed and stable.
- Document every meaningful contract change in docs before handoff.

## Handoff update checklist

When making meaningful runtime changes, update:

- `docs/architecture.md`
- `docs/ai_handoff_canon.md`
- `docs/ai_handoff_history.md`
- `docs/dev_log.md`
- `docs/test_log.md`
- `docs/arch_index.md`

## Output expectation for next operator

Provide:
1. clear summary of changed runtime contracts,
2. exact validation commands + outcomes,
3. known environment caveats,
4. next recommended implementation step.
