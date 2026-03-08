# Test / Validation Log

_Last updated: 2026-03-08._

## Latest validation pass (baseline profile metadata fast follow)

### Commands executed

1. `cargo check --all-targets`
   - Result: PASS
   - Notes: runtime baseline JSON generation compiles after adding top-level `audio_profile` metadata.

2. `cargo check --tests`
   - Result: PASS
   - Notes: test targets continue to compile with diagnostics telemetry fields.

3. `cargo test -q`
   - Result: ENV-LIMITED
   - Output excerpt:
     - `rust-lld: error: unable to find library -lSDL2`

## Interpretation

- Runtime baseline payload now includes top-level `audio_profile` metadata for easier host-side grouping.
- Existing deterministic baseline and replay smoke content remain available in the same payload.
- Full binary-linked test execution remains blocked without system SDL2.
