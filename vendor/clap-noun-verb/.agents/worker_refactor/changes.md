# Summary of Changes

## 1. Root Workspace & Examples
- Added `clap-noun-verb-utils = { path = "utils" }` to `[dev-dependencies]` in `/Users/sac/clap-noun-verb/Cargo.toml`.
- Updated `examples/ggen/validators.rs` to import `clap_noun_verb_utils::adapters::parse_key_val` and refactored the `validate_template_vars` function to parse key-value variables using this utility.
- Added `parking_lot = "0.12"` to root dependencies in `Cargo.toml`.
- Added test configuration for `telemetry_cli_tests` to `Cargo.toml`.

## 2. Standalone Playgrounds (`playground` and `examples/playground`)
- Added `clap-noun-verb-utils` dependency to both packages' `Cargo.toml` using relative paths (`../utils` and `../../utils`).
- Added `clap_complete = "4.5"` as a direct dependency in both packages' `Cargo.toml`.
- Deleted `src/domain/completions.rs` and removed `pub mod completions;` and its re-exports from `src/domain/mod.rs` in both playground packages.
- Refactored shell completions and manpage generation in `src/commands/meta.rs` for both packages:
  - Extracted the shell variant match block into a non-verb helper function `parse_shell` to keep cyclomatic complexity ≤ 5 and conform to the Poka-Yoke guard constraint (FM-1.1).
  - Used `clap_noun_verb_utils::completions::generate_completions` inside the completions verb. Retrieved the CLI command dynamically via thread-local cache `ACTIVE_COMMAND` (falling back to locking registry) to prevent reentrant mutex deadlocks.
  - Used `clap_noun_verb_utils::mangen::generate_manpage` in the manpage verb, rendering directly to stdout (`&mut std::io::stdout()`).
  - Mapped `std::io::Error` to `NounVerbError` via `e.to_string()` to satisfy trait bounds.
- Refactored configuration loading in both packages:
  - Updated `src/domain/config.rs`: Refactored `Config` to be a standard struct with typed fields (`output_dir`, `default_family`, `latex_engine`, `ontology_path`), implementing `Default`, `.get()`, `.all_entries()`, and `Config::is_valid_key` methods.
  - Updated `src/commands/config.rs`: Integrated `clap_noun_verb_utils::adapters::LayeredConfigAdapter::<Config>::new` with `ggen.toml` path and environment prefix `GGEN_` using default/empty `ArgMatches` for resolution.

## 3. Framework
- In `src/cli/registry.rs`, introduced the thread-local `ACTIVE_COMMAND` static variable and cached the built `Command` in it during CLI initialization. This allows downstream verbs to access the constructed CLI structure dynamically without causing reentrant deadlocks.
