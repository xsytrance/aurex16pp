# Aurex-16++ Agent Playground — Implementation Plan

## System Understanding

**Aurex-16++** is a deterministic 2D fantasy console written in Rust with SDL2:
- **Display**: 426×240 @ 60 FPS, RGB555 pixel format
- **Audio**: 48kHz stereo, 16-voice ASU-32 wavetable synthesis engine
- **Input**: Keyboard + game controller (SDL2 EventPump)
- **Cartridges**: Games live in `cartridges/<game_id>/` with `manifest.txt` + binary assets
- **Runtime**: Boot sequence (`PrimeAwakens`) → Library screen → Launch selected cartridge
- **Frame pipeline**: Clock begin → Clear FB → DMA begin → VM run → Scene update → PPU render → Overlay draw → DMA apply → Clock end

**Current problem**: The system is built for *humans* to play. It requires SDL2, a window, keyboard/gamepad input, and active human interaction.

**Goal**: Transform it into an **agent-operated game console** where:
- Humans specify what game to create via a web dashboard
- AI agents generate cartridges using the existing SDK contracts
- Agents "play" games programmatically (no human input)
- Every agent play session is **screen + audio recorded** to MP4
- Humans watch recordings on demand through the web dashboard

---

## Stage 0: Build & Validate (Foundation)

**Objective**: Get the existing codebase compiling and running in the current environment.

**Sub-tasks**:
1. Check Rust toolchain availability (`rustc`, `cargo`)
2. Install SDL2 development libraries if missing
3. Run `cargo check` to verify compilation
4. Run existing diagnostics (`--audio-diagnostics`, `--audit-cartridges`, `--replay-capture-smoke`)
5. Document any environment-specific fixes needed

**Deliverable**: Verified working build with all existing tests passing.

---

## Stage 1: Headless Runtime Mode (Core Transformation)

**Objective**: Decouple the runtime from SDL2 so agents can drive it programmatically.

**Current Architecture**: `main.rs` owns the SDL2 event loop — video, audio, input are all coupled to SDL2.

**New Architecture**: Split into three modes:
1. **`Interactive Mode`** (existing): SDL2 window, human input — unchanged
2. **`Headless Mode`** (new): No SDL2, programmatic input, framebuffer/audio exposed as raw data
3. **`Agent Mode`** (new): Headless + automatic recording + replay output

**Sub-tasks**:
1. **Extract `AurexCore`** from `main.rs` — separate the console simulation from SDL2 presentation
   - Create `AurexRuntime` struct that wraps `Aurex` + audio engine + frame pacer
   - Remove SDL2 dependency from the core simulation path
   - Expose `run_frame(input: InputState) -> FrameOutput { framebuffer, audio_block, events }`

2. **Create headless audio sink**
   - Instead of SDL2 AudioQueue, accumulate audio samples into a Vec<i16> buffer
   - Support both "real-time" (still use SDL2 queue if available) and "capture" mode (buffer everything)

3. **Create headless input API**
   - `AgentController` that takes `InputState` vectors or scripted input sequences
   - Support "playback from recording" ( deterministically replay a `ReplayCapture`)
   - Support "agent AI input" (external process sends input decisions frame-by-frame)

4. **Preserve existing interactive mode**
   - `main.rs` keeps SDL2 path as `--interactive` (default)
   - New `--headless` flag for agent mode

**Deliverable**: A headless Aurex runtime that can be driven frame-by-frame with programmatic input, producing raw framebuffer and audio data per frame, with zero SDL2 dependency in the core.

---

## Stage 2: Screen + Audio Recording (Capture Infrastructure)

**Objective**: Capture every frame's video and audio into a watchable MP4 file.

**Technical Constraints**:
- Framebuffer: 426×240 RGB555 → easily converted to RGB24/RGBA for encoding
- Audio: 48kHz stereo i16 — standard PCM format
- Frame rate: 60 FPS fixed
- Target: MP4 with H.264 video + AAC audio (or VP9 + Opus for faster encoding)

**Sub-tasks**:
1. **Video encoding pipeline**
   - Use `ffmpeg` CLI or `x264`/`x265` Rust bindings (e.g., `rav1e`, `x264-rs`, or just invoke ffmpeg)
   - Approach: Write raw RGB frames to a temp directory + raw PCM audio, then batch-encode with ffmpeg
   - Alternative: Use `image` crate to write frames + `ffmpeg` subprocess for muxing
   - *Simpler approach*: Stream raw RGBA frames to ffmpeg stdin via pipe, let ffmpeg do real-time encoding

2. **Audio capture integration**
   - Tap into the audio synthesis path before (or instead of) SDL2 AudioQueue
   - Accumulate each `render_block()` output into the recording buffer
   - Synchronize audio and video by frame number (60 FPS = exactly 800 samples/frame at 48kHz)

3. **Recording controller**
   - `SessionRecorder` struct:
     - `start(output_path: &str)` — spawn ffmpeg, open pipe
     - `on_frame(framebuffer: &[u16], audio: &[i16])` — write one video frame + corresponding audio samples
     - `stop()` — close pipe, finalize MP4
   - Configurable: resolution, bitrate, codec, audio quality

4. **Output format**
   - MP4 with H.264 video (yuv420p) + AAC audio (stereo 48kHz)
   - Filenames: `<game_id>_<timestamp>_<session_id>.mp4`
   - Stored in `recordings/` directory

**Deliverable**: A recording subsystem that produces MP4 files from any headless gameplay session, with synchronized audio and video.

---

## Stage 3: Agent Game Creation Pipeline (Automation)

**Objective**: Enable AI agents to autonomously create valid cartridges.

**Sub-tasks**:
1. **Cartridge generator service**
   - REST API endpoint: `POST /api/create-game` with JSON body:
     ```json
     {
       "game_id": "neon_circuit",
       "title": "NEON CIRCUIT",
       "genre_tag": "racer",
       "description": "A futuristic arcade racer...",
       "difficulty": "medium"
     }
     ```
   - The service uses an LLM (agent) to generate the cartridge following `docs/llm_prompt_template.md`
   - Enforces all validation rules: identity consistency, budget caps, deterministic language
   - Outputs: `cartridges/<game_id>/manifest.txt` + `.bin` assets

2. **Validation pipeline**
   - After generation, automatically run:
     - `cargo run -- --audit-cartridges --json`
     - `cargo run -- --analyze-cartridges --json`
   - Reject and retry if validation fails (with error feedback to agent)

3. **Asset generation helper**
   - Utility to convert images → tile/sprite binary format
   - Utility to convert audio → ASU-32 compatible format (or generate procedural audio)
   - Color palette optimizer (RGB555 quantization)

4. **Agent play script template**
   - Provide a template that agents use to "play" a game:
     ```rust
     let mut session = AgentSession::new("neon_circuit");
     session.launch_game();
     for frame in 0..3600 { // 60 seconds
         let input = agent_decide_input(frame, &session.last_framebuffer());
         session.run_frame(input);
     }
     let recording = session.finish_recording();
     ```

**Deliverable**: An automated pipeline where a human request → LLM generates cartridge → validation → ready to play.

---

## Stage 4: Web Dashboard (Human Interface)

**Objective**: A single-page web application where humans request games and watch recordings.

**Sub-tasks**:
1. **Backend API server** (Rust + Axum or lightweight HTTP server)
   - `POST /api/games/create` — submit game request, queue for agent processing
   - `GET /api/games` — list all cartridges and their status
   - `GET /api/games/:id/recordings` — list recordings for a game
   - `GET /api/recordings/:id` — stream MP4 file
   - `POST /api/games/:id/play` — trigger an agent play session
   - WebSocket `/ws/events` — real-time status updates

2. **Frontend** (vanilla HTML + CSS + JS, or React if using webapp swarm)
   - **Create Game** form: title, genre, description, difficulty
   - **Library** view: grid of all created cartridges with metadata
   - **Game Detail** view: info + "Watch Agent Play" button + list of recordings
   - **Player** view: HTML5 `<video>` player for recordings
   - **Live Monitor** view: WebSocket-connected view of an in-progress agent session (frame-by-frame stream)

3. **Session management**
   - SQLite or JSON file for persistence
   - Queue for agent tasks (create game, play game)
   - Status tracking: `pending` → `generating` → `validating` → `ready` → `playing` → `recorded`

4. **Real-time streaming** (optional enhancement)
   - For live agent sessions, stream frames as WebP/JPEG over WebSocket
   - Humans can "tune in" to an agent playing in real-time

**Deliverable**: A complete web dashboard served on a local port (e.g., 8080) where humans can create games, browse the library, and watch recordings.

---

## Stage 5: Agent Play Engine (Intelligent Gameplay)

**Objective**: Agents don't just send random input — they play with purpose and strategy.

**Sub-tasks**:
1. **Screen analysis bridge**
   - Every N frames, capture framebuffer as PNG and send to vision-capable LLM
   - LLM decides next input: `"left"`, `"right"`, `"up"`, `"accept"`, etc.
   - Or use a simpler rule-based approach for deterministic games

2. **Input strategy templates**
   - `Explorer`: Randomly explores all directions, presses buttons periodically
   - `Scorer`: Tries to maximize visible score/health indicators
   - `Survivor`: Prioritizes avoiding danger/enemies
   - `Speedrunner`: Aims to complete the game as fast as possible

3. **Session lifecycle**
   - `AgentSession::new(game_id)` → load cartridge, launch game
   - `run_for(duration)` or `run_until(condition)`
   - Auto-save recording on completion
   - Generate summary: frames played, score, events triggered, recording path

4. **Multi-agent support**
   - Multiple agents can play the same game simultaneously (different strategies)
   - Leaderboard of best scores/times per game per strategy

**Deliverable**: A pluggable agent input system with multiple strategy templates, producing recordings with metadata.

---

## Stage 6: Integration, Recording Console & Final Assembly

**Objective**: Tie everything together into a cohesive system.

**Sub-tasks**:
1. **Unified CLI entry point**
   ```
   aurex16pp --mode=interactive              # Human plays (original)
   aurex16pp --mode=headless --play=...      # Programmatic play
   aurex16pp --mode=agent --game=...         # Agent plays with recording
   aurex16pp --mode=server --port=8080       # Web dashboard + API
   aurex16pp --mode=create-game --spec=...   # One-shot cartridge creation
   ```

2. **Recording console** (built-in feature)
   - Every `--mode=agent` session auto-records to `recordings/`
   - `--recordings-dir=/path` to customize
   - `--record-format=mp4|webm|gif` for output format selection
   - `--max-recording-duration=300` to cap session length
   - Recording metadata JSON sidecar: game_id, strategy, frames, score, timestamp

3. **Self-contained packaging**
   - Docker support (all dependencies bundled)
   - `scripts/setup.sh` for environment prep
   - `scripts/start-dashboard.sh` to launch everything

4. **End-to-end testing**
   - Create a test cartridge using the agent pipeline
   - Run agent play session, verify recording outputs valid MP4
   - Load dashboard, verify video playback works
   - Test full flow: human request → agent creation → agent play → human watches

**Deliverable**: A fully integrated system where a human can go from "I want a space shooter" to watching a recorded agent play session in under 5 minutes.

---

## Technology Stack

| Component | Technology |
|-----------|-----------|
| Core runtime | Rust (existing Aurex codebase) |
| Headless decoupling | Refactor `main.rs` into `AurexRuntime` |
| Audio/Video encoding | `ffmpeg` CLI (pipe-based real-time encoding) |
| Web backend | `axum` (Rust) or `actix-web` |
| Frontend | React + Tailwind (using webapp-building-swarm skill) |
| Database | SQLite (simple, file-based) |
| Task queue | In-memory with background thread |
| Agent LLM | OpenAI/Claude API integration in backend |
| Container | Optional Docker for deployment |

---

## File Structure (New)

```
aurex16pp/
├── src/
│   ├── main.rs                    # Mode router (interactive/headless/agent/server)
│   ├── aurex/                     # Existing core (unchanged)
│   ├── runtime_core.rs            # NEW: Headless AurexRuntime
│   ├── recorder.rs                # NEW: SessionRecorder (ffmpeg pipe)
│   ├── agent_controller.rs          # NEW: Agent input strategies
│   ├── server/                    # NEW: Web dashboard backend
│   │   ├── mod.rs
│   │   ├── api.rs
│   │   ├── websocket.rs
│   │   └── db.rs
│   └── dashboard/                 # NEW: Frontend (React)
├── recordings/                    # NEW: Auto-created output directory
├── webapp/                        # NEW: Compiled frontend
├── scripts/
│   ├── setup.sh
│   └── start-dashboard.sh
└── Cargo.toml                     # Updated dependencies
```

---

## Risk Assessment & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| SDL2 removal breaks audio/video sync | High | Keep SDL2 as optional dependency; only disable in headless mode |
| ffmpeg not available in environment | High | Provide fallback: raw frame/audio dump + offline encode script |
| Cartridge generation produces invalid output | Medium | Auto-validation gate + retry loop with error feedback |
| Agent input is too random/boring | Medium | Multiple strategy templates + vision-based decision making |
| Web dashboard performance with large recordings | Low | Streaming MP4 via HTTP range requests |
| Neo-Geo comparison creates scope creep | Medium | Stay focused: this is an agent tool, not a human console upgrade |

---

## Success Criteria

1. ✅ `cargo build` succeeds with all original tests passing
2. ✅ `--mode=headless --game=neon_circuit --frames=1800` runs without SDL2 window and produces framebuffer output
3. ✅ `--mode=agent --game=neon_circuit` produces a valid MP4 recording with synchronized audio
4. ✅ Web dashboard loads at `http://localhost:8080` and lists games
5. ✅ Human can request "make a space shooter" through dashboard, agent generates cartridge, plays it, and human watches recording
6. ✅ Original interactive mode still works exactly as before (`--mode=interactive`)

---

## Execution Order

1. **Stage 0** → Verify build (can run immediately)
2. **Stage 1** → Headless runtime (biggest refactor, core dependency for everything)
3. **Stage 2** → Recording infrastructure (depends on Stage 1)
4. **Stage 4** → Web dashboard backend + frontend (can start backend while finishing frontend)
5. **Stage 3** → Agent creation pipeline (depends on Stages 1+2)
6. **Stage 5** → Agent play strategies (depends on Stages 1+2+3)
7. **Stage 6** → Integration + testing (depends on all prior)

---

*Plan version: 1.0*
*Date: 2026-05-06*
