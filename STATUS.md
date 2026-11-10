# Affidavit — Status Report

**Date:** 2026-11-09
**Current tag:** v26.10.17 (see `CHANGELOG.md`)

Counts and pass/fail figures are receipt-only: run `just validate` for the gate. This file
records state that was re-verified on the date above; older figures were removed (git history).

| Area | Standing | Evidence |
| --- | --- | --- |
| Verb registry parity (100 verbs) | ALIVE | `cargo test --test docs_verb_parity` — 2 passed, 2026-11-09 |
| Value-matrix index | ALIVE | `cargo test --test value_matrix_index` — 1 passed, 2026-11-09 |
| Format | ALIVE | `cargo fmt --all -- --check` — exit 0, 2026-11-09 |
| Full ladder (`just validate`) | UNVERIFIED | not run this session |
| BLS rogue-key claim | BLOCKED | registered DEFECT, capability register row 82 |
| Secure Enclave live KATs | PARTIAL_ALIVE | `#[ignore]`-gated; needs entitlement-signed host |

## See Also

`ROADMAP.md` · `CHANGELOG.md` · `docs/reference/CAPABILITY_REGISTER.md`
