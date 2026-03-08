# Test / Validation Log

_Last updated: 2026-03-08._

## Latest validation pass (deterministic mix profiles + duplicate/warning hygiene continuation)

### Commands executed

1. `cargo fmt -- --check && cargo check --all-targets`
   - Result: PASS
   - Notes: compile succeeds; warning backlog is now intentionally suppressed in unfinished modules via module-level `#![allow(dead_code)]`.

2. `AUREX_SKIP_AUDIT_LINK=1 scripts/preflight.sh`
   - Result: PASS
   - Output excerpt:
     - `[preflight] docs-sync check (shell fallback)`
     - `[preflight] docs-sync check passed (shell fallback)`

3. `cargo check --tests`
   - Result: PASS

4. `cargo test -q`
   - Result: ENV-LIMITED
   - Output excerpt:
     - `rust-lld: error: unable to find library -lSDL2`

## Interpretation

- Mix profile architecture compiles cleanly and is available to diagnostics/baseline/runtime init paths.
- Full test execution remains blocked until system SDL2 is available.
