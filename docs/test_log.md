# Test / Validation Log

_Last updated: 2026-03-08._

## Latest validation pass (handoff doc sync + boot audio stabilization context)

### Commands executed

1. `cargo fmt && cargo check -q`
   - Result: PASS
   - Notes: Compiles cleanly in current container; non-fatal dead-code warnings remain.

2. `cargo test -q`
   - Result: ENV-LIMITED
   - Notes: Fails at link stage in this container because system `SDL2` library is unavailable (`-lSDL2` not found).

## Interpreting results

- `cargo check` success confirms code-level parse/type/borrow correctness for current source.
- `cargo test`/`cargo run` outcomes in this container are not valid indicators of runtime correctness until SDL2 system dependency is present.

## Recommended local/full validation (developer machine with SDL2)

Run in order:

1. `cargo fmt -- --check`
2. `cargo check`
3. `cargo test`
4. `cargo run -- --audio-diagnostics --boot --frames 48000 --json`
5. `cargo run -- --audit-cartridges --json`
6. `cargo run -- --analyze-cartridges --json`
7. `cargo run -- --replay-capture-smoke`

Capture outputs and attach to next handoff update.
