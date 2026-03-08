# AUREX-16++ Human Game Creation Guide (v0.1)

## Why this exists
This is the human companion to:
- `docs/llm_sdk_guide.md`
- `docs/llm_prompt_template.md`

Use this guide when *you* are directing an LLM to create a game cartridge for Aurex.

Aurex is deterministic hardware-fantasy, so game requests must be structured and budget-aware.

---

## 1) Non-negotiable limits (you must respect these)
Before writing any prompt, lock these in:

- 60 FPS deterministic frame loop
- Integer-only core gameplay/render logic
- CPU cap: 200,000 ops/frame
- DMA cap: 4 commands/frame
- VRAM upload cap: 64 KB/frame
- Audio upload cap: 16 KB/frame
- VRAM total: 1 MB partitioned
- No deferred DMA, no hidden budget forgiveness

If your game concept assumes violating these, adjust the concept first.

---

## 2) How to ask an LLM for a game (human workflow)

### Step A — Choose a clear scope
Define a small, single-loop concept first:
- one genre
- one primary mechanic
- one progression axis
- one fail state

Bad: “Make a giant open-world MMO.”
Good: “Make a deterministic single-screen arcade shooter with wave progression.”

### Step B — Fill the required prompt contract
Your request to the LLM should include all required sections from `docs/llm_prompt_template.md`:

1. `GAME_ID`
2. `TITLE`
3. `GENRE_TAG`
4. `LOOP_SPEC`
5. `INPUT_MAP`
6. `ASSET_BUDGET`
7. `DMA_PLAN`
8. `AUDIO_PLAN`
9. `FAILSAFE_RULES`
10. `OUTPUT_FILES`

Missing sections = invalid authoring request.

### Step C — Enforce identity consistency
Require that these all match exactly:
- `GAME_ID`
- cartridge folder name `cartridges/<GAME_ID>/`
- manifest line: `game_id=<GAME_ID>`

The runtime resolver rejects mismatches.

### Step D — Require deterministic language
Ask the LLM to explicitly state:
- no floating point gameplay state
- no nondeterministic random behavior without fixed seed progression
- no asynchronous hidden side effects in core simulation

---

## 3) Authoring checklist (copy/paste)
Use this after the LLM responds:

- [ ] `GAME_ID` is lowercase snake_case (`[a-z0-9_]+`)
- [ ] Output includes `cartridges/<GAME_ID>/manifest.txt`
- [ ] Manifest contains `game_id=<GAME_ID>`
- [ ] DMA plan uses valid regions and chunking strategy
- [ ] Asset budget is under caps
- [ ] Runtime loop is explicitly deterministic
- [ ] No float-based gameplay logic requested
- [ ] Fail rules include hard rejection behavior

---

## 4) Suggested human prompt wrapper

You can wrap your creative request with this preface:

> Build this cartridge for Aurex-16++ using strict deterministic constraints and the required prompt contract sections. Treat hardware limits as hard caps. If any requested feature exceeds limits, propose a constrained alternative instead of silently allowing it.

Then append the full template from `docs/llm_prompt_template.md`.

---

## 5) Common mistakes to avoid
- Asking for unconstrained dynamic content systems.
- Omitting `game_id` in the manifest.
- Using mixed identity names (`GAME_ID` doesn’t match folder/manifest).
- Asking for visual/audio budgets without explicit per-frame limits.
- Leaving input mapping ambiguous (“standard controls”).

---

## 6) Definition of done for a generated game
A generated game request is “ready” when:
1. Prompt contract is complete.
2. Identity fields match (`GAME_ID`/folder/manifest `game_id`).
3. Hardware caps are explicitly respected.
4. Runtime launch resolver can map and validate cartridge identity.
