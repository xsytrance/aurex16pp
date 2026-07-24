# Aurex-16++ Agent Console

A fully agent-operated game console. Humans request games through a web dashboard. AI agents autonomously play them — **live-streamed to the browser while they play** — and every session is screen+audio recorded to MP4 for on-demand viewing.

## Architecture

```
Human -> Web Dashboard (vanilla SPA) -> Axum API -> Agent Session -> Headless Aurex -> FFmpeg -> MP4
                                           |               |
                                           v               v
                                     Live frame map   (real-time paced, 60 FPS)
                                           |
                                           v
                                  Canvas live view (~20 FPS RGBA polling)
```

## Quick Start

### 1. Start the Dashboard Server

```bash
./scripts/start-dashboard.sh
```

The dashboard will be available at `http://localhost:8080` (and on your LAN IP).

### 2. Use the Dashboard

- **Create** a game: Enter title, genre, description → "Create Game"
- **Library**: See all games with status badges, LIVE indicators, and recording counts
- **Play Agent**: Choose a strategy (explorer/passive/aggressive/prime), duration, and audio profile — then **watch the agent play live** on a canvas stream
- **Watch**: MP4 playback with HTTP range/seek support; if the browser cannot decode video (VMs, automation builds), the page automatically falls back to a server-decoded **canvas replay**
- **Recordings**: Browse all recordings, watch any time; the library persists across restarts (`recordings/aurex.db.json`)

### 3. CLI Modes (for developers)

```bash
# Interactive mode (original SDL2 human-playable console)
cargo run --features interactive -- --interactive

# Headless mode (no window, programmatic input)
cargo run --no-default-features --features agent -- --headless --frames 60 --strategy explorer

# Agent mode (headless + auto-recording)
cargo run --no-default-features --features agent -- --agent --max-frames 120 --strategy aggressive --output-dir ./recordings

# Server mode (web dashboard + API)
cargo run --features server -- --server --port 8080 --recordings-dir ./recordings
```

## What Was Built

### Stage 1: Headless Runtime (`src/headless.rs`)
- Decoupled the Aurex core from SDL2
- `HeadlessAurex::run_frame(input) -> FrameOutput { framebuffer, audio, events }`
- Auto-advances boot sequence, auto-transitions to game mode
- Works with zero SDL2 dependencies

### Stage 2: Recording (`src/recorder.rs`)
- `SessionRecorder` uses named FIFOs + threaded writers
- Real-time H.264 + AAC MP4 encoding via FFmpeg
- Synchronized 426x240@60fps video + 48kHz stereo audio
- Non-blocking frame writes via mpsc channels

### Stage 3: Agent Session (`src/agent_session.rs`)
- `AgentSession` orchestrates runtime + recorder + strategy
- Pluggable `InputStrategy` trait:
  - **Explorer**: Deterministic random exploration
  - **Passive**: No input (observational)
  - **Aggressive**: Rapid alternating inputs
- `SessionResult` with recording path, frame count, strategy name

### Stage 4: Web Dashboard (`webapp/dist/index.html`)
- Single-file vanilla-JS SPA — no build step, served directly by the server
- Dark gaming console aesthetic (scanlines, neon accents)
- Pages: Library (+create game), Live view, Recordings, Player
- **Live view**: polls the live-frame endpoint and paints a pixel-perfect canvas at ~20 FPS with progress bar and LIVE indicators
- HTML5 `<video>` player for MP4 playback, with automatic canvas-replay fallback when the browser has no media pipeline

### Stage 5: API Server (`src/server/`)
- **Axum** async web server with CORS
- SQLite-backed persistence (in-memory for MVP)
- Endpoints: games CRUD, recordings stream, agent play trigger
- Static file serving with SPA fallback
- Background agent session execution via `tokio::spawn_blocking`

## API Reference

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/health` | GET | Server status |
| `/api/games/create` | POST | Create new game |
| `/api/games` | GET | List all games |
| `/api/games/:id` | GET | Game detail + recordings |
| `/api/games/:id/play` | POST | Start live agent session (`{strategy, max_frames, audio_profile?}`) |
| `/api/live` | GET | List in-flight sessions (agent play + canvas replays) |
| `/api/live/:session_id/frame` | GET | Latest 426x240 RGBA frame (raw bytes; `x-frame-number`, `x-max-frames` headers) |
| `/api/recordings` | GET | List all recordings |
| `/api/recordings/:id` | GET | Stream MP4 video (HTTP Range supported) |
| `/api/recordings/:id/info` | GET | Recording metadata |
| `/api/recordings/:id/replay` | POST | Server-decoded canvas replay of a recording (for browsers without video decode) |
| `/api/strategies` | GET | Available strategies (explorer, passive, aggressive, prime) |

## Build Features

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `interactive` | SDL2 window, human input | sdl2 |
| `agent` | Headless runtime + recording + strategies | rand |
| `server` | Axum web dashboard + API | axum, tokio, tower, serde, chrono |
| `full` | Everything | all of the above |

## Recording Output

Every agent session produces an MP4 file:
- **Video**: H.264, 426x240, 60 FPS, YUV420p
- **Audio**: AAC, 48kHz, stereo, 128kbps
- **Container**: MP4 with faststart for streaming
- **Location**: `./recordings/session_<game_id>_<timestamp>.mp4`

## File Structure

```
aurex16pp/
├── Cargo.toml                     # Feature-gated dependencies
├── src/
│   ├── main.rs                    # Mode router (interactive/headless/agent/server)
│   ├── headless.rs                # HeadlessAurex runtime
│   ├── recorder.rs                # SessionRecorder with ffmpeg
│   ├── agent_session.rs           # AgentSession + InputStrategy
│   ├── server/                    # Axum backend
│   │   ├── mod.rs
│   │   ├── api.rs
│   │   └── db.rs
│   ├── aurex/                     # Original core (unchanged)
│   └── cart/                      # Cartridge tools
├── webapp/dist/                   # Built React frontend
├── recordings/                    # MP4 output directory
├── scripts/
│   ├── setup.sh                   # Environment setup
│   └── start-dashboard.sh         # Start server
└── docs/                          # Original documentation
```

## Success Criteria — ALL MET

1. ✅ `cargo build --features full` succeeds
2. ✅ Original interactive mode (`--interactive`) works identically
3. ✅ Headless mode (`--headless --frames=1800`) runs without SDL2
4. ✅ Agent mode produces valid MP4 with synchronized audio+video
5. ✅ Web dashboard serves at `http://localhost:8080`
6. ✅ Full end-to-end: request → create → play → record → watch

## License

Same as original Aurex-16++ repository.
