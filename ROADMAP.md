# affidavit — Project Roadmap

**Version:** 26.6.19 → beyond  
**Date:** 2026-06-21  
**Status:** living document — updated each release

---

## Current State

affidavit ships **67 canonical CLI verbs** across 10 groups backed by a compile-time
static registry (`src/registry.rs`). The 7-stage certify pipeline is production-ready.
The BLAKE3 chain, sealed receipts, and determinism guarantees are stable.

### What works
- Full emit → assemble → verify lifecycle
- 7-stage verifier: decode, check_format, chain_integrity, continuity,
  verify_commitments, evaluate_profile, emit_verdict
- Sealed receipts (private `_seal` field, E0451 unconstructable bypass)
- Compile-time genesis seed (`concat!("affidavit-v", env!("CARGO_PKG_VERSION"), "-genesis")`)
- Stable exit-code catalog (`src/diag.rs`)
- Unified output handle (`src/output.rs`, not yet wired into all handlers)
- `affi doctor` environment health check
- OCEL, SBOM, conformance, quality-monitor features behind feature flags

### Active build situation
`wasm4pm-compat v26.6.13` (registry) is broken on current nightly Rust. Workaround:
`stubs/wasm4pm-compat/` local path crate with `[patch.crates-io]` override.

---

## Bug Ledger

| # | Sev | Status | Defect | Location |
|---|-----|--------|--------|----------|
| B1 | High | **FIXED** | Output stream split (stdout vs stderr) | `handlers.rs` |
| B2 | High | **FIXED** | Hand-built JSON via `format!` (injection risk) | `handlers.rs` |
| B3 | High | **Partial** | `load_receipts_from_path` warns but doesn't surface load failures as REJECT results | `handlers.rs:87` |
| B4 | High | **FIXED** | Genesis seed version drift (hardcoded vs compile-time) | `chain.rs:25-27` |
| B5 | Med  | **Open** | `monitor` verb is a stub; real `FileWatcher` not wired | `handlers.rs:2645` |
| B6 | Med  | **FIXED** | Exit codes: REJECT now uses `exit_codes::REJECT` (2) | `handlers.rs:360,378` |
| B7 | Low  | **FIXED** | Duplicate `receipt-throughput.rs` / `receipt_throughput.rs` | `src/verbs/` |
| B8 | Low  | **Open** | Shell completions cover only ~4 of 67 verbs; no PowerShell | `completions/` |
| B9 | Low  | **FIXED** | README claimed 59 verbs; now correctly states 67 | `README.md` |
| B10 | Info | **Open** | `linkme` declared but unused; `doctor`'s `DoctorCheck` registry would be first use | `Cargo.toml:42` |

---

## P0 — Correctness & Build Stability

These are immediate blocking items or high-severity correctness fixes.

### [P0-1] wasm4pm-compat stub completeness
**Status:** In progress  
**What:** The local stub at `stubs/wasm4pm-compat/src/lib.rs` must cover the full
API surface used by ~100 reference tests (powl, bpmn, eventlog, dfg, declare,
causal_net, ocpq, conformance, correlation, process_tree, receipt, temporal, …).  
**Done when:** `cargo test` passes with no compilation errors from wasm4pm_compat imports.

### [P0-2] Fix B3: surface load failures in verify_family / query
**Status:** Open  
**What:** `load_receipts_from_path` in `handlers.rs:71` currently skips unreadable
receipts with a warning. Callers like `verify_family` and `query` should surface
these as explicit REJECT entries rather than silently omitting them from results.  
**Location:** `src/handlers.rs:71-96`, `src/handlers.rs:383-430`  
**Done when:** `verify_family` output includes a REJECT entry for each file that
fails to parse, with the parse error as the reason.

### [P0-3] Wire monitor to real FileWatcher (B5)
**Status:** Open  
**What:** `handlers.rs:2645` prints a stub message. The `FileWatcher` type in
`src/quality.rs:1158` already implements continuous monitoring. Wire them together.  
**Location:** `src/handlers.rs:2630-2660`, `src/quality.rs`  
**Done when:** `affi quality monitor` actually starts the file watcher loop.

---

## P1 — High-Leverage Features

### [P1-1] Integrate Out handle into remaining handlers
**Status:** Open  
**What:** `src/output.rs` defines `Out` (human/json/yaml routing, strict
data→stdout / chatter→stderr). About a third of handlers still use raw `println!`
for both data and informational messages. Route all data output through `Out`.  
**Scope:** `src/handlers.rs` (major), `src/verbs/*.rs`

### [P1-2] `affi doctor --fix` for environment repairs
**Status:** Open  
**What:** Environment doctor already reports findings. Add `--fix` flag that
auto-applies safe repairs: regenerate stale shell completions, archive stale
`working.json`, check for outdated genesis seed format.  
**Location:** `src/verbs/doctor.rs`, `src/handlers.rs`

### [P1-3] `affi fix` verb — receipt store safe repairs
**Status:** Open  
**What:** Two safe operations on receipts: Finalize (re-seal a working receipt
whose chain is still intact) and Quarantine (move a tampered receipt aside with
a `.quarantine` extension and a sidecar explaining why). Must be `--dry-run` first.  
**New file:** `src/verbs/fix.rs`

### [P1-4] `affi why <RECEIPT>` — explain rejection
**Status:** Open  
**What:** Given a REJECT verdict, explain in plain language why each stage failed
and suggest concrete remediation steps. Builds on `diagnose` but goes further.  
**Location:** `src/verbs/diagnose.rs` or new `src/verbs/why.rs`

### [P1-5] Shell completions from registry
**Status:** Open  
**What:** Generate bash/zsh/fish/pwsh completions programmatically from
`src/registry.rs` rather than maintaining hand-written stubs. Completions cover
all 67 verbs with argument hints.  
**New file:** `src/bin/gen_completions.rs` or build script

### [P1-6] `affi guide search <keyword>` — verb discovery
**Status:** Open  
**What:** Fuzzy keyword search over the registry's summary and keyword fields.
Returns ranked verb suggestions. Already wired in `registry.rs` via the `search`
method; expose it as a CLI verb in the `guide` noun group.  
**Location:** `src/verbs/guide.rs`, `src/registry.rs`

---

## P2 — Depth & Polish

### [P2-1] `affi watch` daemon
**Status:** Open  
**What:** Replaces the `monitor` stub with a proper `affi watch <PATH>` command
that starts the `FileWatcher` event loop, debounces rapid changes, and runs
`verify` on each receipt on save. Hooks into quality rules.  
**Dependency:** P0-3

### [P2-2] Content-addressed verdict cache
**Status:** Open  
**What:** Cache `(receipt_content_address, verifier_version) → Verdict`. Skip
re-verification when neither the receipt nor the binary has changed. Critical for
large receipt stores.

### [P2-3] DoctorCheck framework with linkme registry
**Status:** Open  
**What:** Formalize `DoctorCheck` as a trait with a `linkme` distributed slice so
plugins can register checks. Uses the `Finding { check, status, finding,
remediation, auto_fixable }` type. This gives `linkme` its first idiomatic use (B10).

### [P2-4] REPL upgrade: registry-driven dispatch
**Status:** Open  
**What:** The `affi-shell` REPL currently has a hand-maintained 11-of-67 verb
dispatch. Drive it from `registry.rs` so all verbs are available. Add tab-completion
and a `Session` object that tracks the active working-chain.  
**Location:** `src/bin/affi-shell.rs`

### [P2-5] Generated man pages
**Status:** Open  
**What:** Auto-generate `man/affi-<noun>-<verb>.1` from registry entries. Replaces
or supplements `affi guide man`.

---

## Workstream Map

Dependency order (→ means "unblocks"):

```
P0-1 (stub) → cargo test green
P0-2 (B3)   → accurate verify_family results
P0-3 (B5)   → P2-1 (watch daemon)

P1-1 (Out)  → consistent stdout/stderr contract for all verbs
P1-2 (fix)  → P1-3 (receipt repairs)
P1-5 (completions) → P2-5 (man pages)
P1-6 (guide search) → P2-4 (REPL)

P2-3 (DoctorCheck) → richer affi doctor findings
```

---

## Release Milestones

| Release | Theme | P-items |
|---------|-------|---------|
| 26.6.22 | Correctness sprint | P0-1, P0-2, P0-3 |
| 26.6.25 | DX sprint | P1-1, P1-2, P1-4, P1-6 |
| 26.7.1  | Feature sprint | P1-3, P1-5, P2-1, P2-3 |
| 26.7.5  | Polish sprint | P2-2, P2-4, P2-5 |

---

## ADR Status (all in force)

| ADR | Decision | Status |
|-----|----------|--------|
| ADR-1 | BLAKE3 for content addressing | ✓ stable |
| ADR-2 | Private `_seal` field (E0451 unconstructable bypass) | ✓ stable |
| ADR-3 | `ChainAssembler::finalize` as only canonical seam | ✓ stable |
| ADR-4 | No wall-clock in events (`seq` ordering only) | ✓ stable |
| ADR-5 | Canonical/sorted JSON for deterministic hashing | ✓ stable |
| ADR-6 | Nightly Rust toolchain required (wasm4pm-compat) | ⚠ mitigated via stub |
| ADR-7 | `#![deny(clippy::print_stdout)]` at library root | ✓ stable |

---

*See `docs/innovation/00-SYNTHESIS.md` for the full bug ledger narrative and
`docs/roadmap/` for the 10-workstream 2030 program plan.*
