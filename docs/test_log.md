# Test / Validation Log

_Last updated: 2026-03-08._

## Latest validation pass (audio diagnostics depth + preflight docs-sync fallback)

### Commands executed

1. `cargo fmt -- --check && cargo check --all-targets`
   - Result: PASS
   - Notes: format and compile checks completed successfully; warnings only.

2. `AUREX_SKIP_AUDIT_LINK=1 scripts/preflight.sh`
   - Result: PASS
   - Output excerpt:
     - `[preflight] docs-sync check (shell fallback)`
     - `[preflight] docs-sync check passed (shell fallback)`
     - `[preflight] skipping cartridge audit run (AUREX_SKIP_AUDIT_LINK=1)`

3. `cargo check --tests`
   - Result: PASS
   - Notes: test targets compile in this container; warnings only.

4. `cargo test -q`
   - Result: ENV-LIMITED
   - Output excerpt:
     - `rust-lld: error: unable to find library -lSDL2`
   - Rationale: binary-linked test execution remains blocked in this container without system SDL2.

## Interpretation

- New diagnostics fields compile cleanly and are available in JSON/human output paths.
- Preflight docs-sync now remains enforceable even when SDL2 link limitations require audit/run skipping.
