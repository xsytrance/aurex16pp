# AUREX-16++ LLM SDK GUIDE (v0.1)

## Purpose
This guide defines the deterministic prompt structure LLMs must follow to generate Aurex cartridges.

Aurex is not free-form: cartridge outputs must conform to hardware caps and a strict manifest schema.

## Prompt Contract (Required Sections)
Every generation prompt for a cartridge should include these sections in order:

1. `GAME_ID` — lowercase snake_case identifier (example: `neon_circuit`)
2. `TITLE` — display title
3. `GENRE_TAG` — short controlled tag (platformer, racer, shooter, puzzle, tactics)
4. `LOOP_SPEC` — deterministic frame loop summary
5. `INPUT_MAP` — fixed actions and buttons
6. `ASSET_BUDGET` — VRAM/audio usage goals under hard caps
7. `DMA_PLAN` — upload regions/chunk policy
8. `AUDIO_PLAN` — track IDs/voice expectations
9. `FAILSAFE_RULES` — rejection rules when budgets are exceeded
10. `OUTPUT_FILES` — exact files to generate

Any missing section = invalid prompt.

## Deterministic Output Rules
- No floating-point gameplay state.
- No runtime randomness without deterministic seed progression.
- No implicit budget expansion.
- No direct PPU internals mutation from cartridge logic.
- DMA must respect region and per-frame caps.

## Cartridge Descriptor Mapping
Library launch now emits a descriptor:
- `title`
- `cartridge_id`

LLM-created cartridge assets should be placed under:
- `cartridges/<cartridge_id>/`
- `cartridges/<cartridge_id>/manifest.txt`

## Manifest Baseline
Current loader accepts:
- `name=<display name>`
- `upload=<Region,dst_offset,file>`

Future SDK revisions will add validated metadata fields, but this baseline is required now.

## Example Prompt Skeleton
See `docs/llm_prompt_template.md`.


## Cartridge ID Format Rule
- `cartridge_id` / `GAME_ID` must match: `[a-z0-9_]+`
- Uppercase, hyphen, or spaces are invalid and should be rejected before launch orchestration.
