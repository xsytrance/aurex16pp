# Test / Validation Log

_Last updated: 2026-03-08._

## Latest validation pass (baseline JSON shape regression guard)

### Commands executed

1. `cargo check --all-targets`
   - Result: PASS
   - Notes: runtime baseline path compiles after helper extraction (`runtime_baseline_json`).

2. `cargo check --tests`
   - Result: PASS
   - Notes: new JSON shape unit test compiles in `main.rs`.

3. `cargo test -q`
   - Result: ENV-LIMITED
   - Output excerpt:
     - `rust-lld: error: unable to find library -lSDL2`

## Interpretation

- Baseline payload assembly is now centralized and testable.
- Regression coverage now guards top-level `audio_profile` and nested diagnostics `boot_beat_step` field presence.
- Full binary-linked test execution remains blocked without system SDL2.
