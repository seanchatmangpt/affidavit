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

## 2. Genesis Seed Rotation

**Breaking change (intentional):** The rolling chain hash genesis seed in `src/chain.rs` was updated:

```
Before: GENESIS_SEED = b"affidavit-v26.6.14-genesis"
After:  GENESIS_SEED = b"affidavit-v26.6.19-genesis"
```

Receipts assembled under v26.6.17 (or earlier) will fail **stage 3 (`chain_integrity`)** when verified with v26.6.19. This is the intended release boundary — a new genesis seed creates a clear cryptographic break between receipt generations.

The glossary (`docs/glossary.md`) was updated to reflect the new seed value.

---

## 3. Documentation Fixes

- **README.md GitHub URL**: Replaced placeholder `your-repo/affidavit` with the canonical `https://github.com/anthropics/affidavit`.
- **docs/INDEX.md CLNRM link**: Fixed broken link `CLNRM_INTEGRATION_PLAN_26.6.17.md` → `CLNRM_INTEGRATION_PLAN_26.6.14.md` (the file never existed at the `.17` name; only `.14` exists).
- **CHANGELOG.md**: Added full v26.6.19 entry with all changes documented.

---

## 4. State Carried Forward from v26.6.17

All features and test counts from v26.6.17 are unchanged:

- **59 canonical CLI verbs** fully operational
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
