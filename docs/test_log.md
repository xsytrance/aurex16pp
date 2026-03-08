# Test / Validation Log

_Last updated: 2026-03-08._

## Latest validation pass (beat-step API compatibility + envelope smoothing field)

### Commands executed

1. `cargo fmt && cargo check --all-targets && AUREX_SKIP_AUDIT_LINK=1 scripts/preflight.sh && cargo check --tests`
   - Result: PASS
   - Notes: compile/preflight checks pass with docs-sync shell fallback active in link-limited mode.

2. `cargo test -q`
   - Result: ENV-LIMITED
   - Output excerpt:
     - `rust-lld: error: unable to find library -lSDL2`

## Interpretation

- Integration signature drift is resolved in this branch for `run_frame(..., boot_beat_step)` and boot overlay API alignment.
- Voice envelope smoothing state compiles cleanly and no missing-field errors remain.
- Full binary-linked test execution remains blocked without system SDL2.
