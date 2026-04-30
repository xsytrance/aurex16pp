# AUREX-16++ Agent Console Plan

**Date:** 2026-04-30  
**Version:** 0.1  
**Status:** Planning Phase  
**Owner:** xsytrance  

---

## Executive Summary

Aurex-16++ is a deterministic 2D fantasy console with complete hardware-style runtime, typed launch lifecycle, and LLM SDK documentation. **This plan converts it into an agent-native game console where AGENTS design & play games, and HUMANS watch.**

The key insight: Aurex-16++ already has **90% of the plumbing**. We need to wire the missing 10%—cartridge execution—and expose agent-facing APIs for generation, validation, and playback.

**Timeline estimate:** 7-9 weeks for MVP (Phases 1-4)  
**Complexity:** Moderate (2D is easier than 3D Aurex X)  
**Determinism:** Guaranteed (integer-only core, typed events, replay capture)

---

## Vision Statement

> **Aurex Agent Console** is a deterministic 2D game platform where AI agents autonomously design, validate, play, and iterate on cartridges. Humans can spectate agent runs, review replays, and curate agent-generated libraries. All game execution is 100% deterministic, reproducible, and auditable.

---

## Core Principles (Non-Negotiable)

1. **Determinism First** — No floating point in core simulation, no hidden randomness, no frame-order dependencies.
2. **Typed Contracts** — Every agent→runtime interaction uses explicit, documented payloads.
3. **Hardware-Fantasy Identity** — Preserve Aurex's constrained, integer-only, scanline-rendering philosophy.
4. **Documentation → Commit → Push** — Follow the DCG protocol for all milestones.
5. **Canon Supremacy** — If a proposal contradicts `ai_handoff_canon.md`, reject it.

---

## System Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    HUMAN SPECTATOR LAYER                        │
├─────────────────────────────────────────────────────────────────┤
│  Library UI (agent games)  │  Replay Viewer  │  Live Diagnostics│
└─────────────────────────────────────────────────────────────────┘
                                ↓
┌─────────────────────────────────────────────────────────────────┐
│                    AGENT API LAYER                              │
├─────────────────────────────────────────────────────────────────┤
│  Prompt→Cartridge Generator  │  Validation CLI  │  Replay APIs   │
│  (HTTP/CLI)                  │  (--analyze)     │  (input seq)   │
└─────────────────────────────────────────────────────────────────┘
                                ↓
┌─────────────────────────────────────────────────────────────────┐
│                    CARTRIDGE RUNTIME                            │
├─────────────────────────────────────────────────────────────────┤
│  Manifest Validation  │  Upload Budget  │  DMA Scheduling       │
│  ─────────────────────────────────────────────────────────────  │
│  VRAM Upload  │  Audio RAM Upload  │  Event Telemetry           │
└─────────────────────────────────────────────────────────────────┘
                                ↓
┌─────────────────────────────────────────────────────────────────┐
│                    AUREX-16++ FANTASY CONSOLE                   │
├─────────────────────────────────────────────────────────────────┤
│  PPU (426×240, 60 FPS, integer)  │  ASU-32 Audio (16 voices)   │
│  VM-32 CPU (200k ops/frame)      │  DMA Controller (4 cmds/fr) │
│  WRAM (512 KB)  │  VRAM (1 MB)  │  Audio RAM (512 KB)          │
└─────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Cartridge-to-Game Bridge (Weeks 1-2)

### Objective
Wire validated cartridges into actual game execution. Currently, `TitleLaunchResolved` logs success but does nothing.

### Deliverables

1. **`GameRuntime` Trait** (src/aurex/runtime/game_runtime.rs)
   - `fn initialize(&mut self, cartridge: &CartridgeRuntime)`
   - `fn update(&mut self, input: InputState, ops_budget: u32) -> GameOutcome`
   - `fn render(&self, ppu: &mut Ppu, dma: &mut DmaController)`
   - `fn shutdown(&mut self)`
   - Returns typed `GameOutcome` (Running, Paused, Completed, Failed)

2. **Cartridge Load After Resolution** (src/aurex/runtime/launch.rs)
   - After `TitleLaunchResolved`, call `GameRuntime::initialize()`
   - Queue cartridge DMA uploads into VRAM/Audio RAM
   - Emit `TitleLaunchExecuted` event for host loop

3. **Event Telemetry Expansion** (src/aurex/runtime/event.rs)
   - Add `GameStarted(cartridge_id)`
   - Add `GamePaused` / `GameResumed`
   - Add `GameCompleted(score: u32)` / `GameFailed(reason)`
   - Add `GameCpuRejects(count: u32)` for overflow telemetry

4. **Host Loop Integration** (src/main.rs)
   - After `TitleLaunchResolved`, attach `GameRuntime`
   - Per-frame: `game_runtime.update(input)` → render
   - Display game status in library footer (like pending indicator)

5. **Documentation**
   - Update `ai_handoff_canon.md` with new events
   - Update `architecture.md` with game runtime pipeline
   - Update `dev_log.md` with milestone entry

### Acceptance Criteria

- ✅ Cartridge selected in library → launches into game execution
- ✅ Game runs deterministically at 60 FPS
- ✅ CPU reject counts exposed via runtime diagnostics
- ✅ Game lifecycle events visible in console logs
- ✅ `cargo run -- --audit-cartridges` still passes
- ✅ DCG protocol followed (docs → commit → push)

---

## Phase 2: Agent Game Design Interface (Weeks 3-5)

### Objective
Expose agent-facing APIs for prompt→cartridge generation, validation, and asset creation.

### Deliverables

1. **HTTP API Server** (src/aurex/runtime/http_server.rs)
   - `POST /api/v1/cartridge/generate` — accepts LLM prompt contract
   - `POST /api/v1/cartridge/validate` — runs `--analyze-cartridges` logic
   - `GET /api/v1/cartridge/{id}` — returns cartridge metadata
   - `POST /api/v1/assets/tilemap` — generates tilemap from prompt
   - `POST /api/v1/assets/palette` — generates palette from prompt
   - `POST /api/v1/assets/sound` — generates ASU-32 sound pattern

2. **LLM→Cartridge Generator** (crates/aurex-agent-gen/)
   - Accepts 11-section prompt contract (from `llm_prompt_template.md`)
   - Generates:
     - `manifest.txt` with validated `game_id` and `upload` entries
     - Binary tilemaps (4bpp, 32 bytes/tile)
     - Binary palette (RGB555 u16 words)
     - Binary sound patterns (wavetable + sequencer data)
   - Returns base64-encoded cartridge bundle

3. **Asset Generation Primitives** (src/aurex/assets/)
   - `tilemap::generate(width: u16, height: u16, seed: u64) -> Vec<u8>`
   - `palette::generate(colors: usize, seed: u64) -> Vec<u16>`
   - `sound::generate_pattern(voice: u8, wave_type: Wave, duration: u16) -> Vec<u8>`
   - All integer-only, deterministic by seed

4. **Validation CLI Enhancement** (src/main.rs)
   - Add `--validate-prompt` flag: parse 11-section prompt, return JSON
   - Add `--generate-test-cartridge` flag: generate minimal valid cartridge
   - Integrate asset budget checks into validation

5. **Documentation**
   - `docs/agent_api.md` — HTTP API spec with examples
   - `docs/agent_workflows.md` — sample agent workflows
   - Update `llm_sdk_guide.md` with agent-first language
   - Update `dev_log.md` with milestone entry

### Acceptance Criteria

- ✅ Agent can POST prompt contract → receive valid cartridge bundle
- ✅ Validation CLI accepts 11-section prompts and returns errors
- ✅ Asset generation primitives are deterministic (same seed = same output)
- ✅ HTTP API is optional (can be disabled with `--no-http-server` flag)
- ✅ All agent-generated cartridges pass `--analyze-cartridges`
- ✅ DCG protocol followed

---

## Phase 3: Agent Playback & Replay (Weeks 6-7)

### Objective
Enable agents to "play" cartridges by submitting input sequences, capture replays, and iterate.

### Deliverables

1. **Deterministic Replay Capture** (src/aurex/runtime/replay.rs)
   - Extend `ReplayCapture` to include:
     - Input sequence (per-frame `InputState`)
     - Framebuffer hashes (for golden-frame regression)
     - Audio diagnostics (per-frame peak/clipping)
     - CPU/DMA reject counts
   - `ReplayCapture::summary_json()` returns deterministic summary

2. **Input Sequence API** (src/aurex/runtime/input_sequence.rs)
   - `InputSequence::from_frames(frames: Vec<InputState>) -> Self`
   - `InputSequence::play(&mut GameRuntime, &mut ReplayCapture)`
   - `InputSequence::save(&self, path: &Path)`
   - `InputSequence::load(path: &Path) -> Self`

3. **Agent Playback Server** (src/aurex/runtime/http_server.rs)
   - `POST /api/v1/playback/run` — accepts `cartridge_id` + `input_sequence`
   - `GET /api/v1/playback/{run_id}` — returns replay summary
   - `GET /api/v1/playback/{run_id}/framebuffer/{frame}` — returns PNG
   - `DELETE /api/v1/playback/{run_id}` — cleanup

4. **Replay Viewer** (web/replay-viewer/)
   - Simple React/HTML viewer for replay summaries
   - Frame-by-frame scrubbing with framebuffer display
   - Audio waveform visualization (from replay audio diagnostics)
   - Side-by-side comparison (run A vs run B)

5. **Achievement System** (src/aurex/runtime/achievements.rs)
   - Implement designed-but-not-wired achievement API
   - `Achievement::check(game_id: &str, event: &GameEvent)`
   - `Achievement::unlock(profile: &mut Profile, achievement: Achievement)`
   - Persistent JSON profile storage (`~/.aurex/profiles/{cartridge_id}.json`)

6. **Documentation**
   - `docs/replay_protocol.md` — replay capture format spec
   - `docs/achievement_api.md` — achievement check/unlock spec
   - Update `dev_log.md` with milestone entry

### Acceptance Criteria

- ✅ Agent can submit input sequence → get deterministic replay
- ✅ Replay viewer displays frames + audio diagnostics
- ✅ Achievement system unlocks based on game events
- ✅ Golden-frame regression works (same inputs = same framebuffer hash)
- ✅ DCG protocol followed

---

## Phase 4: Library + Human Interface (Weeks 8-9)

### Objective
Upgrade library UI to show agent-generated games and enable human spectator mode.

### Deliverables

1. **Library UI Upgrade** (src/aurex/game/library.rs)
   - Display games in grid layout (agent name, game title, thumbnail)
   - Add "Agent-Generated" badge for cartridges from agent API
   - Show game stats (play count, avg score, achievement count)
   - Add "Play Agent Run" button to load replay

2. **Spectator Mode** (src/main.rs)
   - `--spectator` flag: skip input polling, play replay instead
   - `--replay-path /path/to/replay.json` — load replay file
   - `--speed X` — play at X× speed (1×, 2×, 4×, 8×)
   - `--pause` — pause on first frame, continue on any key

3. **Game Curator API** (src/aurex/runtime/http_server.rs)
   - `GET /api/v1/games` — list all cartridges with metadata
   - `GET /api/v1/games/{id}` — get game details + stats
   - `GET /api/v1/games/{id}/replays` — list replays for game
   - `GET /api/v1/games?tag={tag}&agent={agent}` — filter/search

4. **Standalone Export** (crates/aurex-export/)
   - `aurex-export --cartridge cartridges/{id} --out game.bin`
   - Bundles cartridge + minimal runtime into single binary
   - `game.bin --run` — plays cartridge without library UI
   - `game.bin --replay replay.json` — plays replay

5. **Documentation**
   - `docs/spectator_mode.md` — spectator CLI flags + usage
   - `docs/game_curator_api.md` — game listing/filtering spec
   - `docs/standalone_export.md` — export format + usage
   - Update `dev_log.md` with milestone entry

### Acceptance Criteria

- ✅ Library shows agent-generated games with thumbnails
- ✅ Spectator mode plays replays at adjustable speed
- ✅ Game curator API supports filtering by tag/agent
- ✅ Standalone export produces runnable game binary
- ✅ DCG protocol followed

---

## Example Agent Workflows

### Workflow 1: Agent Designs a Game

```
1. Agent receives prompt: "Design a simple arcade shooter for Aurex-16++"
2. Agent fills out 11-section prompt contract:
   - GAME_ID: cosmic_blaster
   - TITLE: COSMIC BLASTER
   - GENRE_TAG: shooter
   - LOOP_SPEC: wave-based shooting, score progression
   - INPUT_MAP: arrows move, A shoots
   - ASSET_BUDGET: under DMA caps
   - DMA_PLAN: tiles, tilemap, palette uploads
   - PALETTE_PLAN: 32 colors, space theme
   - AUDIO_PLAN: shoot SFX, explosion SFX, track 0
   - FAILSAFE_RULES: reject if budget exceeded
   - OUTPUT_FILES: cartridges/cosmic_blaster/*
3. Agent POSTs prompt to `/api/v1/cartridge/generate`
4. Agent receives base64 cartridge bundle
5. Agent decodes and writes to `cartridges/cosmic_blaster/`
6. Agent runs `aurex16pp --analyze-cartridges --json`
7. If valid, agent POSTs to `/api/v1/playback/run` with test input sequence
8. Agent reviews replay, iterates on design if needed
```

### Workflow 2: Agent Plays a Game

```
1. Agent receives prompt: "Find the highest score for cosmic_blaster"
2. Agent loads game runtime + known good input patterns
3. Agent runs 1000 trials with varied inputs
4. Agent captures replays for top 10 scores
5. Agent uploads replays to `/api/v1/playback/{run_id}`
6. Agent returns best score + replay path
```

### Workflow 3: Human Spectates Agent Play

```
1. Human starts `aurex16pp --spectator --replay-path replay.json --speed 4`
2. Human watches agent play at 4× speed
3. Human pauses on interesting moments
4. Human reviews framebuffer + audio diagnostics
5. Human saves replay to library
```

---

## Success Metrics

| Phase | Metric | Target |
|-------|--------|--------|
| 1 | Cartridge launches from library | 100% success rate |
| 2 | Agent-generated cartridges pass validation | 100% compliance |
| 3 | Replay determinism (same inputs = same hash) | 100% match |
| 4 | Standalone export runs without library | 100% compatibility |

**Overall MVP Success:** Agent can autonomously design → validate → play → iterate on a cartridge, and human can spectate the result.

---

## Technical Constraints (Reiterating Canon)

- **Resolution:** 426×240, RGB555, 60 FPS fixed
- **CPU Budget:** 200,000 ops/frame (hard cap)
- **DMA Caps:** 4 commands/frame, 64 KB VRAM/frame, 16 KB audio/frame
- **Memory:** 512 KB WRAM, 1 MB VRAM, 512 KB Audio RAM
- **Core Math:** Integer-only, no floating point in VM/PPU paths
- **Determinism:** No hidden randomness, no frame-order dependencies
- **Documentation:** DCG protocol mandatory for all milestones

---

## Next Steps

1. ✅ **This plan approved** (user sign-off)
2. ⏭️ **Phase 1 Implementation** (Cartridge-to-Game Bridge)
   - Create `GameRuntime` trait
   - Wire cartridge execution after `TitleLaunchResolved`
   - Add game lifecycle events
   - Follow DCG protocol

---

## References

- `docs/architecture.md` — current runtime architecture
- `docs/ai_handoff_canon.md` — hardware canon (supreme authority)
- `docs/llm_sdk_guide.md` — LLM cartridge authoring contract
- `docs/llm_prompt_template.md` — required 11-section prompt template
- `docs/structural_contract.md` — governance and discipline rules
- `docs/aurex_vs_neo_geo.md` — capability comparison target

---

**END OF PLAN**
