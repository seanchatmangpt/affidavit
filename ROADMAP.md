# affidavit — Project Roadmap

**Version:** 26.9.6 → beyond
**Date:** 2026-09-06
**Status:** living document — re-verified against the code each release

---

## Current State

affidavit ships **77 canonical CLI verbs** across 11 groups backed by a
compile-time static registry (`src/registry.rs`). The 7-stage certify pipeline is
production-ready. The BLAKE3 chain, sealed receipts, and determinism guarantees
are stable. As of v26.9.6 the evidence federation kernel has an operator surface.

### What works
- Full emit → assemble → verify lifecycle
- 7-stage verifier: decode, check_format, chain_integrity, continuity,
  verify_commitments, evaluate_profile, emit_verdict
- Sealed receipts (private `_seal` field, E0451 unconstructable bypass)
- Compile-time genesis seed (`concat!("affidavit-v", env!("CARGO_PKG_VERSION"), "-genesis")`)
  and `affi --version` reporting that same version
- Stable exit-code catalog (`src/diag.rs`)
- Unified output handle (`src/output.rs`) plus the crate-internal `outln!`/`out!`
  macros keeping `#![deny(clippy::print_stdout)]` honest
- `affi doctor` (with `--fix`) over a `linkme`-discovered `DoctorCheck` registry
- **Federation courts** — `affi standing`, `affi ecosystem`, `affi errc` over the
  `standing` / `ecosystem` / `errc` / `errc_claim_assurance` certifiers
  (see `docs/FEDERATION.md`)
- Registry ↔ ontology ↔ projection parity, enforced by tests
- OCEL, SBOM, conformance, quality-monitor features behind feature flags

### Build state
The root crate builds, tests, and lints clean. The real published
`wasm4pm-compat 26.8.7` is the admitted structural dependency; `wasm4pm` and
`clnrm-core` remain fenced by local stubs via `[patch.crates-io]`. A stub is a
named capability boundary, not proof upstream behaviour executed — do not
silently broaden one. Run the full AGENTS.md §6 ladder with `just validate`.

---

## Bug Ledger

Re-verified against the code at 26.9.6. Several items previously listed as open
had already shipped; the statuses below carry the evidence.

| # | Sev | Status | Defect | Evidence |
|---|-----|--------|--------|----------|
| B1 | High | **FIXED** | Output stream split (stdout vs stderr) | `src/output.rs`, `src/macros.rs` |
| B2 | High | **FIXED** | Hand-built JSON via `format!` (injection risk) | `handlers.rs` uses `serde_json::json!` |
| B3 | High | **FIXED** | `load_receipts_from_path` swallowed load failures | `handlers.rs:429` emits a REJECT entry per failure |
| B4 | High | **FIXED** | Genesis seed version drift | `chain.rs:25`; witnessed by `tests/release_identity.rs` |
| B5 | Med  | **FIXED** | `monitor` was a stub; `FileWatcher` not wired | `handlers.rs:3052` behind the `file-watch` feature |
| B6 | Med  | **FIXED** | REJECT used a generic exit code | `handlers.rs` uses `exit_codes::REJECT` (2) |
| B7 | Low  | **FIXED** | Duplicate `receipt-throughput.rs` | removed from `src/verbs/` |
| B8 | Low  | **Partial** | Completions covered ~4 of 69 verbs | v26.9.6 covers all 77 verbs and all 4 nouns in bash/zsh/fish; **no PowerShell**, and they are still hand-maintained (see P1-5) |
| B9 | Low  | **FIXED** | README verb count wrong | witnessed by `tests/release_identity.rs` |
| B10 | Info | **FIXED** | `linkme` declared but unused | `src/doctor_check.rs` distributed slice, consumed at `handlers.rs:4456` |
| B11 | Med | **FIXED** | `affi --version` reported the clap-noun-verb version (`cli 26.6.2`) | `src/bin/affi.rs` answers the bare top-level flag |
| B12 | Med | **FIXED** | `why`, `fix`, `install-git-hook`, `monitor` shipped undeclared in the ontology | declared in `ontology/affi-cli.ttl`; blocked by `registry.rs` parity test |
| B13 | Low | **Open** | `clap-noun-verb` appends its rendering of each verb's return value to stdout, so redirecting certify output yields unparseable JSON | mitigated by `--out` on the federation verbs; other verbs still affected |

---

## Shipped in 26.9.6

| Item | What landed |
|------|-------------|
| **Federation CLI surface** | `src/federation.rs` + 8 verbs across 3 new nouns; the kernel is reachable |
| **Ontology parity** | `every_registry_verb_is_declared_in_the_ontology`, `every_registry_entry_has_a_verb_projection` |
| **Release identity** | version bump to 26.9.6, `affi --version` fixed, `tests/release_identity.rs` |
| **`just validate`** | the AGENTS.md §6 ladder as one recipe |
| **Completions** | all 77 verbs, all 4 nouns, bash/zsh/fish |
| **`docs/FEDERATION.md`** | operator guide, exit-code contract, executed worked example |

---

## P1 — High-Leverage, Still Open

### [P1-5] Generate shell completions from the registry
**Status:** Open
**What:** `completions/affi.{bash,zsh,fish}` are hand-maintained. v26.9.6 brought
them back into sync, but nothing prevents the next verb from desynchronising
them again — the registry parity tests cover the ontology and the projection,
not the completions. Generate them from `REGISTRY` instead, and add PowerShell
(closes B8 fully).
**New file:** `src/bin/gen_completions.rs` or a build script
**Done when:** adding a `VerbEntry` and regenerating is the only step, and a test
fails if the checked-in completions differ from the generated ones.

### [P1-6] Return-value rendering contract (B13)
**Status:** Open
**What:** Every verb's stdout carries a trailing `null` from the framework's
return-value rendering, so `affi receipt verify --format json > v.json` is not
parseable. The federation verbs route around this with `--out`; the general fix
is either a `--quiet`-by-default data path or an upstream change.
**Done when:** `affi <any verb> --format json` produces a single JSON document on
stdout.

---

## P2 — Depth & Polish

### [P2-2] Content-addressed verdict cache
**Status:** Open
**What:** Cache `(receipt_content_address, verifier_version) → Verdict`. Skip
re-verification when neither the receipt nor the binary has changed. Critical for
large receipt stores. The genesis-seed/version binding makes the cache key
naturally correct across releases.

### [P2-4] REPL upgrade: registry-driven dispatch
**Status:** Open
**What:** `src/bin/affi-shell.rs` hand-maintains a 15-arm dispatch against a
77-verb registry, and does not reference `registry.rs` at all. Drive it from the
registry, add tab completion, and add a `Session` that tracks the active
working-chain.

### [P2-5] Generated man pages
**Status:** Open
**What:** Auto-generate `man/affi-<noun>-<verb>.1` from registry entries.
**Dependency:** P1-5 (same generation seam)

### [P2-6] Federation receipt DAG
**Status:** Open
**What:** Every federation profile already carries `previous_receipt` for
receipt-DAG lineage, but nothing walks the chain. Add
`affi standing lineage <RECEIPT>` / `affi ecosystem lineage` to resolve and
verify a whole ancestry, not one link.
**Dependency:** v26.9.6 federation courts

---

## Workstream Map

```
P1-5 (generated completions) → P2-5 (man pages)
P1-6 (stdout contract)       → clean machine consumption for all 77 verbs
P2-4 (REPL from registry)    → one dispatch surface instead of two
P2-6 (federation DAG)        → multi-release standing histories
```

---

## Release Milestones

| Release | Theme | Items |
|---------|-------|-------|
| 26.6.22 | Correctness sprint | B1–B7, output routing, clippy gate |
| 26.9.1  | Federation kernel (library) | standing, ecosystem, errc, claim assurance |
| **26.9.6** | **Federation surface + release identity** | **the kernel becomes reachable; B11, B12** |
| next    | Generation sprint | P1-5, P2-5 |
| later   | Contract sprint | P1-6, P2-2 |
| later   | Lineage sprint | P2-4, P2-6 |

---

## ADR Status (all in force)

| ADR | Decision | Status |
|-----|----------|--------|
| ADR-1 | BLAKE3 for content addressing | ✓ stable |
| ADR-2 | Private `_seal` field (E0451 unconstructable bypass) | ✓ stable |
| ADR-3 | `ChainAssembler::finalize` as only canonical seam | ✓ stable |
| ADR-4 | No wall-clock in events (`seq` ordering only) | ✓ stable |
| ADR-5 | Canonical/sorted JSON for deterministic hashing | ✓ stable |
| ADR-6 | Date-pinned nightly toolchain | ✓ stable (`rust-toolchain.toml`) |
| ADR-7 | `#![deny(clippy::print_stdout)]` at library root | ✓ stable |
| ADR-8 | The CLI reaches kernel laws; it never adds one | ✓ stable (`src/federation.rs`) |

---

*See `AGENTS.md` for execution doctrine, `docs/FEDERATION.md` for the federation
courts, and `docs/roadmap/` for the 10-workstream 2030 program plan.*
