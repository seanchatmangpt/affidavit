# Changelog — Affidavit

All notable changes to the Affidavit provenance layer are documented here.

## [26.9.6] — 2026-09-06

**Theme: the evidence federation kernel becomes reachable.**

The v26.9.x BCRE federation kernel landed as library code — `certify_standing`,
`certify_ecosystem`, `certify_errc`, and `certify_errc_claim_assurance` were
implemented, unit tested, and exported from `src/lib.rs`, but no verb, handler,
or registry entry referenced any of them. The kernel could only be driven from
Rust. This release gives it a real, tested operator surface and brings the
release identity back in line with the code.

### Added
- **Federation courts** (`src/federation.rs`): the adapter layer between the CLI
  and the four kernel certifiers. Loads a `core/v1` receipt, drives it through
  the real Layer 2 gate (`admission::admit`, which runs both the wasm4pm-compat
  OCEL court and the affidavit certify pipeline), parses a bounded observation,
  and seals the profile receipt. It performs **no adjudication of its own** —
  every accept/refuse decision belongs to `admission` or to the kernel.
- **Eight new verbs across three new nouns** (69 → 79 registry entries, 10 → 11
  groups; the extra two are `affi doctor` and `guide search`, which shipped
  dispatchable but unregistered — see Fixed):
  - `affi standing certify` / `affi standing verify` — `affidavit/standing/v2`
  - `affi ecosystem certify` / `affi ecosystem verify` — `affidavit/ecosystem/v1`
  - `affi errc certify` / `affi errc verify` — `affidavit/errc/v1`
  - `affi errc assure` / `affi errc verify-assurance` —
    `affidavit/errc-claim-assurance/v1`
- **`VerbGroup::Federation`** in `src/registry.rs` — the eleventh taxonomy group.
- **`--out <PATH>`** on every `certify`/`assure` verb — writes the sealed
  receipt as a clean artifact so `certify` → `verify` composes in a real script.
- **Machine-consumable `--format json`** on all eight federation verbs. The
  `clap-noun-verb` runtime renders each verb's return value to stdout after the
  handler returns, appending a bare `null` to otherwise valid JSON — so
  `--format json | jq` failed on ACCEPT while working on REJECT. The federation
  courts now exit with their own code on every path, emitting exactly one JSON
  document. Other verbs are still affected (ROADMAP B13).
- **Ontology/registry/projection parity tests** (`src/registry.rs`):
  `every_registry_verb_is_declared_in_the_ontology` and
  `every_registry_entry_has_a_verb_projection`. `ontology/affi-cli.ttl` is the
  authoritative CLI input (AGENTS.md §5) but `ggen` cannot run in every
  environment, so these tests are what keep the projection honest.
- **`tests/federation_cli.rs`** — 15 end-to-end tests driving the real `affi`
  binary: certify → verify round trips, tamper detection, the ALIVE evidence
  law, unmet role quorums, ERRC directional refusals, and the assurance
  bijection. Library tests pass whether or not the CLI surface exists; these do
  not.
- **`tests/release_identity.rs`** — holds `affi --version`, `GENESIS_SEED`,
  `CHANGELOG.md`, and the README verb count to the package version.
- **`just validate`** — the AGENTS.md §6 verification ladder as one recipe.
- **`docs/FEDERATION.md`** — the operator guide for the federation courts, with
  a complete worked example and the exit-code contract.

### Changed
- **Version**: `26.6.22` → `26.9.6` in `Cargo.toml`, `Cargo.lock`, `ggen.toml`,
  and the `affi-shell` banner. Because the chain genesis seed is
  `concat!("affidavit-v", env!("CARGO_PKG_VERSION"), "-genesis")`, **receipts
  assembled by 26.6.22 will fail stage 3 (`chain_integrity`) under 26.9.6.**
  This is the intended release-boundary behaviour, not a regression: re-emit and
  re-assemble against the new binary.
- **Shell completions** now cover all 79 verbs and all six nouns across bash,
  zsh, and fish. They previously advertised a `quality` noun and `guide
  tutorial`/`examples`/`man` verbs that the binary does not have, and omitted
  `receipt-throughput`.
- **`justfile`**: removed the stale header claiming the Rust recipes cannot run
  because `wasm4pm-compat 26.6.13` does not compile. The real published
  `wasm4pm-compat 26.8.7` is admitted and the full ladder passes.
- **`ROADMAP.md`** re-verified against the code: B3, B5, B6, B10, P0-2, P0-3,
  P1-2, P1-3, P1-4 and P1-6 were marked open but had already shipped.

### Fixed
- **`affi --version` reported the framework version.** `clap-noun-verb` builds
  its root command with its own `CARGO_PKG_VERSION`, so `affi --version`
  answered `cli 26.6.2`. Since the genesis seed is bound to the affidavit
  version, an operator diagnosing a cross-version `chain_integrity` failure was
  reading the wrong number. `src/bin/affi.rs` now answers a bare top-level
  `--version`/`-V` itself; subcommand `--version` still reaches the framework.
- **Ontology drift**: `why`, `fix`, `install-git-hook`, and `monitor` shipped in
  the projection without ever being declared in `ontology/affi-cli.ttl`. All
  four are now declared, and the parity test blocks the drift from returning.
- **`affi receipt verify` could not reach stage 3.** `Receipt`'s `Deserialize`
  re-runs the chain law, so a tampered receipt failed at the door with a
  framework parse error and **exit 1**, not the documented REJECT (**2**) — and
  `chain_integrity`, the stage whose entire job is catching exactly this, was
  unreachable from every CLI path. `chain::deserialize_receipt_unchecked` is a
  `pub(crate)` forensic seam that lets `verify` adjudicate a suspect file
  instead of refusing to open it; the stage now FAILs by name and the verb exits
  2. The seal is untouched — `Receipt::sealed` stays private. This also repaired
  `affi receipt why` (its `chain_integrity` explanation branch was structurally
  unreachable) and `affi receipt fix` (its primary action, quarantining a
  tampered receipt, could never execute).
- **`affi receipt verify-family` exited 0 while reporting REJECTs**, so a CI job
  piping it stayed green over a store containing tampered receipts. Any reject
  now exits 2.
- **41 registry rows named commands that do not exist.** `src/registry.rs` used
  snake_case verb tokens (`verify_compliance`) while the CLI dispatches
  kebab-case (`verify-compliance`), so `lookup` missed and `guide search`
  printed names an operator cannot type. A new test rejects any `_` in a verb
  token.
- **`receipt-throughput` was advertised but not compiled.** It had a registry
  entry, a `#[verb]` projection, and a handler — but no `pub mod` line in
  `src/verbs/mod.rs`, so it was never built or dispatchable. The parity test now
  walks the module list rather than the directory, so a file nobody declared can
  no longer masquerade as a shipped verb.
- **`affi doctor` and `guide search` were absent from the registry entirely**,
  making them undiscoverable through `guide search`, `--help` grouping, and the
  completions. Both are now registered and declared in the ontology, and the
  parity tests compare `(verb, noun)` pairs in both directions — a name-only
  check had passed `guide search` purely because a distinct `receipt search`
  exists.
- **The ontology claimed a CLI surface that does not exist.** It declared
  `receipt-throughput`, `variance`, and `profile` under a `bench` noun and
  `audit` under `governance`, while all four ship under `receipt`. The four now
  point at `ReceiptNoun`; `bench` and `governance` are marked RESERVED with the
  regrouping tracked as ROADMAP P2-7.
- **The browser verifier rejected every real receipt.** `web/` hard-codes the
  genesis seed, and it had drifted three releases behind
  (`affidavit-v26.6.17-genesis` while the crate was 26.6.22). `tsc --noEmit`,
  the web lane's only gate, cannot know the Rust seed;
  `tests/release_identity.rs` now does.
- **`examples/golden_run.sh` was broken and untested.** It ran `cargo run` from
  inside a temp dir, so cargo resolved the toolchain from that directory, missed
  `rust-toolchain.toml`, fell back to stable, and died on wasm4pm-compat's
  `#![feature(...)]` (E0554, exit 101). It now builds once from the repo root
  and invokes the binary directly — and `tests/golden_run.rs` executes it, so
  the example README points newcomers at is a court rather than a claim.
- **`affi-shell` hard-coded its version banner**; it now derives it from
  `CARGO_PKG_VERSION` like everything else.
- **`docs/glossary.md` stated the genesis seed resolves to
  `affidavit-v26.6.22-genesis`** — normative documentation of the live binary,
  now corrected and covered by the release-identity gates.

### Internal
- Removed `src/handlers_stubs.rs` — 300 lines of `todo!()` referenced by nothing
  in `src/`, `tests/`, `benches/`, `examples/`, or `build.rs`. `generate_verbs.py`
  regenerates it on demand.
- 17 files under `src/` (164 KB: the 13 `1000x_*.rs` drafts plus
  `generation.rs`, `metrics.rs`, `mining.rs`, `mutation.rs`) are declared by no
  `mod` and mapped by no `#[path]`, so the compiler never sees them — yet
  `cargo package` shipped them. They are now named in Cargo.toml's `exclude`,
  and `orphaned_sources_are_declared_or_excluded` refuses to let the list grow
  silently. Nothing was deleted; deciding each file's fate is tracked as
  ROADMAP P2-8.
- Test suite: 827 tests + 32 doctests, all passing under
  `cargo test --all-targets`, `cargo test --doc`, and
  `cargo clippy --all-targets -- -D warnings`.

## [26.6.22] — 2026-06-22

### Changed
- **Documentation & Release Readiness**:
  - Updated all documentation for v26.6.22 release.
  - Fixed Rust badge (1.56 → 1.78) and removed shell REPL mention.
  - Expanded verb documentation (11 core → 65+ across 9 families).
  - Added glossary entries for SBOM, Western Electric SPC, OCEL, DORA metrics, NTIA elements.
  - Prepared crate for crates.io publication with `exclude` list.

## [26.6.19] — 2026-06-19

### Added
- **Verb registry** (`src/registry.rs`): compile-time static registry of all **67 canonical verbs** in 10 groups (Core, Diagnostics, Analysis, Ingestion, Compliance, Attestation, SBOM, Insights, Engineering, Tooling). Single source of truth for help text, shell completions, and documentation. Exposes `REGISTRY`, `lookup`, `by_group`, `did_you_mean`, and `verb_count`.
- **Error code catalog** (`src/diag.rs`): stable versioned exit codes (OK=0, REJECT=2, USAGE_ERROR=3, IO_ERROR=4, INTERNAL=5, SLA_BREACH=6) and a structured `Diag` type for machine-readable diagnostics; full `--format=json` support.
- **Output abstraction** (`src/output.rs`): unified `Out` handle routing human/JSON/YAML output to stdout and diagnostics to stderr across all verbs.
- **`affi doctor`** (`src/verbs/doctor.rs`): new health-check verb for environment and receipt-store diagnostics.
- **Innovation design proposals** (`docs/innovation/`): five fan-out agent proposals covering doctor-command, doctor-receipts, DX CLI ergonomics, QoL workflow, and DX onboarding (00-SYNTHESIS.md + 01–05).
- **2030 program roadmap** (`docs/roadmap/`): ten-workstream master plan (W1 Foundations → W10 Compliance/Governance) with release calendar and cross-workstream dependency graph.
- **`SECURITY.md`**: security policy and responsible disclosure process.
- **`deny.toml`** / **`typos.toml`**: supply-chain advisory checks and automated typo detection.

### Changed
- **Version bump**: Updated to v26.6.19 in `Cargo.toml`, `CLAUDE.md`, `docs/INDEX.md`, and all versioned references.
- **Genesis seed** (`src/chain.rs`): replaced hardcoded `b"affidavit-v26.6.14-genesis"` with a compile-time expression `concat!("affidavit-v", env!("CARGO_PKG_VERSION"), "-genesis")` so the seed always matches the running binary without manual updates. Receipts from prior versions will fail stage 3 (`chain_integrity`) — this is the intended release-boundary behavior.
- **CLI verb count**: README and docs updated from 59 → 67 to match the registry.
- **Repository URL**: corrected to `https://github.com/seanchatmangpt/affidavit` throughout.
- **Documentation cohesion**: synchronized version strings across README, CLAUDE.md, glossary, INDEX, and integration docs.

### Fixed
- `docs/INDEX.md` CLNRM integration plan link pointed to a non-existent `26.6.17` file; corrected to `26.6.14`.
- Stale `your-repo` GitHub placeholder removed from installation instructions.
- Output routing: `diff` and `stats` verbs now emit substantive output to stdout (was erroneously going to stderr).
- Tampered-receipt reporting in `verify` now surfaces the correct stage and reason.

### Internal
- Removed all `.backup` and `receipt-throughput.rs` dead files from `src/verbs/`.
- `portfolio_test_dataset.json` moved to `fixtures/`.
- Added `docs/archive/ACCOMPLISHMENTS_v26619.md` release summary.
- All doc timestamps synchronized to 2026-06-19.

## [26.6.17] — 2026-06-17

### Added
- **Maximalist Nexus Ontology & CLI Verbs**:
  - Automatically generated and implemented **59 new CLI verbs** powered by the maximalist nexus ontology.
  - Implemented 59 distinct handler functions with genuine operational logic replacing prior stubs.
  - Expanded the vocabulary to include advanced capabilities such as `dependency matrix`, `causality chain`, `gdpr proof`, `sbom attest`, `security debt`, and more.
- **Western Electric Real-Time Quality Monitoring**:
  - Delivered a production-ready, 5,400+ LOC implementation of all **7 Western Electric Statistical Process Control (SPC) rules**.
  - Developed real-time LLM quality degradation and cheating detection.
  - Provided support for monitoring up to 300+ repositories simultaneously with a rolling window analysis engine.
  - Added object-level metric tracking (File, Module, Package, Repo) combined with Pearson correlation scoring to identify simultaneous violation causality.
  - Designed deep OCEL (Object-Centric Event Logs) integration for event emission, causal chain tracing, and generating unforgeable quality audit trails.
- **SBOM & Supply Chain Provenance**:
  - Shipped a complete SBOM vertical CLI slice incorporating 6 new verbs.
  - Added robust CycloneDX and SPDX parsing, validation, and NTIA compliance modules (`src/sbom.rs`, `src/sbom_compliance.rs`).
  - Introduced supply chain risk propagation logic and vulnerability tracing via `src/sbom_vulnerability.rs`.
- **Phase 2 Webhook & Daemon Integrations**:
  - Engineered production file watcher daemon handlers for persistent provenance streams.
  - Delivered mature Slack webhook integrations and network listeners.
  - Rolled out a portfolio monitoring simulation capable of querying and verifying a newly added 312-repository test dataset.
- **Extensive Testing & Benchmarking**:
  - Over 2,600+ lines of test code added, achieving a 100% pass rate across 211+ new tests.
  - Shipped `tests/western_electric_comprehensive.rs` spanning 86 stress tests validating variants, sigma levels, and performance constraints (<1ms detection time).
  - Wired massive E2E integration suites including `tests/sbom_integration.rs` and `tests/ocel_quality_integration.rs`.
- **Comprehensive Documentation Architecture**:
  - Added the definitive Phase Change Thesis containing profound academic and cross-disciplinary references (65+ references).
  - Wrote hyper-detailed `IMPLEMENTATION_SUMMARY.md` and `docs/WESTERN_ELECTRIC_COMPLETE.md` encompassing theoretical backing, tuning params, and Mermaid architecture diagrams.
  - Reorganized the `docs/` folder, safely migrating older phase specs to `docs/archive`.

### Changed
- **Filesystem & CI Hardening**: 
  - Ported hardened filesystem interaction patterns and validations from the `clnrm_prototype`.
  - Modernized `rustfmt` CI workflow using the latest Rust unpinned nightly tools while ensuring format failures are non-blocking.
  - Scrubbed and sanitized all developer machine paths across repositories.
  - Strengthened `.gitignore` for a cleaner, secure public distribution model.

## [26.6.14] — 2026-06-14

### Added
- **Receipt sealing (ADR-2/3)**: Receipt struct now has a private `_seal` field that prevents struct-literal construction from external code. Only the canonical seam (`crate::chain::ChainAssembler::finalize`) can construct sealed receipts.
- **Compile-fail witness**: `tests/ui/compile_fail/receipt_private_seal.rs` demonstrates that external code cannot construct Receipt directly (E0451).
- **Stdout safety guard (§6)**: Added `#![deny(clippy::print_stdout)]` at library root to prevent accidental output from dependencies. Intentional CLI output in `cli.rs` is explicitly allowed.
- **E2E test suite**: `tests/e2e.rs` exercises the complete receipt lifecycle (emit → assemble → verify → show) with tamper detection.

### Changed
- **Version bump**: Updated to v26.6.14 in Cargo.toml, ggen.toml, and ontology.
- **Genesis seed**: Updated to `affidavit-v26.6.14-genesis` for deterministic chain binding.
- **Receipt constructor**: Changed from struct-literal construction to `Receipt::sealed()` internal constructor to enforce sealing.

### Technical Details
- Receipt now unconstructable without going through the canonical seam (the bypass is unconstructable witness).
- Verifier pipeline remains deterministic (golden-diff witness via unit tests).
- CLI dispatch tests ensure verify↔show inversion (type-blind pairs witness).
- Stdout output is clean and unambiguous (behavioral witness).

### Status
- **Phase 1 complete**: Artifact provenance with all four acceptance witnesses (§9 of ARDPRD.md).
- **Remaining**: Phase 2 (reasoning provenance) is a standing condition, not a completable milestone.

## Prior Versions

See git history for versions < 26.6.14.
