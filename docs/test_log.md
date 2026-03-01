
## Phase 4
- Verified framebuffer alloc + per-frame clear compiles clean

## Phase 3.6
- Removed audio DMA references from controller
- Fixed compile error after DMA signature change
- Verified WRAM → VRAM copy path builds clean

## Phase 3.5
- Removed DMA smoke test from core loop
- Fixed stack overflow caused by large array allocations
- Verified deterministic frame pacing