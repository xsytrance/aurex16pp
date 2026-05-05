# Aurex-16++ Agent Playground — SPEC.md

## 1. Project Identity

Transform the existing Aurex-16++ fantasy console from a human-playable SDL2 application into an **agent-operated game console** where:
- AI agents autonomously create and play games
- Every session is screen+audio recorded to MP4
- Humans interact through a web dashboard (request games, watch recordings)

## 2. Core Architecture

### 2.1 Mode System

The binary supports three mutually exclusive modes:

| Mode | Flag | Purpose |
|------|------|---------|
| Interactive | `--interactive` (default) | Original SDL2 human-playable mode |
| Headless | `--headless` | Programmatic drive, no SDL2 window |
| Agent | `--agent --game=<id>` | Headless + auto-recording + agent input |
| Server | `--server --port=8080` | Web dashboard + API server |

### 2.2 Module Structure

```
src/
  main.rs                    # Mode router, CLI parsing
  aurex/                     # EXISTING core (DO NOT MODIFY unless necessary)
    mod.rs, ppu/, runtime/, audio/, ...
  headless.rs                # NEW: HeadlessAurex runtime wrapper
  recorder.rs                # NEW: SessionRecorder (ffmpeg pipe)
  agent_session.rs           # NEW: AgentSession lifecycle manager
  server/mod.rs              # NEW: Axum HTTP server + WebSocket
  server/api.rs              # NEW: REST API endpoints
  server/db.rs               # NEW: SQLite persistence
  dashboard/                 # NEW: React frontend (built separately)
```

## 3. Headless Runtime Specification

### 3.1 `HeadlessAurex` (src/headless.rs)

A wrapper around the existing `Aurex` + `AudioEngine` that removes all SDL2 dependencies.

```rust
pub struct HeadlessAurex {
    system: Aurex,
    audio: AudioEngine,
    flow: FlowController,
    runtime_events: Vec<RuntimeEvent>,
    audio_buffer: Vec<i16>,
    frame_count: u64,
}

pub struct FrameOutput {
    pub framebuffer: Vec<u16>,        // 426x240 RGB555 pixels
    pub audio_samples: Vec<i16>,      // 800 samples (48kHz / 60fps, stereo interleaved)
    pub events: Vec<RuntimeEvent>,
    pub phase: FlowPhase,
    pub frame_number: u64,
}
```

**Constructor:**
```rust
impl HeadlessAurex {
    pub fn new(audio_profile: MixProfile) -> Self;
}
```

**Per-frame tick:**
```rust
pub fn run_frame(&mut self, input: InputState) -> FrameOutput;
```

Behavior per call:
1. If `flow.phase() == FlowPhase::AwaitStart` and `input.accept == true`, call `flow.tick(true)` then `system.start_game()`
2. If `flow.phase() == FlowPhase::Boot`, auto-advance boot (no start required for headless)
3. Determine `AudioMode` from flow phase
4. Call `system.run_frame(input, boot_beat_step)`
5. Drain runtime events
6. Render 800 audio samples (48kHz / 60 FPS = 800 samples/frame) into `audio_buffer`
7. Return `FrameOutput` with cloned framebuffer, audio, events

**Boot auto-advance for headless:**
- After 300 boot frames, automatically transition to `AwaitStart`
- On next frame with any input (or default), transition to `Game`

### 3.2 `AudioCapture` (inline in headless.rs)

Instead of SDL2 AudioQueue, the headless runtime captures audio into a Vec<i16>:

```rust
const SAMPLES_PER_FRAME: usize = 800; // 48000 / 60

// In run_frame:
let mut block = [0i16; SAMPLES_PER_FRAME * 2]; // stereo interleaved
self.audio.render_block(audio_mode, &mut block);
// block goes into FrameOutput.audio_samples
```

## 4. Recording Specification

### 4.1 `SessionRecorder` (src/recorder.rs)

Manages ffmpeg subprocess for real-time MP4 encoding.

```rust
pub struct SessionRecorder {
    ffmpeg_stdin: Option<std::process::ChildStdin>,
    ffmpeg_child: Option<std::process::Child>,
    width: u32,
    height: u32,
    frame_count: u64,
    output_path: String,
}

pub struct RecorderConfig {
    pub width: u32,           // 426
    pub height: u32,          // 240
    pub fps: u32,             // 60
    pub sample_rate: u32,     // 48000
    pub output_path: String,
    pub video_codec: String,  // "libx264"
    pub audio_codec: String,  // "aac"
    pub crf: u8,              // 23 (quality)
    pub preset: String,       // "ultrafast" for speed
}
```

**FFmpeg command:**
```bash
ffmpeg -y -f rawvideo -pix_fmt rgb24 -s {width}x{height} -r {fps} \
  -thread_queue_size 512 -i - \
  -f s16le -ar {sample_rate} -ac 2 -thread_queue_size 512 -i - \
  -c:v libx264 -pix_fmt yuv420p -preset ultrafast -crf 23 \
  -c:a aac -b:a 128k \
  -movflags +faststart \
  {output_path}
```

**Per-frame write:**
- Convert RGB555 framebuffer → RGB24 bytes (3 bytes per pixel)
- Write video frame to ffmpeg stdin pipe A
- Write interleaved i16 audio samples to ffmpeg stdin pipe B
- Use two threads or async channels to avoid blocking

### 4.2 Dual-pipe approach

Since ffmpeg needs two inputs (video pipe + audio pipe), use named pipes (FIFOs):

```
/tmp/aurex_recording_<pid>/
  video_fifo
  audio_fifo
  output.mp4
```

FFmpeg args:
```bash
ffmpeg -y -f rawvideo -pix_fmt rgb24 -s 426x240 -r 60 -i video_fifo \
       -f s16le -ar 48000 -ac 2 -i audio_fifo \
       -c:v libx264 -pix_fmt yuv420p -preset ultrafast -crf 23 \
       -c:a aac -b:a 128k -shortest output.mp4
```

**Start sequence:**
1. Create temp directory
2. Create named pipes with `nix::mkfifo` or `std::process::Command("mkfifo")`
3. Spawn ffmpeg reading from pipes
4. Spawn writer threads that open pipes and block until ffmpeg connects

**Per frame:**
1. Convert framebuffer RGB555 → RGB24
2. Write 426*240*3 = 306,180 bytes to video pipe
3. Write 800*2*2 = 3,200 bytes (800 stereo frames * 2 channels * 2 bytes) to audio pipe

**Stop sequence:**
1. Close video pipe
2. Close audio pipe
3. Wait for ffmpeg to finish (waitpid)
4. Return output path

## 5. Agent Session Specification

### 5.1 `AgentSession` (src/agent_session.rs)

Orchestrates a complete agent gameplay session with recording.

```rust
pub struct AgentSession {
    runtime: HeadlessAurex,
    recorder: Option<SessionRecorder>,
    strategy: Box<dyn InputStrategy>,
    game_id: String,
    recording_path: Option<String>,
    metadata: SessionMetadata,
}

pub struct SessionMetadata {
    pub game_id: String,
    pub strategy_name: String,
    pub start_time: std::time::SystemTime,
    pub end_time: Option<std::time::SystemTime>,
    pub frames_played: u64,
    pub events_triggered: Vec<String>,
}
```

**Lifecycle:**
```rust
impl AgentSession {
    pub fn new(game_id: &str, strategy: Box<dyn InputStrategy>, record: bool) -> Self;
    pub fn run_for_frames(&mut self, frames: u64);
    pub fn run_until<F: Fn(&FrameOutput) -> bool>(&mut self, condition: F);
    pub fn finish(self) -> SessionResult;
}
```

### 5.2 `InputStrategy` trait

```rust
pub trait InputStrategy: Send {
    fn decide_input(&mut self, frame_number: u64, framebuffer: &[u16]) -> InputState;
    fn name(&self) -> &'static str;
}
```

**Built-in strategies:**
- `ExplorerStrategy`: Random directional exploration with periodic button presses
- `SurvivorStrategy`: Prioritizes avoiding danger (left/right only, rarely up)
- `AggressiveStrategy`: Frequent accept presses, rapid movement
- `PassiveStrategy`: Minimal input, mostly idle

### 5.3 Frame loop

```rust
for frame in 0..max_frames {
    let input = strategy.decide_input(frame, &last_framebuffer);
    let output = runtime.run_frame(input);
    
    if let Some(ref mut rec) = recorder {
        rec.write_frame(&output.framebuffer, &output.audio_samples);
    }
    
    if condition_met(&output) {
        break;
    }
}
```

## 6. Web Dashboard Specification (Stage 4)

### 6.1 Backend (Axum)

**REST API:**
```
POST /api/games/create          -> { game_id, title, genre, description }
GET  /api/games                 -> [ { game_id, title, status, created_at } ]
GET  /api/games/:id             -> { game_id, title, status, cartridges, recordings }
POST /api/games/:id/play        -> { strategy, max_frames } -> { session_id }
GET  /api/recordings            -> [ { id, game_id, path, duration, created_at } ]
GET  /api/recordings/:id/stream  -> MP4 stream (Range request supported)
GET  /api/recordings/:id/download -> MP4 download
```

**WebSocket `/ws/live`:**
- Real-time frame streaming for active agent sessions
- Binary WebP/JPEG frame data
- JSON status messages

### 6.2 Frontend (React + Tailwind)

**Pages:**
1. **Home** — Hero + "Create Game" CTA
2. **Create Game** — Form with title, genre, description, difficulty
3. **Library** — Grid of all games with thumbnails, status badges
4. **Game Detail** — Info + recordings list + "Watch Agent Play" button
5. **Player** — HTML5 video player for recordings
6. **Live** — Real-time frame stream viewer

## 7. Cartridge Generation Pipeline (Stage 3)

### 7.1 LLM Prompt Template

Use the existing `docs/llm_prompt_template.md` contract. The backend:
1. Accepts human game request (title, genre, description)
2. Builds structured prompt from template
3. Sends to LLM API
4. Parses response into cartridge files
5. Validates with `cargo run -- --audit-cartridges`
6. Stores in `cartridges/<game_id>/`

### 7.2 Asset Generation Helpers

- `image_to_tiles(path) -> Vec<u8>`: Convert PNG/JPG to 4bpp tile data
- `palette_optimizer(colors) -> Vec<u16>`: RGB888 → RGB555 palette
- `generate_wavetable(spec) -> Vec<i16>`: Procedural audio assets

## 8. Integration Points

### 8.1 Unified CLI

```bash
# Interactive (original behavior, SDL2 window)
./aurex16pp --interactive --audio-profile=arcade

# Headless (programmatic, no recording)
./aurex16pp --headless --game=neon_circuit --frames=1800 --input-script=explore.json

# Agent mode (headless + recording + auto-input)
./aurex16pp --agent --game=neon_circuit --strategy=explorer --max-frames=3600 --output-dir=recordings/

# Server mode (web dashboard)
./aurex16pp --server --port=8080 --recordings-dir=recordings/
```

### 8.2 Environment Variables

```
AUREX_RECORDINGS_DIR=/var/recordings
AUREX_DATA_DIR=/var/lib/aurex
AUREX_LLM_API_KEY=sk-...
AUREX_LLM_MODEL=gpt-4o
AUREX_SKIP_AUDIT_LINK=1
```

## 9. Data Model

### 9.1 SQLite Schema

```sql
CREATE TABLE games (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    genre TEXT,
    description TEXT,
    status TEXT DEFAULT 'pending',
    created_at INTEGER,
    updated_at INTEGER
);

CREATE TABLE recordings (
    id TEXT PRIMARY KEY,
    game_id TEXT REFERENCES games(id),
    path TEXT NOT NULL,
    strategy TEXT,
    frames INTEGER,
    duration_secs REAL,
    file_size_bytes INTEGER,
    created_at INTEGER
);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    game_id TEXT REFERENCES games(id),
    status TEXT DEFAULT 'running',
    current_frame INTEGER,
    strategy TEXT,
    started_at INTEGER,
    ended_at INTEGER
);
```

## 10. Build & Deployment

### 10.1 Dependencies (Cargo.toml additions)

```toml
[dependencies]
# Existing
anyhow = "1.0"
sdl2 = { version = "0.36", optional = true }

# New
axum = { version = "0.7", optional = true }
tokio = { version = "1", features = ["full"], optional = true }
tower = { version = "0.4", optional = true }
tower-http = { version = "0.5", features = ["fs", "cors"], optional = true }
serde = { version = "1.0", features = ["derive"], optional = true }
serde_json = { version = "1.0", optional = true }
rusqlite = { version = "0.32", optional = true }
rand = { version = "0.8", optional = true }
reqwest = { version = "0.12", features = ["json"], optional = true }
chrono = { version = "0.4", optional = true }

[features]
default = ["sdl2"]
interactive = ["sdl2"]
server = ["axum", "tokio", "tower", "tower-http", "serde", "serde_json", "rusqlite", "chrono"]
agent = ["rand"]
full = ["interactive", "server", "agent"]
```

### 10.2 Build Script

```bash
#!/bin/bash
# scripts/setup.sh
. "$HOME/.cargo/env"
export CARGO_TARGET_DIR=/tmp/aurex-target
export RUSTFLAGS="-C linker=gcc -L /tmp/sdl2-link"

# Create SDL2 symlink if missing
mkdir -p /tmp/sdl2-link
ln -sf /usr/lib/x86_64-linux-gnu/libSDL2-2.0.so.0 /tmp/sdl2-link/libSDL2.so 2>/dev/null || true

# Build
cd /tmp/aurex-build/aurex16pp
cargo build --release --features full
```

## 11. Success Criteria

1. `cargo check --features full` passes
2. `--interactive` mode works identically to original
3. `--headless --game=... --frames=N` runs without SDL2 and produces framebuffer output
4. `--agent --game=...` produces a valid MP4 with synchronized audio
5. Web dashboard serves at port 8080
6. Full end-to-end: request → generate → play → record → watch
