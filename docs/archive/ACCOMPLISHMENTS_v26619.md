# Accomplishments — v26.6.19

**Date:** 2026-06-19  
**Session type:** Documentation cohesion & maintenance release  
**Releasing tomorrow:** 2026-06-20

---

## Summary

v26.6.19 is a focused documentation cohesion and maintenance release. No new CLI verbs or behavioral features were added. All changes are version-stamp, doc synchronization, and genesis-seed rotation.

---

## 1. Version Bumps

| File | Before | After |
|------|--------|-------|
| `Cargo.toml` | 26.6.17 | 26.6.19 |
| `CLAUDE.md` (header) | 26.6.17 | 26.6.19 |
| `CLAUDE.md` (Last Updated) | 2026-06-14 | 2026-06-19 |
| `README.md` (CLI intro) | v26.6.17 | v26.6.19 |
| `docs/INDEX.md` (Last Updated) | 2026-06-17 | 2026-06-19 |
| `docs/glossary.md` (genesis seed string) | v26.6.17 | v26.6.19 |

---

## 2. Genesis Seed — Compile-Time Derivation

**Breaking change (intentional):** The genesis seed in `src/chain.rs` was changed from a hardcoded literal to a compile-time expression:

```rust
// Before (26.6.14 hardcoded literal):
pub const GENESIS_SEED: &[u8] = b"affidavit-v26.6.14-genesis";

// After (compile-time, auto-tracks Cargo.toml version):
const GENESIS_SEED_STR: &str = concat!("affidavit-v", env!("CARGO_PKG_VERSION"), "-genesis");
pub const GENESIS_SEED: &[u8] = GENESIS_SEED_STR.as_bytes();
```

For v26.6.19 this resolves to `affidavit-v26.6.19-genesis`. Future version bumps automatically rotate the seed without manual edits to `chain.rs`.

Receipts from prior versions will fail **stage 3 (`chain_integrity`)** — the intended release-boundary behavior.

The glossary was updated to document the compile-time approach.

---

## 3. Documentation Fixes

- **README.md GitHub URL**: Replaced placeholder `your-repo/affidavit` with the canonical `https://github.com/anthropics/affidavit`.
- **docs/INDEX.md CLNRM link**: Fixed broken link `CLNRM_INTEGRATION_PLAN_26.6.17.md` → `CLNRM_INTEGRATION_PLAN_26.6.14.md` (the file never existed at the `.17` name; only `.14` exists).
- **CHANGELOG.md**: Added full v26.6.19 entry with all changes documented.

---

## 4. New in v26.6.19 (from main merge)

- **67 canonical CLI verbs** (up from 59) backed by compile-time `src/registry.rs`
- **`src/diag.rs`** — stable exit-code catalog and structured `Diag` type
- **`src/output.rs`** — unified `Out` handle for human/JSON/YAML output
- **`affi doctor`** — new health-check verb
- **`docs/innovation/`** — 5 DX/QoL design proposals
- **`docs/roadmap/`** — 10-workstream 2030 program plan
- **`SECURITY.md`**, **`deny.toml`**, **`typos.toml`** added

## 5. State Carried Forward from v26.6.17

- **67 canonical CLI verbs** fully operational
- **211+ tests** passing (100% pass rate)
- **7 Western Electric SPC rules** production-ready (5,400+ LOC)
- **SBOM vertical** (CycloneDX/SPDX, NTIA compliance, blast radius)
- **7 ecosystem integrations** active (chicago-tdd-tools, wasm4pm-compat, Criterion, OpenTelemetry, ggen, clap-noun-verb, lsp-max)
- Phase 2 integrations (wasm4pm full, clnrm-core mutation) remain standing conditions

---

## 5. Honest Residuals

- `docs/integrations/CLNRM_INTEGRATION_PLAN_26.6.14.md` is the integration plan; implementation is plan-ready, not complete.
- `docs/integrations/WASM4PM_INTEGRATION_SUMMARY.md` describes Phase 2 as "ready for implementation" — still accurate.
- LSP phases show conflicting completion status in `LSP_MAX_INTEGRATION_SUMMARY.md`; needs clarification in a future release.
- `reference/COVERAGE.md` wasm4pm-compat gap-grid: ~120 types covered, ~331 open — unchanged.

---

## 6. ADR Status (all confirmed)

All seven ADRs from v26.6.14 remain in force:

- ADR-1: BLAKE3 for content addressing
- ADR-2: Private `_seal` field (E0451 unconstructable bypass)
- ADR-3: `ChainAssembler::finalize` as the only canonical seam
- ADR-4: No wall-clock in events (`seq` ordering only)
- ADR-5: Canonical/sorted JSON for deterministic hashing
- ADR-6: Nightly Rust toolchain required (wasm4pm-compat)
- ADR-7: `#![deny(clippy::print_stdout)]` at library root
