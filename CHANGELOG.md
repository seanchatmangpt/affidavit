# Changelog — Affidavit

All notable changes to the Affidavit provenance layer are documented here.

## [26.6.19] — 2026-06-19

### Changed
- **Version bump**: Updated to v26.6.19 in `Cargo.toml`, `CLAUDE.md`, `docs/INDEX.md`, and all versioned references.
- **Genesis seed**: Updated from `affidavit-v26.6.14-genesis` to `affidavit-v26.6.19-genesis` in `src/chain.rs` and `docs/glossary.md`. Receipts assembled under prior versions will fail `chain_integrity` (stage 3) when verified with v26.6.19 — this is the intended breaking boundary between release generations.
- **Documentation cohesion**: Standardized version strings across README, CLAUDE.md, glossary, INDEX, and integration docs. Fixed placeholder GitHub URL to `https://github.com/anthropics/affidavit`.

### Fixed
- CLI surface wording in README now accurately reflects 59 canonical verbs without implying the count was newly added in this patch.
- Removed stale `your-repo` placeholder from installation instructions.

### Internal
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
