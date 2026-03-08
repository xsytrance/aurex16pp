# Test / Validation Log

_Last updated: 2026-03-08._

## Latest validation pass (profile-aware diagnostics delta assertions)

### Commands executed

1. `cargo check --all-targets`
   - Result: PASS
   - Notes: runtime and CLI compile clean after profile regression test additions.

2. `cargo check --tests`
   - Result: PASS
   - Notes: test targets compile including new profile ordering assertions.

3. `cargo test -q`
   - Result: ENV-LIMITED
   - Output excerpt:
     - `rust-lld: error: unable to find library -lSDL2`

## Interpretation

- Regression coverage now explicitly validates deterministic mix-profile loudness ordering (`soft <= default <= arcade`) for diagnostics outputs.
- Deterministic baseline expectations still include zero clipping in this path.
- Full binary-linked test execution remains blocked without system SDL2.
