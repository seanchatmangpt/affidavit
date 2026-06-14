# Changelog — Affidavit

All notable changes to the Affidavit provenance layer are documented here.

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
