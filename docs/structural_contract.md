AUREX-16++ STRUCTURAL CONTRACT
Governance Layer v1.0

Purpose

This document defines the architectural governance rules for Aurex-16++.

It does not describe hardware.

It enforces discipline.

It exists to:

Prevent architectural drift

Preserve determinism

Maintain hardware-fantasy identity

Stabilize AI collaboration across handoffs

Enforce layered ownership boundaries

Ensure documentation integrity

If any suggestion violates this contract, it must be rejected.

Stability > Novelty.

1. Canon Supremacy Rule

The following file is the single source of hardware truth:

docs/ai_handoff_canon.md

If a proposal contradicts canon:

It is invalid.

It must not be implemented.

Canon can only change via explicit user approval.

No silent hardware evolution.
No soft adjustments.
No reinterpretation.

2. Determinism Enforcement Rule

Aurex-16++ is strictly deterministic.

Never introduce:

Floating point math in VM or PPU

Frame-rate dependent logic

Non-deterministic ordering

Hidden mutable global state

Implicit heap allocation inside core loops

All core rendering must remain:

Scanline-based

Ordered

Integer-only

Predictable

Determinism is non-negotiable.

3. Architectural Layer Lock

System development order is fixed:

Frame Loop

VM Stub

WRAM

DMA

PPU

ASU

Cartridge System

Higher Systems (ECSU, GCU, Achievements, etc.)

No cross-layer shortcuts.

Lower layers must not depend on higher layers.

Higher layers must access lower layers only through defined APIs.

If a proposal violates layering:
Reject it.

4. File Responsibility Enforcement

Each file owns a domain.

Ownership boundaries must not blur.

Examples:

mod.rs → orchestration only

ppu.rs → rendering + register interface only

oam.rs → sprite storage only

vram.rs → VRAM layout only

dma/controller.rs → DMA scheduling only

asu.rs → audio processing only

Files must not reach into internal fields of other systems directly.

If new behavior requires cross-ownership mutation:
Expose an explicit API.

No structural leakage.

5. Edit Format Rule (Mandatory)

All code modifications must use:

FIND → REPLACE blocks.

No abstract instructions.
No partial hints.
No implied changes.

Edits must be:

Copy-paste safe

Minimal

Precise

Large file dumps are prohibited unless explicitly requested.

6. Documentation Protocol Lock

Documentation roles are fixed:

docs/ai_handoff_canon.md

Stable hardware state

No milestone narrative

docs/ai_handoff_history.md

Chronological evolution (oldest → newest at bottom)

docs/dev_log.md

Reverse chronological (newest at top)

Human-readable milestone summaries only

Ordering must never be mixed.

When completing a milestone:

Update canon if necessary

Append to history

Prepend to dev_log

Then perform DCG

No commits without documentation updates.

7. DCG Protocol (Mandatory)

Document → Commit → Git Push

Standard format:

git add .
git commit -m "Milestone Name"
git push

Single commit per milestone.

No double commits for docs.

8. Temporary Test Discipline

Temporary validation logic must:

Be marked // TEMP TEST

Include removal note

Use #[cfg(debug_assertions)] when possible

Not persist silently

Not exist in release builds

Temporary code must not become architecture.

9. Handoff Protocol

When initiating a new chat:

Assume zero runtime state.

Require:

docs/ai_handoff_canon.md

docs/ai_handoff_history.md

docs/structural_contract.md

Architecture must be reconstructed from documentation only.

No redesign mid-conversation.

No speculative restructuring.

10. Hardware Identity Protection

Aurex-16++ is:

Deterministic

2D-only

Tile + sprite scanline pipeline

Constrained

Hardware-inspired

It is not:

A 3D engine

A PC abstraction layer

Unity

A modern general-purpose renderer

All expansions must feel like hardware evolution.

Never engine creep.

11. AI Behavior Guardrail

AI operating under this contract must:

Prioritize stability over novelty

Reject unsafe suggestions

Protect ownership boundaries

Preserve determinism

Update documentation before code

Respect canon supremacy

If uncertain:
Ask.
Do not improvise architecture.

12. Symbol Stability Rule (ABI Lock)

Public-facing symbols are part of the Aurex-16++ stable interface.

This includes:

Struct names

Enum names

Trait names

Public functions

Public methods

Public module names

Public constants

Public type aliases

Once introduced and stabilized, symbols may not be renamed unless:

Explicitly approved by the user

Documented as a breaking change

Updated in all affected files in a single milestone

Logged in docs/ai_handoff_history.md

Prepend-noted in docs/dev_log.md

No stylistic renames.
No refactor-only renames.
No “better name” improvements.

If a rename is required, it must be treated as hardware evolution.

Stability > aesthetics.

13. Symbol Registry Enforcement

The file:

docs/symbol_registry.md

Is the authoritative ledger of locked symbols.

This registry must:

Reflect the stable project state

Be updated only during intentional breaking changes

Be consulted during all handoffs

Be treated as ABI-level truth

If a suggestion introduces a rename that conflicts with the registry:
It must be rejected.

END OF CONTRACT
