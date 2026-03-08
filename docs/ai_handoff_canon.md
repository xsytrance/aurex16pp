# AUREX-16++ AI HANDOFF — CANON

Purpose:
This document defines the current canonical state of the Aurex-16++ hardware.
It must remain concise and reconstructable.
No historical narrative belongs here.

---

# 1. Core Hardware Specification (Locked)

Resolution: 426×240 (16:9)  
Frame Rate: 60 FPS deterministic  
Color Format: RGB555 (15-bit)  
On-Screen Colors: 256 max

CPU Cap: 200,000 ops per frame (hard)

Memory:

- 512 KB WRAM (locked)
- 1 MB VRAM (partitioned)
- 256 KB Audio RAM

DMA Limits:

- 4 commands per frame
- 64 KB VRAM upload per frame max
- 16 KB audio upload per frame max
- Immediate reject if exceeded

No floating point in core VM or PPU.

All compositing is integer-only.

---

# 2. VRAM Layout (Canonical)

VRAM_TOTAL_BYTES must equal 1 MB.

Regions:

A — BG Pattern Memory  
B — BG Tilemaps  
C — Sprite Pattern Memory  
E — Mode7 Texture  
H — Palettes  
Reserved region locked

This layout must not be restructured.

---

# 3. Rendering Pipeline Order (Locked)

Per scanline:

1. BG0
2. BG1 (window-masked)
3. Sprites
4. Additive blending during sprite pass

Transparent BG pixels do not block sprites.

High-priority BG blocks low-priority sprites.

High-priority sprite always wins.

---

# 4. Background Layers

BG0:

- 64×64 tilemap
- 8×8 tiles
- 4bpp packed (32 bytes per tile)
- Scroll registers
- Per-tile priority bit

BG1:

- Same format
- Independent scroll
- Window mask capable

Per-scanline scroll tables exist:

- bg0_scroll_x_line[FB_H]
- bg1_scroll_x_line[FB_H]

---

# 5. Sprite System

Max sprites total: 256  
Max per scanline: 8 (overflow latched)

Sprite attributes:

- x
- y
- tile_index
- palette
- priority
- visible
- size_16
- hflip
- vflip
- blend

8×8 and 16×16 supported.

16×16 layout:

[ base base+1 ]
[ base+2 base+3 ]

Flip applies to full composite.

Color index 0 is transparent.

Blending modes:

- Normal
- Additive (RGB555 clamp)

---

# 6. Register System

PPU supports:

Enum-based:

- write_reg(PpuReg, value)
- read_reg(PpuReg)

Address-based:

- write_addr(addr, value)
- read_addr(addr)

All frame mutation must occur through register interface.

No direct field mutation outside PPU.

- VRAM DMA writes are gated by VBlank.
  - If not in VBlank, DMA apply() performs no writes.
  - Deterministic silent rejection.
  - No IRQ or stall behavior yet.

  ### PPU Phase 6 — VBlank Simulation (Foundational / Pre-Timing)

  Status: Implemented (foundational latch only)

Rules:

- PPU is passive: it does not mutate bus registers during rendering.
- VBlank is a deterministic internal state latch (not cycle-accurate yet):
  - `vblank = false` at start of `render_frame`
  - `vblank = true` after the last scanline is rendered
- No IRQ timing, no mid-frame toggles, no per-scanline status yet.

Purpose:

- Provides a clean deterministic hook for DMA gating (VRAM writes only during VBlank).
- Sets the stage for later timing granularity without redesigning the architecture.

---

# 7. Determinism Guarantees

- No float usage
- No hidden mutation
- No frame-order dependency
- No dynamic reallocation during scanline
- No deferred DMA

---

# 8. File Responsibility Lock

mod.rs:

- System orchestration only
- Must not mutate PPU internals directly

ppu.rs:

- Owns rendering pipeline
- Owns register interface
- Owns scanline logic

oam.rs:

- Owns sprite storage only

vram.rs:

- Owns VRAM layout only

Cross-boundary mutation is forbidden.

---

# 9. Register Bus Discipline (Locked)

PPU state mutation must occur exclusively through the register interface:

- write_reg(PpuReg, value)
- read_reg(PpuReg)
- write_addr(addr, value)
- read_addr(addr)

Direct mutation of PPU fields outside `ppu.rs` is forbidden.

Frame logic must simulate CPU-style register writes rather than directly modifying internal state.

Register mutation hierarchy:

Frame / Cartridge Logic  
→ Address Bus (write_addr)  
→ write_reg  
→ Internal PPU fields

This layering must be preserved for:

- Future CPU bus integration
- Cartridge system control
- Deterministic replay
- Save-state integrity
- LLM-targetable SDK surface

No future system may bypass this structure.

---

## PPU Phase 6 — VBlank State Simulation (Foundational)

The PPU now exposes a hardware-style `vblank` flag.

### Behavior

- `vblank = false` at start of `render_frame`
- `vblank = true` after all scanlines are rendered
- No IRQ or timing granularity yet
- No behavioral change to rendering

### Purpose

This establishes:

- Future VBlank-safe DMA gating
- Register update timing discipline
- Deterministic save-state completeness
- IRQ simulation foundation

### Scope

- No interrupts implemented
- No cycle timing
- No frame timing redesign
- No change to frame rate or determinism

## Phase 6.5 — Boot Validation & Sprite Tile Format Confirmation

- Sprite tile format confirmed:
  - 4bpp
  - 8×8
  - 32 bytes per tile
  - Linear nibble-packed (NOT planar)
  - 4 bytes per row
  - High nibble = left pixel
  - Low nibble = right pixel

- 16×16 sprites composed of 2×2 8×8 tiles
  - Base tile index = top-left tile
  - Tile index offset applied per quadrant

- DMA sprite tile upload validated via PrimeIgnition boot module.
- VBlank gating confirmed functional.
- Palette memory currently uninitialized (visual artifacts expected).
- No direct VRAM mutation used — DMA-only writes preserved.

Architecture remains stable.

END OF CANON


---

# Library Profile Canon (Current)

- Runtime scene: Boot → Library.
- Library entries are represented by profile data:
  - title string
  - track id
  - color theme
  - icon kind
- Selection emits `AudioCue::SelectTrack`.
- Audio engine uses track id mapping for deterministic per-title song playback.


- Runtime event boundary active:
  - Simulation emits typed runtime events
  - Host loop drains and dispatches side effects




## Library Feedback Canon (2026-03-08 01:08:00Z)

- Launch-intent UX now includes a deterministic audio stinger cue:
  - `AudioCue::LaunchRequest`
- Library scene visual feedback includes deterministic footer pulse + meter animation.
- Host interpretation of non-audio runtime events should prefer runtime diagnostics collection over ad-hoc per-loop matching.



## Launch Intent Lifecycle Canon (2026-03-08 01:37:00Z)

- Launch selection intent is reversible before host-side cartridge boot is attached.
- Runtime event set for library intent now includes:
  - `RuntimeEvent::TitleLaunchRequested(LaunchDescriptor)`
  - `RuntimeEvent::TitleLaunchCanceled`
- Audio cue set now includes explicit cancel intent (`AudioCue::Cancel`).



## Launch Stage Canon (2026-03-08 02:02:00Z)

- Launch intent now has an explicit runtime stage domain:
  - `LaunchStage::Idle`
  - `LaunchStage::Pending(LaunchDescriptor)`
  - `LaunchStage::Validating(LaunchDescriptor)`
  - `LaunchStage::Ready(LaunchDescriptor)`
  - `LaunchStage::Rejected(LaunchValidationError)`
- Stage transitions emit `RuntimeEvent::LaunchStageChanged(LaunchStage)`.
- Library HUD presents pending stage visually (`PENDING` marker + boosted meter bars).



## LLM SDK Canon (2026-03-08 02:28:00Z)

- Cartridge generation is prompt-structured, not free-form.
- Required authoring references:
  - `docs/llm_sdk_guide.md`
  - `docs/llm_prompt_template.md`
- Launch descriptor identity includes `cartridge_id` to bridge library selection and cartridge asset folders.



## Launch Validation Canon (2026-03-08 02:56:00Z)

- Launch descriptors are validated before entering pending launch stage.
- Invalid descriptors emit `RuntimeEvent::TitleLaunchRejected(LaunchValidationError)`.
- Current validation includes strict cartridge ID format enforcement (`[a-z0-9_]+`).



## Launch Ready Canon (2026-03-08 03:22:00Z)

- Launch flow now includes deterministic validating and ready stages.
- `TitleLaunchReady(LaunchDescriptor)` is the runtime signal reserved for future cartridge boot handoff.

## Runtime Handoff Contract (Current)

Scene lifecycle contract:
- Boot scene remains deterministic and non-interruptible until flow gate opens.
- Start gate transition is explicit (`AwaitStart`).
- Entering library emits a scene transition event.

Event contract:
- `RuntimeEvent::Audio(AudioCue)` for soundtrack/SFX intent.
- `RuntimeEvent::SceneChanged(SceneId)` for lifecycle telemetry.
- `RuntimeEvent::TitleLaunchRequested(LaunchDescriptor)` for explicit library launch intent.
- `RuntimeEvent::TitleLaunchCanceled` for launch clear intent.
- `RuntimeEvent::LaunchStageChanged(LaunchStage)` for lifecycle stage telemetry.

Host contract:
- Drain runtime events every frame after `run_frame`.
- Route side effects in host/runtime dispatch layer, not inside scene simulation logic.
