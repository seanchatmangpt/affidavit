# Project: Integrate Shared Utils Crate

## Architecture
- `utils` (`clap-noun-verb-utils`): Provides shared modules (`completions`, `mangen`, `markdown`, `number_parsing`, `display_json`, `adapters`, `help`).
- `examples`: Contains tutorial, how-to, and reference CLI examples. Need to refactor these to import from `clap-noun-verb-utils`.
- `playground`: A standalone playground target outside the main workspace. Needs update to consume `clap-noun-verb-utils` structures instead of redundant local utilities.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Exploration & Target Mapping | Identify stubs, custom helpers, and adapters to be replaced in `examples/` and `playground/`. | None | DONE |
| 2 | Examples Refactoring | Update `examples/` targets to import and consume functions/traits from `clap-noun-verb-utils`. | M1 | IN_PROGRESS |
| 3 | Playground & Commands Refactoring | Update standalone `playground/` to consume `clap-noun-verb-utils`. | M2 | IN_PROGRESS |
| 4 | Compilation & Verification | Build and test workspace (`cargo test`, `cargo build --examples`) and playground to verify no regressions. | M3 | PLANNED |
| 5 | Forensic Verification | Run Forensic Auditor to ensure compliance with integrity and design specifications. | M4 | PLANNED |

## Interface Contracts
### examples / playground ↔ clap-noun-verb-utils
- Refactored crates will depend on `clap-noun-verb-utils` via Cargo.toml.
- Custom/stub implementations in local modules will be removed, and imports redirected to `clap_noun_verb_utils::*`.
