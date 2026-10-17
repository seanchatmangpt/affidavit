## 2026-05-28T19:03:14Z
You are teamwork_preview_worker.
Your working directory is /Users/sac/clap-noun-verb/.agents/worker_refactor.

Your task is to integrate the newly created shared `utils` (`clap-noun-verb-utils`) package across the workspace examples, the standalone playground packages, and associated commands.

Specifically, implement the following refactorings:

1. **Root Workspace & Examples**:
   - Add `clap-noun-verb-utils = { path = "utils" }` to `[dev-dependencies]` in `/Users/sac/clap-noun-verb/Cargo.toml`.
   - Update `examples/ggen/validators.rs` to import `clap_noun_verb_utils::adapters::parse_key_val` and refactor the `validate_template_vars` function to parse key-value variables using this utility. Make sure any missing imports are resolved.
   - Verify that all examples and workspace tests compile cleanly (`cargo build --examples` and `cargo test`).

2. **Standalone Playgrounds** (both `/Users/sac/clap-noun-verb/playground` and `/Users/sac/clap-noun-verb/examples/playground`):
   - Add `clap-noun-verb-utils` dependency to both `Cargo.toml` files, using relative paths (`../utils` and `../../utils` respectively).
   - Delete `src/domain/completions.rs` in both playground packages and remove `pub mod completions;` from their parent `src/domain/mod.rs` files.
   - Refactor completions and manpage generation in both `src/commands/meta.rs` files:
     - Use `clap_noun_verb_utils::completions::generate_completions` inside the completions verb. You can get the current CLI `Command` to pass to the completions generator via `clap_noun_verb::cli::registry::CommandRegistry::get().lock().unwrap().build_command()`. Parse the shell string argument into a `clap_complete::Shell` variant (mapping "bash", "zsh", "fish", "powershell", "elvish" - and pwsh to powershell).
     - Use `clap_noun_verb_utils::mangen::generate_manpage` inside the manpage verb, rendering directly to stdout (`&mut std::io::stdout()`) from the CLI `Command` retrieved from the global registry.
   - Refactor configuration loading in both packages:
     - Update `src/domain/config.rs`: Refactor the `Config` struct to be a standard struct with typed fields: `output_dir: String`, `default_family: String`, `latex_engine: String`, `ontology_path: String`. Implement `Default`, `.get()`, `.all_entries()`, and `Config::is_valid_key` methods on it to preserve the existing API surface.
     - Update `src/commands/config.rs`: Use `clap_noun_verb_utils::adapters::LayeredConfigAdapter::<Config>::new` with `ggen.toml` file path and environment prefix `GGEN_` to resolve the configuration. Use a default/empty `ArgMatches` for resolution.
   - Verify both playground targets compile cleanly and pass their unit/integration tests (`cargo build` and `cargo test` inside each directory).

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT
hardcode test results, create dummy/facade implementations, or
circumvent the intended task. A Forensic Auditor will independently
verify your work. Integrity violations WILL be detected and your
work WILL be rejected.

Please execute these changes, verify that the workspace and both playground targets build and test cleanly, and write a summary of changes to `changes.md` and a final handoff report to `handoff.md`. Send a completion message back.
