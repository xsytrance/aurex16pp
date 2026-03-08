# Test / Validation Log

_Last updated: 2026-03-08._

## Latest validation pass (runtime AV stage 3 follow-through)

### Commands executed

1. `cargo fmt`
   - Result: PASS
   - Output: command completed with exit code 0 and no diff/output.

2. `cargo check`
   - Result: PASS
   - Output excerpt:
     - `Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.65s`
     - warning-only run (dead-code/unused items), no errors.

3. `cargo check --tests`
   - Result: PASS
   - Output excerpt:
     - `Checking aurex16pp v0.1.0 (/workspace/aurex16pp)`
     - `Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.70s`

4. `AUREX_SKIP_AUDIT_LINK=1 scripts/preflight.sh`
   - Result: PASS (link-limited mode)
   - Output excerpt:
     - `[preflight] formatting check`
     - `[preflight] compile check`
     - `[preflight] skipping cartridge audit run (AUREX_SKIP_AUDIT_LINK=1)`

5. `cargo test -q`
   - Result: ENV-LIMITED
   - Output excerpt:
     - linker failure: `rust-lld: error: unable to find library -lSDL2`
   - Rationale: tests build to a binary target that links `sdl2`; container lacks system SDL2.

## Interpreting results

- Compile/type validation for runtime, new CLI paths, and added regression test code is clean.
- Full execution of binary-linked tests/CLI runtime commands (`cargo run`, `cargo test`) remains blocked until SDL2 system libs are present.

## Recommended full validation on SDL2-enabled host

1. `scripts/preflight.sh`
2. `cargo test`
3. `cargo run -- --docs-sync-check`
4. `cargo run -- --generate-runtime-baseline --frames 48000 --out artifacts/runtime_audio_diag_baseline.json`
5. `cargo run -- --audio-diagnostics --boot --frames 48000 --json`
6. `cargo run -- --replay-capture-smoke`
