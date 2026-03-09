# Test / Validation Log

_Last updated: 2026-03-08._

## Latest validation pass (boot/library sound timing fix)

### Commands executed

1. `cargo fmt`
   - Result: PASS

2. `cargo check --all-targets`
   - Result: PASS

3. `cargo check --tests`
   - Result: PASS

4. `cargo test -q`
   - Result: ENV-LIMITED
   - Output excerpt:
     - `rust-lld: error: unable to find library -lSDL2`

## Interpretation

- Boot sequencing now uses dedicated boot tick timing (`BOOT_TICK_HZ=8`).
- Library sequencing now uses BPM-derived tick intervals and immediate-start behavior on `PlayTrack`.
- Main-loop ordering now renders audio after dispatching runtime events, removing one-frame track-switch lag.
- Full binary-linked test execution remains blocked without system SDL2.
