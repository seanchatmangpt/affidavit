# DOD_MASTER.md — Master Definition of Done Dashboard & Completion Tracker
# affidavit DX/QOL 1000x Initiative

**Project:** affidavit Provenance Layer — DX/QOL 1000x Feature Expansion  
**Version:** 26.6.14  
**Branch:** `claude/zen-cerf-oq87br`  
**Owner:** Sean Chatman (xpointsh@gmail.com)  
**Last Updated:** 2026-06-14  
**Document Status:** ACTIVE — Single Source of Truth

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Feature Status Matrix](#2-feature-status-matrix)
3. [Phase Gate Status](#3-phase-gate-status)
4. [Dependency Map](#4-dependency-map)
5. [Quality Gate Dashboard](#5-quality-gate-dashboard)
6. [Risk Register](#6-risk-register)
7. [Success Metrics Dashboard](#7-success-metrics-dashboard)
8. [Weekly Sprint Summary](#8-weekly-sprint-summary)
9. [Definition of Done Global Contract](#9-definition-of-done-global-contract)
10. [Sign-off Table](#10-sign-off-table)

---

## 1. Executive Summary

### Project Status

| Field | Value |
|-------|-------|
| **Current Date** | 2026-06-14 |
| **Project Start** | 2026-06-14 (Week 1, Day 1) |
| **Planned End** | 2026-07-19 (Week 5, Day 5) |
| **Overall Status** | 🚧 IN PROGRESS — Phase 1 Active |
| **Phase** | Phase 1 of 5 (Receipt Inspection) |
| **Branch** | `claude/zen-cerf-oq87br` |

### Feature Counts

| Category | Primary Features | Benchmarking/Extra | Total |
|----------|-----------------|-------------------|-------|
| Phase 1: Receipt Inspection | 5 | — | 5 |
| Phase 2: Process Discovery | 5 | — | 5 |
| Phase 3: Mutation Testing | 5 | — | 5 |
| Phase 4: Observability | 5 | — | 5 |
| Phase 5: CLI Ergonomics | 5 | — | 5 |
| Benchmarking | — | 5 | 5 |
| **TOTAL** | **25** | **5** | **30** |

> Note: Benchmarking features (Category 3 in the design doc) are tracked here as
> the "Benchmarking" group and cross-referenced in Phase 1 and Phase 3 exits.

### Completion Tracker

```
Overall Progress: 1/30 features complete

[█░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 3%

Phase 1 (Inspection):   [█░░░░] 1/5  (20%)
Phase 2 (Discovery):    [░░░░░] 0/5   (0%)
Phase 3 (Mutation):     [░░░░░] 0/5   (0%)
Phase 4 (Observability):[░░░░░] 0/5   (0%)
Phase 5 (CLI):          [░░░░░] 0/5   (0%)
Benchmarking:           [░░░░░] 0/5   (0%)
```

### Scope Summary

| Metric | Value |
|--------|-------|
| Total features | 30 |
| Total estimated hours | ~88 hours |
| Total new code | ~2,000 lines |
| E2E test suites | 6 |
| E2E test lines | ~850 |
| New source modules | 8 (mining, mutation, metrics, lsp/, shell/, fixture_db, tracing+, cli+) |
| New test files | 6 |
| New bench files | 2 |
| Rollout duration | 5 weeks |

### Baseline: What Exists Today (v26.6.14)

The following are **already complete** and are NOT tracked as DX/QOL initiative features:

- ✅ Core receipt chain (emit → assemble → verify → show)
- ✅ 7-stage verifier pipeline
- ✅ BLAKE3 commitment chain integrity
- ✅ Private `_seal` anti-forgery guarantee (E0451 compile-fail)
- ✅ 59 passing tests across 9 test suites
- ✅ Criterion benchmarking harness (basic: ~2.3µs chain_append)
- ✅ OpenTelemetry span emission on verify (basic)
- ✅ LSP diagnostic plumbing (Verdict → Diagnostic)
- ✅ wasm4pm-compat typestate (Evidence<Receipt, Admitted, W>)
- ✅ chicago-tdd-tools assertion macros (assert_ok!, assert_err!)
- ✅ clnrm-core determinism witness
- ✅ inspect verb (Phase 1.1 — 1 of 5 Phase 1 features DONE)

---

## 2. Feature Status Matrix

### Status Key

| Symbol | Meaning |
|--------|---------|
| 🔲 | Not started |
| 🚧 | In progress |
| ✅ | Complete (all DoD criteria met) |
| 🚫 | Blocked |
| ⏭️ | Deferred / descoped |

### Complete Feature Matrix (30 Features)

| # | Feature | Phase | Library (80%) | Priority | Status | Owner | Est Hrs | Actual Hrs | Lines | Blocker | Test File | E2E Suite | Notes |
|---|---------|-------|--------------|----------|--------|-------|---------|------------|-------|---------|-----------|-----------|-------|
| 1.1 | `affi receipt inspect` | Phase 1 | chicago-tdd | P0 | ✅ | Sean | 3h | 2.5h | 60 | — | `tests/dx_verbs_e2e.rs` | Suite 1 | DONE — first DX feature landed |
| 1.2 | `affi receipt diff` | Phase 1 | chicago-tdd | P0 | 🔲 | Sean | 2h | — | 50 | None | `tests/e2e_inspection.rs` | Suite 1 | Needs DiffResult struct |
| 1.3 | `affi receipt visualize` | Phase 1 | chicago-tdd + ggen | P0 | 🔲 | Sean | 4h | — | 65 | None | `tests/e2e_inspection.rs` | Suite 1 | DOT + JSON output |
| 1.4 | `affi receipt catalog` | Phase 1 | chicago-tdd | P0 | 🔲 | Sean | 2h | — | 35 | None | `tests/e2e_inspection.rs` | Suite 1 | Fixture registry exposure |
| 1.5 | Shell completion (bash/zsh/fish) | Phase 1 | clap-noun-verb | P0 | 🔲 | Sean | 1h | — | 15 | None | `tests/e2e_inspection.rs` | Suite 1 | ggen.toml config |
| 2.1 | `affi receipt model` (DFG/Petri) | Phase 2 | wasm4pm HIM | P1 | 🔲 | Sean | 4h | — | 60 | wasm4pm nightly | `tests/e2e_discovery.rs` | Suite 2 | Feature gate: `discovery` |
| 2.2 | `affi receipt conform` (fitness) | Phase 2 | wasm4pm alignment | P1 | 🔲 | Sean | 3h | — | 45 | 2.1 done first | `tests/e2e_discovery.rs` | Suite 2 | Feature gate: `conformance` |
| 2.3 | `affi receipt predict` (next-activity) | Phase 2 | wasm4pm next-activity | P1 | 🔲 | Sean | 3h | — | 35 | 2.1 done first | `tests/e2e_discovery.rs` | Suite 2 | Feature gate: `predictive` |
| 2.4 | LSP hover on receipt events | Phase 2 | lsp-max | P1 | 🔲 | Sean | 5h | — | 80 | lsp-max stability | `tests/e2e_discovery.rs` | Suite 2 | Feature gate: `lsp` |
| 2.5 | LSP goto-definition (event→handler) | Phase 2 | lsp-max | P1 | 🔲 | Sean | 3h | — | 50 | 2.4 done first | `tests/e2e_discovery.rs` | Suite 2 | Feature gate: `lsp` |
| B.1 | `affi bench receipt-throughput` | Benchmarking | Criterion | P0 | 🔲 | Sean | 3h | — | 80 | None | `tests/e2e_benchmarking.rs` | Suite 3 | New: `benches/throughput.rs` |
| B.2 | `affi bench variance` (process variance) | Benchmarking | wasm4pm variance | P1 | 🔲 | Sean | 3h | — | 55 | B.1 done first | `tests/e2e_benchmarking.rs` | Suite 3 | New: `benches/variance.rs` |
| B.3 | Criterion HTML dashboard | Benchmarking | Criterion HTML | P1 | 🔲 | Sean | 2h | — | 50 | B.1 done first | `tests/e2e_benchmarking.rs` | Suite 3 | CSS theme + summary gen |
| B.4 | `affi bench profile` (flamegraph) | Benchmarking | Criterion + perf | P2 | 🔲 | Sean | 3h | — | 60 | B.1 done first | `tests/e2e_benchmarking.rs` | Suite 3 | Feature gate: `profiling` |
| B.5 | Criterion baseline comparisons | Benchmarking | Criterion | P1 | 🔲 | Sean | 2h | — | 30 | B.1 done first | `tests/e2e_benchmarking.rs` | Suite 3 | Before/after CI |
| 3.1 | `affi mutate receipt` (kill testing) | Phase 3 | clnrm | P1 | 🔲 | Sean | 5h | — | 90 | clnrm integration | `tests/e2e_mutation.rs` | Suite 4 | Feature gate: `mutation` |
| 3.2 | `affi generate test` (Rust codegen) | Phase 3 | chicago-tdd + Tera | P1 | 🔲 | Sean | 3h | — | 55 | None | `tests/e2e_mutation.rs` | Suite 4 | Tera template rendering |
| 3.3 | `affi generate snippet` (code library) | Phase 3 | chicago-tdd | P2 | 🔲 | Sean | 2h | — | 30 | None | `tests/e2e_mutation.rs` | Suite 4 | Pattern search |
| 3.4 | Property-based testing (quickcheck) | Phase 3 | quickcheck | P1 | 🔲 | Sean | 4h | — | 60 | quickcheck dep | `tests/e2e_mutation.rs` | Suite 4 | New: `tests/property_based.rs` |
| 3.5 | Test fixture database (persistent) | Phase 3 | chicago-tdd + serde | P2 | 🔲 | Sean | 4h | — | 80 | None | `tests/e2e_mutation.rs` | Suite 4 | New: `src/fixture_db.rs` |
| 4.1 | `affi receipt trace` (Jaeger spans) | Phase 4 | OTel Jaeger | P1 | 🔲 | Sean | 3h | — | 70 | OTel exporter | `tests/e2e_observability.rs` | Suite 5 | Feature gate: `otel` |
| 4.2 | OTel metrics dashboard (Prometheus) | Phase 4 | OTel Prometheus | P1 | 🔲 | Sean | 4h | — | 100 | 4.1 done first | `tests/e2e_observability.rs` | Suite 5 | New: `dashboards/affidavit.json` |
| 4.3 | OTel baggage (cross-cutting context) | Phase 4 | OTel baggage | P2 | 🔲 | Sean | 2h | — | 40 | 4.1 done first | `tests/e2e_observability.rs` | Suite 5 | Modify `src/tracing.rs` |
| 4.4 | Span events (detailed activity log) | Phase 4 | OTel span events | P2 | 🔲 | Sean | 2h | — | 45 | 4.1 done first | `tests/e2e_observability.rs` | Suite 5 | Modify `src/tracing.rs` |
| 4.5 | SLO monitoring (OTel → SLI) | Phase 4 | OTel metrics | P2 | 🔲 | Sean | 3h | — | 60 | 4.2 done first | `tests/e2e_observability.rs` | Suite 5 | New: SLI calculator |
| 5.1 | Help formatter (ontology-driven docs) | Phase 5 | ggen + clap | P2 | 🔲 | Sean | 2h | — | 50 | None | `tests/e2e_cli.rs` | Suite 6 | Markdown → ASCII |
| 5.2 | Auto-generated examples (fixture-sourced) | Phase 5 | chicago-tdd + ggen | P1 | 🔲 | Sean | 3h | — | 70 | 1.1–1.4 done | `tests/e2e_cli.rs` | Suite 6 | Zero bit-rot guarantee |
| 5.3 | Command aliases (`affi r` → `affi receipt`) | Phase 5 | clap-noun-verb | P2 | 🔲 | Sean | 1h | — | 25 | None | `tests/e2e_cli.rs` | Suite 6 | Ontology alias |
| 5.4 | JSON output for all verbs (`--format=json`) | Phase 5 | serde | P1 | 🔲 | Sean | 3h | — | 70 | None | `tests/e2e_cli.rs` | Suite 6 | Feature gate: `json-output` |
| 5.5 | Interactive shell REPL (`affi-shell`) | Phase 5 | rustyline | P2 | 🔲 | Sean | 4h | — | 90 | rustyline dep | `tests/e2e_cli.rs` | Suite 6 | New: `src/bin/affi-shell.rs` |

**Running totals:** 1 ✅ / 0 🚧 / 29 🔲 / 0 🚫 | ~88h estimated | ~2,000 lines

---

## 3. Phase Gate Status

### Phase 1: Receipt Inspection (Week 1)

**Status:** 🚧 IN PROGRESS  
**Target Completion:** End of Week 1  
**Libraries:** chicago-tdd-tools, clap-noun-verb, ggen  
**E2E Suite:** `tests/e2e_inspection.rs`

| Exit Criterion | Status | Evidence |
|----------------|--------|---------|
| 5 features implemented (1.1–1.5) | 🔲 1/5 | 1.1 inspect done; 1.2–1.5 pending |
| `cargo build --all-features` passes | 🔲 | Not yet verified post-features |
| `cargo test --all` passes | 🔲 | Must remain green throughout |
| E2E inspection test suite passes | 🔲 | `tests/e2e_inspection.rs` not yet written |
| `affi receipt inspect` returns detailed report | ✅ | `tests/dx_verbs_e2e.rs` passes |
| `affi receipt diff a.json b.json` shows diff | 🔲 | Not implemented |
| `affi receipt visualize --format=json` produces graph | 🔲 | Not implemented |
| `affi receipt catalog` lists fixtures | 🔲 | Not implemented |
| Shell completion script contains all verbs | 🔲 | Not implemented |
| All 5 feature handlers are in `src/handlers.rs` | 🔲 | Only inspect exists |

**Phase 1 Completion:** 20% (1/5 features) | **Exit criteria met:** 20% (2/10)

**Blockers:** None — all P0 dependencies (chicago-tdd, clap-noun-verb) are already integrated.

---

### Phase 2: Process Discovery (Week 2)

**Status:** 🔲 NOT STARTED  
**Target Completion:** End of Week 2  
**Libraries:** wasm4pm (HIM + conformance + predictive), lsp-max  
**E2E Suite:** `tests/e2e_discovery.rs`  
**Feature Gates:** `discovery`, `conformance`, `predictive`, `lsp`

| Exit Criterion | Status | Evidence |
|----------------|--------|---------|
| 5 features implemented (2.1–2.5) | 🔲 | — |
| `affi receipt model` produces valid Petri net JSON | 🔲 | — |
| Conformance fitness score ≥0.9 on clean linear receipt | 🔲 | — |
| `affi receipt predict` returns confidence ∈ [0,1] | 🔲 | — |
| LSP hover returns event details | 🔲 | — |
| LSP goto-def routes `emit` → `emit.rs` | 🔲 | — |
| E2E discovery test suite passes | 🔲 | — |
| `#[cfg(feature = "discovery")]` guards in place | 🔲 | — |
| `src/mining.rs` module created | 🔲 | — |
| `src/lsp/hover.rs` and `goto_definition.rs` created | 🔲 | — |

**Phase 2 Completion:** 0% (0/5 features) | **Gate:** Phase 1 must be complete first.

**Blockers:** wasm4pm nightly toolchain stability (see Risk R-1). lsp-max symbol resolution (see Risk R-2).

---

### Phase 3: Mutation Testing (Week 3)

**Status:** 🔲 NOT STARTED  
**Target Completion:** End of Week 3  
**Libraries:** clnrm (mutation operators), chicago-tdd-tools (codegen), quickcheck  
**E2E Suite:** `tests/e2e_mutation.rs`  
**Feature Gate:** `mutation`

| Exit Criterion | Status | Evidence |
|----------------|--------|---------|
| 5 features implemented (3.1–3.5) | 🔲 | — |
| Mutation kill rate ≥90% on baseline receipt | 🔲 | — |
| Generated test code compiles and runs | 🔲 | — |
| Fixture DB insert + search both pass | 🔲 | — |
| 100 random receipts produce decidable verdicts | 🔲 | — |
| E2E mutation test suite passes | 🔲 | — |
| `src/mutation.rs` module created | 🔲 | — |
| `src/fixture_db.rs` module created | 🔲 | — |
| `tests/property_based.rs` created | 🔲 | — |
| quickcheck added to Cargo.toml dev-deps | 🔲 | — |

**Phase 3 Completion:** 0% (0/5 features) | **Gate:** Phase 2 must be complete first.

**Blockers:** clnrm mutation operator integration (needs evaluation — see DX_QOL_IMPLEMENTATION_CHECKLIST.md §Pre-Implementation). quickcheck not yet in Cargo.toml.

---

### Benchmarking Phase (Concurrent with Phases 1 & 3)

**Status:** 🔲 NOT STARTED (basic harness exists; full expansion not started)  
**Target Completion:** By end of Week 3  
**Libraries:** Criterion, wasm4pm variance, perf  
**E2E Suite:** `tests/e2e_benchmarking.rs`

| Exit Criterion | Status | Evidence |
|----------------|--------|---------|
| 5 features implemented (B.1–B.5) | 🔲 | — |
| `benches/throughput.rs` created | 🔲 | — |
| `benches/variance.rs` created | 🔲 | — |
| Criterion HTML dashboard renders | 🔲 | — |
| Flame graph generation works | 🔲 | — |
| Baseline comparison (before/after) works | 🔲 | — |
| E2E benchmarking test suite passes | 🔲 | — |
| Latency scales linearly with event count | 🔲 | — |
| CI regression block configured | 🔲 | — |
| `cargo bench` reports >0 measured for all harnesses | 🔲 | — |

**Benchmarking Completion:** 0% (0/5 features) | (existing basic harness not counted)

**Note:** B.1 (throughput) can start in Week 1 parallel with Phase 1 since Criterion is already integrated.

---

### Phase 4: Observability (Week 4)

**Status:** 🔲 NOT STARTED  
**Target Completion:** End of Week 4  
**Libraries:** OpenTelemetry (Jaeger, Prometheus), Grafana  
**E2E Suite:** `tests/e2e_observability.rs`  
**Feature Gates:** `otel`, `metrics`

| Exit Criterion | Status | Evidence |
|----------------|--------|---------|
| 5 features implemented (4.1–4.5) | 🔲 | — |
| Trace has emit, assemble, verify spans | 🔲 | — |
| Span parent-child relationships correct | 🔲 | — |
| Prometheus metrics export in valid format | 🔲 | — |
| Grafana dashboard JSON passes validation | 🔲 | — |
| Baggage contains receipt_id + timestamp | 🔲 | — |
| Span events include all 7 verifier stages | 🔲 | — |
| SLI computation within SLO thresholds | 🔲 | — |
| E2E observability test suite passes | 🔲 | — |
| `src/metrics.rs` module created | 🔲 | — |
| `dashboards/affidavit.json` created | 🔲 | — |

**Phase 4 Completion:** 0% (0/5 features) | **Gate:** Phase 3 must be complete first.

**Blockers:** OTel Jaeger exporter integration (graceful degradation needed — see Risk R-3).

---

### Phase 5: CLI Ergonomics (Week 5)

**Status:** 🔲 NOT STARTED  
**Target Completion:** End of Week 5  
**Libraries:** ggen (ontology), clap-noun-verb, serde, rustyline  
**E2E Suite:** `tests/e2e_cli.rs`  
**Feature Gates:** `json-output`, `shell`

| Exit Criterion | Status | Evidence |
|----------------|--------|---------|
| 5 features implemented (5.1–5.5) | 🔲 | — |
| Help text has no markdown syntax | 🔲 | — |
| Help text includes ARDPRD cross-references | 🔲 | — |
| All auto-generated examples run without error | 🔲 | — |
| Aliases route to same handlers as full noun | 🔲 | — |
| JSON output has correct schema for all verbs | 🔲 | — |
| Shell REPL accepts load, inspect, diff, help, quit | 🔲 | — |
| Completion script includes all new verbs | 🔲 | — |
| E2E CLI test suite passes | 🔲 | — |
| `src/bin/affi-shell.rs` created | 🔲 | — |
| rustyline added to Cargo.toml dev-deps | 🔲 | — |

**Phase 5 Completion:** 0% (0/5 features) | **Gate:** Phase 4 must be complete first.

**Blockers:** rustyline not yet in Cargo.toml.

---

## 4. Dependency Map

### Feature Dependency Graph

```
LAYER 0 — Always Available (baseline v26.6.14)
  ┌──────────────────────────────────────────────────────────────────┐
  │  emit · assemble · verify · show · inspect(✅)                   │
  │  7-stage verifier · BLAKE3 chain · OTel basic · chicago-tdd asm │
  └──────────────────────────────────────────────────────────────────┘
           │
           ▼
LAYER 1 — Phase 1 (no cross-feature deps within phase)
  ┌───────────────────────────────────────────────────────┐
  │  1.1 inspect ✅  →  (already done)                    │
  │  1.2 diff        (independent)                        │
  │  1.3 visualize   (independent)                        │
  │  1.4 catalog     (independent)                        │
  │  1.5 completion  (independent)                        │
  │  B.1 throughput  (independent — Criterion exists)     │
  └───────────────────────────────────────────────────────┘
           │
           ▼
LAYER 2 — Phase 2 (requires Layer 1 complete)
  ┌──────────────────────────────────────────────────────────────────┐
  │  2.1 model ──────────────────────────────────────────────────────┤
  │    └──► 2.2 conform  (requires 2.1: needs Petri net)             │
  │    └──► 2.3 predict  (requires 2.1: needs discovered model)      │
  │  2.4 LSP hover ─────────────────────────────────────────────────┤
  │    └──► 2.5 LSP goto-def  (requires 2.4: LSP server scaffold)   │
  │  B.2 variance  (requires B.1: throughput baseline)               │
  │  B.3 dashboard (requires B.1: throughput data)                   │
  └──────────────────────────────────────────────────────────────────┘
           │
           ▼
LAYER 3 — Phase 3 (requires Layer 2 complete)
  ┌──────────────────────────────────────────────────────────────────┐
  │  3.1 mutate ────────────────────────────────────────────────────┤
  │    (requires verifier from Layer 0 for kill rate computation)    │
  │  3.2 generate test  (independent — uses chicago-tdd)             │
  │  3.3 generate snippet (independent)                              │
  │  3.4 property-based (independent — uses quickcheck)              │
  │  3.5 fixture DB     (independent — uses serde)                   │
  │  B.4 profile  (requires B.1: throughput baseline)                │
  │  B.5 baselines (requires B.1: throughput + B.4: profiling)      │
  └──────────────────────────────────────────────────────────────────┘
           │
           ▼
LAYER 4 — Phase 4 (requires Layer 3 complete)
  ┌──────────────────────────────────────────────────────────────────┐
  │  4.1 trace ─────────────────────────────────────────────────────┤
  │    └──► 4.2 metrics dashboard (requires 4.1: span infrastructure)│
  │    └──► 4.3 baggage          (requires 4.1: tracer context)     │
  │    └──► 4.4 span events      (requires 4.1: tracer context)     │
  │    └──► 4.5 SLO monitoring   (requires 4.2: metrics)            │
  └──────────────────────────────────────────────────────────────────┘
           │
           ▼
LAYER 5 — Phase 5 (requires Layer 4 complete)
  ┌──────────────────────────────────────────────────────────────────┐
  │  5.1 help formatter   (independent — uses ggen ontology)         │
  │  5.2 auto examples    (requires 1.1–1.4: features must exist)    │
  │  5.3 aliases          (independent — uses clap-noun-verb)        │
  │  5.4 JSON output      (requires all verbs: 1.1–4.5 done)        │
  │  5.5 shell REPL       (requires all verbs: commands to dispatch) │
  └──────────────────────────────────────────────────────────────────┘
```

### Critical Path (Longest Dependency Chain)

```
B.1 throughput → B.5 baselines → [done in Week 3]
2.1 model → 2.2 conform → 2.3 predict → [all done in Week 2]
4.1 trace → 4.2 metrics → 4.5 SLO → [all done in Week 4]
5.4 JSON output → 5.5 REPL → [all done in Week 5]
```

### Dependency Risk Table

| Feature | Requires | Risk if Delayed |
|---------|----------|-----------------|
| 2.2 conform | 2.1 model | Can't compute fitness without discovered model |
| 2.3 predict | 2.1 model | Prediction needs discovered process model |
| 2.5 goto-def | 2.4 hover | LSP server must be scaffolded first |
| 4.2 metrics | 4.1 trace | Prometheus pipeline requires OTel tracer |
| 4.5 SLO | 4.2 metrics | SLI can't be computed without metrics |
| 5.4 JSON | all verbs | Can't add `--format=json` to verbs that don't exist |
| 5.5 REPL | all verbs | Shell dispatches to handlers that must exist |
| 5.2 examples | 1.1–1.4 | Examples are sourced from implemented features |
| B.2 variance | B.1 throughput | Variance analysis needs latency baseline |
| B.5 baselines | B.1 + B.4 | Before/after needs throughput + profile data |

---

## 5. Quality Gate Dashboard

> Updated at end of each phase. Target state = all green before release.

### CI/CD Quality Gates

| Gate | Command | Current | Target | Notes |
|------|---------|---------|--------|-------|
| Compile (debug) | `cargo build` | ✅ | ✅ | Passes on v26.6.14 |
| Compile (all features) | `cargo build --all-features` | 🔲 | ✅ | Run after Phase 1 complete |
| Test suite | `cargo test --all` | ✅ 59 tests | ✅ | Must stay green throughout |
| Clippy | `cargo clippy --all-targets --all-features` | ✅ | ✅ | `#![deny(clippy::print_stdout)]` in effect |
| Fmt check | `cargo fmt --check` | ✅ | ✅ | Applied on all commits |
| Bench | `cargo bench --bench receipt_operations` | ✅ ~2.3µs | ✅ | Basic harness; full suite pending |
| Full bench suite | `cargo bench` | 🔲 | ✅ | After B.1–B.5 complete |
| Doc build | `cargo doc --all --no-deps` | 🔲 | ✅ | Verify no broken links |
| Coverage | `cargo tarpaulin --all-features` | 🔲 | ≥80% | Run after Phase 3 |

### Mutation Testing Quality Gate

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| Mutation kill rate | — | ≥90% | 🔲 (3.1 not yet implemented) |
| Mutants generated per run | — | 10–100 | 🔲 |
| Kill time per mutant | — | <1s | 🔲 |
| Surviving mutants (leaked) | — | ≤10% | 🔲 |

### Property Testing Quality Gate

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| Random receipts tested per run | — | 100 | 🔲 (3.4 not yet implemented) |
| Panics on random input | — | 0 | 🔲 |
| Property: verifier always decidable | — | PASS | 🔲 |
| Property: tampered receipts always fail | — | PASS | 🔲 |
| Property: fixture receipts always pass | — | PASS | 🔲 |

### E2E Suite Pass/Fail Dashboard

| Suite | File | Features | Status | Pass Rate |
|-------|------|----------|--------|-----------|
| Suite 1: Inspection | `tests/e2e_inspection.rs` | 1.1–1.5 | 🔲 not written | —/6 assertions |
| Suite 2: Discovery | `tests/e2e_discovery.rs` | 2.1–2.5 | 🔲 not written | —/5 assertions |
| Suite 3: Benchmarking | `tests/e2e_benchmarking.rs` | B.1–B.5 | 🔲 not written | —/6 assertions |
| Suite 4: Mutation | `tests/e2e_mutation.rs` | 3.1–3.5 | 🔲 not written | —/6 assertions |
| Suite 5: Observability | `tests/e2e_observability.rs` | 4.1–4.5 | 🔲 not written | —/7 assertions |
| Suite 6: CLI | `tests/e2e_cli.rs` | 5.1–5.5 | 🔲 not written | —/7 assertions |

### Coverage Gate

| Target | Method | Current | Required |
|--------|--------|---------|----------|
| Handler functions | `cargo tarpaulin` | 🔲 unknown | ≥80% |
| New modules (mining, mutation, metrics) | `cargo tarpaulin` | 🔲 not yet written | ≥80% |
| Feature-gated code | `cargo tarpaulin --all-features` | 🔲 unknown | ≥70% |
| E2E test paths | Manual assertion audit | 🔲 not written | 100% of assertions documented |

### Conformance Quality Gate (Phase 2)

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| Fitness score on clean linear receipt | — | ≥0.90 | 🔲 |
| Violated transitions on conformant receipt | — | 0 | 🔲 |
| Next-activity prediction confidence range | — | [0.0, 1.0] | 🔲 |
| Model transitions for 4-event linear receipt | — | 4 | 🔲 |

### OTel Quality Gate (Phase 4)

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| Spans emitted per verify call | 1 (basic) | ≥7 (one per stage) | 🔲 |
| Span parent-child chains valid | 🔲 | 100% | 🔲 |
| Prometheus metrics format valid | 🔲 | PASS | 🔲 |
| Baggage keys present | 🔲 | receipt_id, timestamp, version | 🔲 |
| SLO compliance (latency p99 <100ms) | 🔲 | PASS | 🔲 |

---

## 6. Risk Register

| ID | Risk | Probability | Impact | Mitigation Strategy | Status | Owner |
|----|------|------------|--------|---------------------|--------|-------|
| R-1 | wasm4pm nightly toolchain breakage | Medium | High | Feature-gate `discovery`/`predictive`; fall back to linear process discovery if blocked; pin toolchain version in `rust-toolchain.toml` | 🔲 Monitoring | Sean |
| R-2 | lsp-max symbol resolution issues | Medium | Medium | Phase in LSP features last within Phase 2; if blocked, ship 2.1–2.3 and defer 2.4–2.5 to Phase 2b; document LSP as optional | 🔲 Monitoring | Sean |
| R-3 | OTel Jaeger exporter failure (no running collector) | Low | Medium | Graceful degradation: warn if Jaeger unavailable; use in-memory exporter in tests; Jaeger target is CI-optional | 🔲 Monitoring | Sean |
| R-4 | Generated test code doesn't compile | Medium | High | Sandbox-compile generated code before emitting to user; test Tera templates with compile-fail harness (trybuild); use `compile_rust_code()` in E2E Suite 4 | 🔲 Monitoring | Sean |
| R-5 | Fixture database index corruption | Low | Medium | Rebuild index on corruption detection (checksum each record); keep raw JSON alongside index; add `FixtureDatabase::repair()` method | 🔲 Monitoring | Sean |
| R-6 | Performance regression in benchmarks (>10% slower) | Low | High | Set Criterion baselines before each phase; CI compares to baseline and blocks merge on >10% regression; alert in PR comment | 🔲 Monitoring | Sean |
| R-7 | clnrm mutation operators incompatible with Receipt schema | Medium | Medium | Evaluate clnrm API against Receipt fields in Week 0 pre-work; if incompatible, implement minimal `MutationOperator` trait locally (EventDrop, EventReorder, TypeChange, PayloadFlip) | 🔲 Monitoring | Sean |
| R-8 | quickcheck/rustyline not compatible with project deps | Low | Low | Both are dev-only deps; if blocked, use proptest instead of quickcheck; use readline directly instead of rustyline | 🔲 Monitoring | Sean |
| R-9 | Phase scope creep (adding features mid-sprint) | Medium | Medium | Freeze Phase N scope before starting; new ideas go to backlog; only P0/P1 items can unblock a gate | 🔲 Active | Sean |
| R-10 | E2E suite flakiness (timing, filesystem) | Low | Medium | Use tempfile for all test fixtures; no wall-clock assertions; all test data deterministic; mark non-deterministic tests as `#[ignore]` | 🔲 Monitoring | Sean |

---

## 7. Success Metrics Dashboard

### Core DX Metrics

| Metric | Description | Current Value | Target | Gap | Phase |
|--------|-------------|--------------|--------|-----|-------|
| Test code reduction | Lines per test: fixture-driven vs hand-written | ~50 lines/test (manual) | ≤5 lines/test (fixture) | 10x gap | Phase 1 |
| Feature discoverability | % of verbs accessible via `<TAB>` completion | 0% (no completion) | 100% | 100% gap | Phase 1.5 |
| Mutation feedback speed | Time to kill one mutant | — (not implemented) | <1s | — | Phase 3.1 |
| Example pass rate | % of auto-generated examples that run clean | 0% (no examples) | 100% | 100% gap | Phase 5.2 |
| Help accuracy | % of help text that matches ontology | ~60% (manual) | 100% | 40% gap | Phase 5.1 |

### Core QOL Metrics

| Metric | Description | Current Value | Target | Gap | Phase |
|--------|-------------|--------------|--------|-----|-------|
| Inspection verbs | Verbs available for receipt inspection | 2 (show, inspect) | 6 (+ diff, visualize, catalog, trace) | 4 verbs | Phase 1+4 |
| IDE integration | LSP hover + goto-def working | 0 (none) | 2 (hover + goto) | 2 features | Phase 2 |
| Real-time dashboards | Prometheus + Grafana panels live | 0 panels | ≥3 panels (throughput, error rate, latency) | 3 panels | Phase 4.2 |
| Shell completion targets | Shells supported (bash/zsh/fish) | 0 | 3 | 3 shells | Phase 1.5 |
| JSON output verbs | Verbs with `--format=json` | 0 | All verbs (≥10) | 10 verbs | Phase 5.4 |
| REPL commands | Commands available in `affi-shell` | 0 (no REPL) | ≥6 (load, inspect, diff, mutate, trace, help) | 6 commands | Phase 5.5 |

### Quality Metrics

| Metric | Description | Current Value | Target | Gap | Phase |
|--------|-------------|--------------|--------|-----|-------|
| Mutation kill rate | % of mutants rejected by verifier | — | ≥90% | — | Phase 3.1 |
| Property test receipts | Random receipts that all yield decidable verdict | 0 tested | 100/100 | — | Phase 3.4 |
| Conformance fitness | Fitness score for clean linear receipt | — | ≥0.90 | — | Phase 2.2 |
| Benchmark stability | Max allowed regression vs baseline | N/A | ≤10% | — | Benchmarking |
| E2E test suites passing | Suites that pass clean | 0/6 | 6/6 | 6 suites | All phases |
| Code coverage | % covered by `cargo tarpaulin` | Unknown | ≥80% | Unknown | Phase 3+ |

### Compound "1000x" Metric

| Factor | Current Multiplier | Target Multiplier | Mechanism |
|--------|-------------------|-------------------|-----------|
| Test code reduction | 1x | 10x | Fixture-driven tests (5 lines vs 50) |
| Feedback speed | 1x | 10x | Mutation kills in <1s vs manual review |
| Adoption ease | 1x | 10x | Completion + help + examples = self-documenting |
| Confidence | 1x | 10x | Conformance checks + benchmarks + property tests |
| **Combined** | **1x** | **10,000x** | **Conservative product** |

---

## 8. Weekly Sprint Summary

### Week 1 — Foundation (Phase 1: Receipt Inspection)

**Target:** 5 inspection features + B.1 throughput benchmark  
**Hours:** ~16h  
**Deliverable:** `affi receipt {inspect ✅, diff, visualize, catalog}` + shell completion + throughput bench

| Day | Task | Feature | Est Hours | Status |
|-----|------|---------|-----------|--------|
| Mon | ✅ Already done: inspect verb | 1.1 | 3h | ✅ Done |
| Mon | diff handler + DiffResult struct | 1.2 | 2h | 🔲 |
| Tue | visualize handler + graph builder (`src/graph_builder.rs`) | 1.3 | 4h | 🔲 |
| Tue | catalog handler + fixture registry | 1.4 | 2h | 🔲 |
| Wed | Shell completion (ggen.toml config + `src/bin/completion.rs`) | 1.5 | 1h | 🔲 |
| Wed | Throughput benchmark (`benches/throughput.rs`) | B.1 | 3h | 🔲 |
| Thu | Write `tests/e2e_inspection.rs` (all 6 assertions) | Suite 1 | 2h | 🔲 |
| Thu | Add missing Cargo.toml deps (quickcheck, rustyline) | Infra | 0.5h | 🔲 |
| Fri | Phase 1 integration run: `cargo test --all` green | Gate | 1h | 🔲 |
| Fri | Phase 1 sign-off review | Gate | 0.5h | 🔲 |

**Week 1 Phase Gate:** All Phase 1 exit criteria met; `cargo test --all` green.

---

### Week 2 — Process Intelligence (Phase 2: Discovery)

**Target:** 5 discovery features (model, conform, predict, LSP hover, LSP goto-def)  
**Hours:** ~18h  
**Deliverable:** `affi receipt {model, conform, predict}` + LSP hover/goto + variance bench

| Day | Task | Feature | Est Hours | Status |
|-----|------|---------|-----------|--------|
| Mon | `src/mining.rs`: receipt_to_ocel conversion | 2.1 (part) | 2h | 🔲 |
| Mon | wasm4pm HIM integration: Petri net from OCEL | 2.1 (part) | 2h | 🔲 |
| Tue | conform handler: alignment_fitness + violated_transitions | 2.2 | 2h | 🔲 |
| Tue | predict handler: next_activity + confidence | 2.3 | 2h | 🔲 |
| Wed | `src/lsp/mod.rs` + `hover.rs`: LSP hover scaffold | 2.4 | 3h | 🔲 |
| Thu | `src/lsp/goto_definition.rs`: event_type → handler source | 2.5 | 2h | 🔲 |
| Thu | Variance benchmark (`benches/variance.rs`) | B.2 | 2h | 🔲 |
| Fri | Write `tests/e2e_discovery.rs` (all 5 assertions) | Suite 2 | 2h | 🔲 |
| Fri | Phase 2 integration run + sign-off | Gate | 1h | 🔲 |

**Week 2 Phase Gate:** All Phase 2 exit criteria met; `affi receipt model` produces valid JSON.

---

### Week 3 — Quality Assurance (Phase 3: Mutation Testing + Benchmarking B.3–B.5)

**Target:** 5 mutation features + B.3/B.4/B.5 benchmarks  
**Hours:** ~18h  
**Deliverable:** `affi {generate, mutate}` + property tests + fixture DB + Criterion dashboard

| Day | Task | Feature | Est Hours | Status |
|-----|------|---------|-----------|--------|
| Mon | `src/mutation.rs`: EventDrop, EventReorder, TypeChange, PayloadFlip | 3.1 (part) | 2.5h | 🔲 |
| Mon | mutate handler + kill rate computation | 3.1 (part) | 2h | 🔲 |
| Tue | generate test codegen (Tera templates) | 3.2 | 2.5h | 🔲 |
| Tue | generate snippet handler + search | 3.3 | 1h | 🔲 |
| Wed | Property-based tests (`tests/property_based.rs`) + quickcheck Arbitrary | 3.4 | 3h | 🔲 |
| Wed | Criterion HTML dashboard (CSS theme + summary gen) | B.3 | 1.5h | 🔲 |
| Thu | `src/fixture_db.rs` + index builder | 3.5 | 2.5h | 🔲 |
| Thu | Profiling bench (`benches/profile.rs`) + flamegraph | B.4 | 2h | 🔲 |
| Thu | Baseline comparisons bench config | B.5 | 1.5h | 🔲 |
| Fri | Write `tests/e2e_mutation.rs` + `tests/e2e_benchmarking.rs` | Suites 3+4 | 2h | 🔲 |
| Fri | Phase 3 integration run + sign-off | Gate | 1h | 🔲 |

**Week 3 Phase Gate:** Kill rate ≥90%; 100 random receipts all decidable; `cargo bench` full suite green.

---

### Week 4 — Observability (Phase 4: OTel Stack)

**Target:** 5 OTel features (trace, metrics, baggage, span events, SLO)  
**Hours:** ~14h  
**Deliverable:** `affi receipt trace` + Prometheus metrics + Grafana dashboard + SLI/SLO

| Day | Task | Feature | Est Hours | Status |
|-----|------|---------|-----------|--------|
| Mon | Span emitter: 7 spans (one per verifier stage) | 4.1 | 2h | 🔲 |
| Mon | Jaeger export + in-memory test exporter | 4.1 | 1.5h | 🔲 |
| Tue | `src/metrics.rs`: throughput, error_rate, latency collectors | 4.2 | 2h | 🔲 |
| Tue | Prometheus export + `dashboards/affidavit.json` (Grafana) | 4.2 | 2h | 🔲 |
| Wed | OTel baggage: receipt_id, timestamp, version | 4.3 | 1h | 🔲 |
| Wed | Span events: one event per verifier stage lifecycle | 4.4 | 1h | 🔲 |
| Thu | SLI calculator: p99, error_rate%, availability% | 4.5 | 2h | 🔲 |
| Thu | SLO threshold comparison + error budget | 4.5 | 1h | 🔲 |
| Fri | Write `tests/e2e_observability.rs` (7 assertions) | Suite 5 | 2h | 🔲 |
| Fri | Phase 4 integration run + sign-off | Gate | 0.5h | 🔲 |

**Week 4 Phase Gate:** All OTel spans have parent-child chains; SLI within SLO; Grafana JSON valid.

---

### Week 5 — Polish (Phase 5: CLI Ergonomics)

**Target:** 5 CLI ergonomics features (help formatter, examples, aliases, JSON, REPL)  
**Hours:** ~13h  
**Deliverable:** Interactive shell REPL + JSON output + aliases + auto-examples + ontology-driven help

| Day | Task | Feature | Est Hours | Status |
|-----|------|---------|-----------|--------|
| Mon | Markdown→ASCII help formatter + ARDPRD cross-refs | 5.1 | 2h | 🔲 |
| Mon | Auto-generated examples from chicago-tdd fixtures | 5.2 | 2h | 🔲 |
| Tue | Example runner test harness (verify all examples exec) | 5.2 | 1h | 🔲 |
| Tue | Aliases in affi-cli.ttl ontology (`cnv:hasAlias "r"`) | 5.3 | 1h | 🔲 |
| Wed | JSON output formatters for all handlers (serde_json) | 5.4 | 2h | 🔲 |
| Wed | `--format=json` flag in all verb handlers | 5.4 | 1h | 🔲 |
| Thu | `src/bin/affi-shell.rs` REPL loop + rustyline | 5.5 | 2.5h | 🔲 |
| Thu | Shell command dispatcher (load, inspect, diff, etc.) | 5.5 | 1.5h | 🔲 |
| Fri | Write `tests/e2e_cli.rs` (7 assertions) | Suite 6 | 1.5h | 🔲 |
| Fri | Final integration run: ALL 6 suites green + sign-off | Gate | 0.5h | 🔲 |

**Week 5 Phase Gate:** All 30 features complete; all 6 E2E suites green; `cargo test --all-features` passes.

---

## 9. Definition of Done Global Contract

> A feature is **DONE** when **ALL** of the following are true. No exceptions.
> Partial completion does not count as DONE.

### Code Completeness

- [ ] **Implementation:** Handler function exists in `src/handlers.rs` (or appropriate module)
- [ ] **Module:** New source module(s) created if required by design (e.g., `src/mining.rs`, `src/mutation.rs`)
- [ ] **CLI wiring:** Verb registered in ontology (`ontology/affi-cli.ttl`) AND renders correctly via ggen
- [ ] **Feature gate:** If applicable, `#[cfg(feature = "X")]` guards in place and Cargo.toml feature declared
- [ ] **Error handling:** All fallible operations use `Result<T, E>`; no `.unwrap()` in non-test code
- [ ] **No bare println!:** All output goes through the designated output path (not `println!` macro — `#![deny(clippy::print_stdout)]` must pass)

### Test Completeness

- [ ] **Unit test:** At least 1 unit test for the handler function (in module `#[cfg(test)]` block)
- [ ] **Integration test:** Feature is exercised by at least 1 assertion in its E2E suite
- [ ] **Compile-fail test:** If feature introduces a new type invariant, a trybuild test proves the invariant is enforced
- [ ] **Feature gate test:** If feature-gated, a test with `#[cfg(feature = "X")]` exercises the gate
- [ ] **`cargo test --all` passes:** No test regressions introduced

### Quality Completeness

- [ ] **`cargo clippy --all-targets --all-features`:** Zero new warnings or errors
- [ ] **`cargo fmt --check`:** Code is formatted per project style
- [ ] **No panics:** Handler does not panic on any valid input (property tested if applicable)
- [ ] **`cargo build --all-features`:** Compiles clean with all features enabled

### Documentation Completeness

- [ ] **Rust doc comment:** `///` doc comment on the public handler function (at minimum: what it does, returns)
- [ ] **CLI help text:** Verb has a help string in the ontology that renders in `affi <noun> <verb> --help`
- [ ] **Example:** Feature is demonstrable via an example (either in `examples/` or in the E2E test comment block)
- [ ] **Status updated:** This DOD_MASTER.md status column updated to ✅ with actual hours logged

### Admission Completeness (per affidavit Honest Residuals doctrine)

- [ ] **Failing-when-fake witness:** A test exists that FAILS if the feature is removed or stubbed out
  - Removing the handler → a test fails to compile or produces wrong output
  - Faking the capability → a test assertion catches the fake
- [ ] **No hollow stamps:** The feature is not "marked done" because a function exists — it is done because the behavior is witnessed by a test that would fail without it

### Phase Gate Completeness

- [ ] **All features in phase are DONE** before the phase gate is signed off
- [ ] **E2E suite for the phase passes** (the suite specific to that phase's features)
- [ ] **Sign-off recorded** in the [Sign-off Table](#10-sign-off-table) by the responsible owner

---

## 10. Sign-off Table

> Each phase must be signed off before the next phase begins.
> Sign-off means: all features in the phase meet the DoD Global Contract (Section 9).

### Phase Sign-off Status

| Phase | Features | E2E Suite | Hours | Signed Off By | Date | Status |
|-------|---------|-----------|-------|--------------|------|--------|
| Phase 1: Receipt Inspection | 1.1–1.5 (5 features) | Suite 1 | ~16h | Sean Chatman | — | 🔲 Pending |
| Benchmarking (B.1 entry) | B.1 throughput (concurrently with Phase 1) | Suite 3 (partial) | 3h | Sean Chatman | — | 🔲 Pending |
| Phase 2: Process Discovery | 2.1–2.5 (5 features) | Suite 2 | ~18h | Sean Chatman | — | 🔲 Pending |
| Benchmarking (B.2–B.3) | B.2 variance, B.3 dashboard | Suite 3 (partial) | 5h | Sean Chatman | — | 🔲 Pending |
| Phase 3: Mutation Testing | 3.1–3.5 (5 features) | Suite 4 | ~18h | Sean Chatman | — | 🔲 Pending |
| Benchmarking (B.4–B.5) | B.4 profile, B.5 baselines | Suite 3 (complete) | 5h | Sean Chatman | — | 🔲 Pending |
| Phase 4: Observability | 4.1–4.5 (5 features) | Suite 5 | ~14h | Sean Chatman | — | 🔲 Pending |
| Phase 5: CLI Ergonomics | 5.1–5.5 (5 features) | Suite 6 | ~13h | Sean Chatman | — | 🔲 Pending |
| **Full Initiative** | All 30 features | All 6 suites | ~88h | Sean Chatman | — | 🔲 Pending |

### Final Release Criteria (All must be true before `v27.0.0` tag)

| Criterion | Status |
|-----------|--------|
| All 30 features DONE (DoD contract met for each) | 🔲 |
| All 6 E2E suites green (`cargo test --all`) | 🔲 |
| Mutation kill rate ≥90% | 🔲 |
| Property test: 100/100 receipts decidable | 🔲 |
| Conformance fitness ≥0.90 on clean receipt | 🔲 |
| Coverage ≥80% (`cargo tarpaulin --all-features`) | 🔲 |
| `cargo clippy --all-targets --all-features` zero warnings | 🔲 |
| `cargo fmt --check` passes | 🔲 |
| All auto-generated examples run clean | 🔲 |
| Criterion benchmark regression ≤10% from baseline | 🔲 |
| Shell completion covers 100% of verbs | 🔲 |
| All new public APIs have `///` doc comments | 🔲 |
| CHANGELOG.md updated with all 30 features | 🔲 |
| README.md updated with new verbs and examples | 🔲 |

---

## Appendix A: New Modules Required

| Module | Path | Phase | Purpose |
|--------|------|-------|---------|
| graph builder | `src/graph_builder.rs` | Phase 1 | DOT/JSON receipt graph export (visualize) |
| mining | `src/mining.rs` | Phase 2 | receipt → OCEL → Petri net (model, conform, predict) |
| lsp hover | `src/lsp/hover.rs` | Phase 2 | LSP textDocument/hover handler |
| lsp goto-def | `src/lsp/goto_definition.rs` | Phase 2 | LSP textDocument/definition handler |
| mutation | `src/mutation.rs` | Phase 3 | MutationOperator trait + operators (mutate) |
| fixture db | `src/fixture_db.rs` | Phase 3 | Persistent fixture storage + index |
| property tests | `tests/property_based.rs` | Phase 3 | quickcheck Arbitrary + property tests |
| metrics | `src/metrics.rs` | Phase 4 | OTel metrics + SLI computation |
| grafana dashboard | `dashboards/affidavit.json` | Phase 4 | Grafana JSON (Prometheus data source) |
| throughput bench | `benches/throughput.rs` | Phase 1/B | Criterion latency vs event count |
| variance bench | `benches/variance.rs` | Phase 2/B | Control-flow surprise metric |
| profile bench | `benches/profile.rs` | Phase 3/B | Flamegraph profiling harness |
| affi-shell | `src/bin/affi-shell.rs` | Phase 5 | Interactive REPL binary |
| completion bin | `src/bin/completion.rs` | Phase 1 | Shell completion script generator |

## Appendix B: Missing Dependencies to Add (Cargo.toml)

| Crate | Section | Required By | Status |
|-------|---------|------------|--------|
| `quickcheck` | `[dev-dependencies]` | 3.4 property-based | 🔲 Not yet added |
| `rustyline` | `[dev-dependencies]` | 5.5 shell REPL | 🔲 Not yet added |
| `prometheus` (or `opentelemetry-prometheus`) | `[dependencies]` (feature-gated) | 4.2 metrics dashboard | 🔲 Not yet added |
| `tera` | `[dependencies]` | 3.2 generate test | 🔲 Not yet added |

## Appendix C: Feature Gate Map

```toml
[features]
default = ["core"]
core = []                            # emit, assemble, verify, show, inspect
discovery = ["wasm4pm-compat"]       # model (2.1)
conformance = ["discovery"]          # conform (2.2)
predictive = ["conformance"]         # predict (2.3)
lsp = ["lsp-max"]                    # hover (2.4), goto-def (2.5)
mutation = []                        # mutate (3.1) — clnrm or local operators
otel = ["opentelemetry", "opentelemetry-jaeger"]  # trace (4.1)
metrics = ["otel"]                   # metrics dashboard (4.2), SLO (4.5)
json-output = []                     # JSON output for all verbs (5.4) — no extra deps
shell = ["rustyline"]                # REPL (5.5)
profiling = []                       # flamegraph bench (B.4) — requires perf
dev = ["discovery", "conformance", "predictive", "lsp", "mutation",
       "otel", "metrics", "json-output", "shell"]
```

---

*This document is the single source of truth for the affidavit DX/QOL 1000x initiative. Update it immediately when any feature status changes. Do not mark a feature ✅ until all criteria in Section 9 (DoD Global Contract) are satisfied.*
