# Definition of Done — Exit Gate Specification

**Project:** affidavit DX/QOL 1000x Initiative  
**Branch:** `claude/zen-cerf-oq87br`  
**Phases:** 5 | **Features:** 22 | **Estimated Hours:** 88  
**Governs:** When a feature, phase, or the full project is DONE  
**Version:** 1.0.0 | **Date:** 2026-06-14

---

## Doctrine

> **Certify, don't decide.**

This document applies the same principle to the initiative itself: it certifies whether work meets the standard. Gates are binary — a gate either passes or it does not. No partial credit. No "mostly done." A feature is not merged until its gate passes. A phase does not close until all its feature gates pass and the phase gate passes. The project does not ship until the final project exit gate passes.

---

## Table of Contents

1. [Global Merge Requirements](#1-global-merge-requirements)
2. [Feature-Level Exit Gate Template](#2-feature-level-exit-gate-template)
3. [Phase Exit Gates](#3-phase-exit-gates)
   - [Phase 1 — Inspection](#phase-1--inspection)
   - [Phase 2 — Discovery](#phase-2--discovery)
   - [Phase 3 — Mutation](#phase-3--mutation)
   - [Phase 4 — Observability](#phase-4--observability)
   - [Phase 5 — CLI](#phase-5--cli)
4. [Final Project Exit Gate](#4-final-project-exit-gate)
5. [Escalation Protocol](#5-escalation-protocol)
6. [Gate Verification Commands](#6-gate-verification-commands)

---

## 1. Global Merge Requirements

**Every merge to `main` — regardless of phase, feature, or size — must satisfy ALL of the following before a pull request may be merged. No exceptions.**

### 1.1 Compilation & Static Analysis

| # | Gate | Fail Condition | Command |
|---|------|---------------|---------|
| G-01 | `cargo fmt --check` passes | Any formatting deviation | `cargo fmt --check` |
| G-02 | `cargo clippy -- -D warnings` passes | Any warning promoted to error | `cargo clippy -- -D warnings` |
| G-03 | `cargo check --all-features` passes | Any compile error with any feature combination | `cargo check --all-features` |
| G-04 | `cargo check` (no features) passes | Default feature set fails to compile | `cargo check` |
| G-05 | No new compiler warnings | `RUSTFLAGS="-D warnings"` emits warnings not present on `main` | `RUSTFLAGS="-D warnings" cargo build` |

### 1.2 Test Suite

| # | Gate | Fail Condition | Command |
|---|------|---------------|---------|
| G-06 | `cargo test` passes — all tests | Any test failure | `cargo test` |
| G-07 | All 30+ tests pass (minimum count enforced) | Test count regresses below 30 | `cargo test 2>&1 \| grep "test result"` |
| G-08 | No panics in any test run | `panicked at` in test output | `cargo test 2>&1 \| grep -c "panicked at"` (must be 0) |
| G-09 | No `unwrap()` in non-test code | `unwrap()` call outside `#[cfg(test)]` block | `grep -rn "\.unwrap()" src/ --include="*.rs"` (must be 0) |
| G-10 | No `expect()` in non-test code | `expect(` call outside `#[cfg(test)]` block | `grep -rn "\.expect(" src/ --include="*.rs"` (must be 0) |

### 1.3 Documentation

| # | Gate | Fail Condition | Command / Check |
|---|------|---------------|---------|
| G-11 | `CHANGELOG.md` has entry for this change | No entry added under `[Unreleased]` | Manual review of `CHANGELOG.md` diff |
| G-12 | `CLAUDE.md` updated if architecture changed | New module, verb, or integration point undocumented | Manual review: new `src/` files have corresponding entries |
| G-13 | All public functions have doc comments | `cargo doc --no-deps 2>&1` reports missing docs | `RUSTDOCFLAGS="-D missing_docs" cargo doc --no-deps` |
| G-14 | `cargo doc --no-deps` builds without errors | Documentation build fails | `cargo doc --no-deps 2>&1 \| grep -c "error"` (must be 0) |

### 1.4 Code Quality

| # | Gate | Fail Condition | Command / Check |
|---|------|---------------|---------|
| G-15 | No `todo!()` macros in non-test code | `todo!()` in production code path | `grep -rn "todo!()" src/ --include="*.rs"` (must be 0) |
| G-16 | No `unimplemented!()` macros in non-test code | `unimplemented!()` in production path | `grep -rn "unimplemented!()" src/ --include="*.rs"` (must be 0) |
| G-17 | No `dbg!()` macros anywhere | Debug macro left in code | `grep -rn "dbg!(" src/ --include="*.rs"` (must be 0) |
| G-18 | No `println!` in library code | `println!` outside `src/bin/` | `grep -rn "println!" src/ --include="*.rs" \| grep -v "src/bin/"` (must be 0) |

### 1.5 PR Requirements

| # | Gate | Fail Condition |
|---|------|---------------|
| G-19 | PR references the feature ID (e.g., `P1-F1`) | PR description has no feature reference |
| G-20 | PR description states which phase exit gates were verified | Missing verification checklist in PR body |
| G-21 | CI passes on `claude/zen-cerf-oq87br` before merge to `main` | CI red at time of merge |
| G-22 | No force-push to `main` | Direct history rewrite on protected branch |

---

## 2. Feature-Level Exit Gate Template

**Copy this checklist into every feature PR. All boxes must be checked before the PR is mergeable.**

```
### Feature Exit Gate: [Feature Name] ([Feature ID])

**Phase:** [1–5]
**Estimated Hours:** [N]
**Handler / Module:** `src/[path].rs`

#### Implementation
- [ ] Handler implemented with correct signature matching trait or interface contract
- [ ] Handler registered in the appropriate dispatch table or CLI command tree
- [ ] All public types derive `Debug`, `Clone` where applicable
- [ ] No `unwrap()` or `expect()` in non-test code paths

#### Tests
- [ ] Unit test(s) pass: ≥ 1 test per public handler function
- [ ] Unit tests cover the happy path
- [ ] Unit tests cover at least 1 error/rejection path
- [ ] Test uses `assert!` / `assert_eq!` — not just "runs without panic"
- [ ] If the feature adds a new CLI verb: integration test exercises it end-to-end

#### Feature Gating
- [ ] Feature compiles with the feature flag enabled: `cargo check --features [flag]`
- [ ] Feature compiles without the feature flag: `cargo check` (no flag)
- [ ] Feature-gated code uses `#[cfg(feature = "...")]` consistently (no leakage)
- [ ] Dependencies behind the gate are listed under `[features]` in `Cargo.toml`

#### Error Handling
- [ ] All fallible operations propagate errors via `Result<T, E>`
- [ ] Error types are defined in `src/types.rs` or the relevant module (not ad hoc strings)
- [ ] Error messages are user-facing and actionable (no raw Rust panic messages)
- [ ] Admission gate validates input before processing (fail fast)

#### Documentation
- [ ] Doc comment (`///`) on every public function and type
- [ ] Doc comment includes an `# Errors` section if the function returns `Result`
- [ ] Doc comment includes an `# Examples` section or reference to `examples/`
- [ ] `CHANGELOG.md` entry added under `[Unreleased]`
- [ ] `CLAUDE.md` updated if a new module, verb, or integration point was added

#### Example
- [ ] Example file in `examples/[feature_name].rs` (or `examples/golden_run.sh` extended)
- [ ] Example runs without error: `cargo run --example [name]` exits 0
- [ ] Example output is human-readable and demonstrates the feature's value

#### Global Gates (re-verified for this feature)
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes (all tests, including newly added ones)
- [ ] `cargo check --all-features` passes
```

---

## 3. Phase Exit Gates

### Phase 1 — Inspection

**Features (5):** Receipt inspect, Receipt diff, Receipt visualize, Receipt catalog, Shell completion  
**Target tests:** `tests/e2e_inspection.rs`

#### 3.1.1 Feature-Level Prerequisite

All 5 features in Phase 1 must independently pass the [Feature-Level Exit Gate](#2-feature-level-exit-gate-template) before the phase gate is evaluated.

- [ ] P1-F1 (receipt inspect) feature gate passed
- [ ] P1-F2 (receipt diff) feature gate passed
- [ ] P1-F3 (receipt visualize) feature gate passed
- [ ] P1-F4 (receipt catalog) feature gate passed
- [ ] P1-F5 (shell completion) feature gate passed

#### 3.1.2 Integration Tests

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P1-I1 | `tests/e2e_inspection.rs` passes | All test functions in the file pass | `cargo test --test e2e_inspection` |
| P1-I2 | 0 broken pre-existing tests | `cargo test` count ≥ pre-phase-1 baseline | `cargo test 2>&1 \| grep "test result"` |
| P1-I3 | Baseline Criterion benchmarks within 10% | Receipt ops benchmarks do not regress | `cargo bench --bench receipt_operations 2>&1 \| grep "time:"` |

#### 3.1.3 CLI Functional Gates

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P1-C1 | `affi receipt inspect <path>` produces valid output | Exits 0, outputs JSON or human-readable with all 7 stage fields | `affi receipt inspect tests/fixtures/valid.json` |
| P1-C2 | `affi receipt diff <path1> <path2>` produces valid output | Exits 0, outputs structural diff | `affi receipt diff tests/fixtures/a.json tests/fixtures/b.json` |
| P1-C3 | `affi receipt visualize <path>` produces valid output | Exits 0, outputs DOT or Mermaid graph | `affi receipt visualize tests/fixtures/valid.json` |
| P1-C4 | `affi receipt catalog <dir>` produces valid output | Exits 0, lists all receipts in directory | `affi receipt catalog tests/fixtures/` |
| P1-C5 | `affi receipt inspect --format=json` is valid JSON | `jq .` parses the output without error | `affi receipt inspect tests/fixtures/valid.json --format=json \| jq .` |
| P1-C6 | `affi receipt diff --format=json` is valid JSON | `jq .` parses the output without error | `affi receipt diff tests/fixtures/a.json tests/fixtures/b.json --format=json \| jq .` |

#### 3.1.4 Shell Completion Gates

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P1-S1 | Bash completion script generates without error | Exits 0, produces non-empty output | `affi completion bash` |
| P1-S2 | Zsh completion script generates without error | Exits 0, produces non-empty output | `affi completion zsh` |
| P1-S3 | Fish completion script generates without error | Exits 0, produces non-empty output | `affi completion fish` |
| P1-S4 | Bash completion script is syntactically valid | `bash -n` accepts the script | `affi completion bash \| bash -n` |
| P1-S5 | Zsh completion script is syntactically valid | `zsh -n` accepts the script | `affi completion zsh \| zsh -n` |

#### 3.1.5 Phase 1 Sign-off Checklist

```
Phase 1 Sign-off — Inspection
Reviewer: ___________________  Date: ___________

- [ ] All 5 feature gates individually passed (P1-F1 through P1-F5)
- [ ] e2e_inspection.rs: all tests pass (P1-I1)
- [ ] 0 pre-existing test regressions (P1-I2)
- [ ] Criterion benchmarks within 10% baseline (P1-I3)
- [ ] All 4 subcommands produce valid output (P1-C1 through P1-C4)
- [ ] Both JSON-format subcommands parse with jq (P1-C5, P1-C6)
- [ ] All 3 shell completion scripts generate (P1-S1 through P1-S3)
- [ ] Bash and zsh scripts are syntactically valid (P1-S4, P1-S5)
- [ ] All Global Merge Requirements (G-01 through G-22) pass
```

---

### Phase 2 — Discovery

**Features (5):** Petri net model extraction, Conformance checking, LSP hover, OCEL adapter, WASM4PM feature gate  
**Target tests:** `tests/e2e_discovery.rs`

#### 3.2.1 Feature-Level Prerequisite

- [ ] P2-F1 (Petri net model) feature gate passed
- [ ] P2-F2 (conformance checking) feature gate passed
- [ ] P2-F3 (LSP hover) feature gate passed
- [ ] P2-F4 (OCEL adapter) feature gate passed
- [ ] P2-F5 (wasm4pm feature gate) feature gate passed

#### 3.2.2 Integration Tests

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P2-I1 | `tests/e2e_discovery.rs` passes | All test functions in the file pass | `cargo test --test e2e_discovery` |
| P2-I2 | 0 broken pre-existing tests | Superset of Phase 1 tests still pass | `cargo test` |
| P2-I3 | wasm4pm feature compiles in isolation | `--features wasm4pm` compiles | `cargo check --features wasm4pm` |
| P2-I4 | Default build excludes wasm4pm | No wasm4pm symbols in default binary | `cargo check` (no features) |

#### 3.2.3 Model Extraction Gates

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P2-M1 | `affi receipt model <path>` exits 0 | Command succeeds | `affi receipt model tests/fixtures/valid.json` |
| P2-M2 | Model output is valid JSON | `jq .` parses without error | `affi receipt model tests/fixtures/valid.json --format=json \| jq .` |
| P2-M3 | Model JSON has `transitions` field | Field present and non-empty array | `affi receipt model tests/fixtures/valid.json --format=json \| jq '.transitions \| length > 0'` (must be `true`) |
| P2-M4 | Model JSON has `places` field | Field present and non-empty array | `affi receipt model tests/fixtures/valid.json --format=json \| jq '.places \| length > 0'` (must be `true`) |
| P2-M5 | Each transition references valid place IDs | No dangling references | Automated test in `e2e_discovery.rs` |

#### 3.2.4 Conformance Gates

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P2-CF1 | `affi receipt conformance <path>` exits 0 for conforming receipt | Command succeeds | `affi receipt conformance tests/fixtures/valid.json` |
| P2-CF2 | Conformance fitness ≥ 0.9 for conforming receipts | Fitness score in output ≥ 0.9 | `affi receipt conformance tests/fixtures/valid.json --format=json \| jq '.fitness >= 0.9'` (must be `true`) |
| P2-CF3 | Conformance exits non-zero for non-conforming receipt | Non-zero exit code | `affi receipt conformance tests/fixtures/tampered.json; [ $? -ne 0 ]` |
| P2-CF4 | Conformance fitness < 0.5 for tampered receipt | Fitness score reflects violation | `affi receipt conformance tests/fixtures/tampered.json --format=json \| jq '.fitness < 0.5'` (must be `true`) |

#### 3.2.5 LSP Hover Gates

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P2-L1 | LSP module compiles with `lsp` feature | No compile errors | `cargo check --features lsp` |
| P2-L2 | LSP hover returns within 100ms | Unit test asserts latency bound | Test in `src/lsp.rs`: `assert!(elapsed < Duration::from_millis(100))` |
| P2-L3 | LSP hover returns event type for valid receipt path | Response contains `event_type` field | Unit test in `src/lsp.rs` |
| P2-L4 | LSP hover returns verification status | Response contains `verdict` field | Unit test in `src/lsp.rs` |

#### 3.2.6 OCEL Adapter Gates

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P2-O1 | `OcelAdapter::from_receipt_event` round-trips cleanly | No data loss in conversion | Unit test in `src/ocel.rs` |
| P2-O2 | OCEL output has `ocel:type` field | Required OCEL field present | Unit test asserts field |
| P2-O3 | OCEL output has `ocel:timestamp` field | Required OCEL field present | Unit test asserts field |
| P2-O4 | `examples/ocel_events.rs` runs without error | `cargo run --example ocel_events` exits 0 | `cargo run --example ocel_events` |

#### 3.2.7 Phase 2 Sign-off Checklist

```
Phase 2 Sign-off — Discovery
Reviewer: ___________________  Date: ___________

- [ ] All 5 feature gates individually passed (P2-F1 through P2-F5)
- [ ] e2e_discovery.rs: all tests pass (P2-I1)
- [ ] 0 pre-existing test regressions (P2-I2)
- [ ] wasm4pm feature compiles in isolation (P2-I3)
- [ ] Default build excludes wasm4pm symbols (P2-I4)
- [ ] Model output is valid JSON with transitions & places (P2-M1 through P2-M5)
- [ ] Conformance fitness ≥ 0.9 for conforming receipts (P2-CF2)
- [ ] Conformance correctly rejects tampered receipts (P2-CF3, P2-CF4)
- [ ] LSP hover responds within 100ms (P2-L2)
- [ ] LSP hover returns event_type and verdict (P2-L3, P2-L4)
- [ ] OCEL round-trip is lossless (P2-O1)
- [ ] OCEL output has required ocel: fields (P2-O2, P2-O3)
- [ ] ocel_events example runs without error (P2-O4)
- [ ] All Global Merge Requirements (G-01 through G-22) pass
```

---

### Phase 3 — Mutation

**Features (5):** Mutation testing harness, Generated test code, Property-based tests, Fixture database, Receipt repair  
**Target tests:** `tests/e2e_mutation.rs`, `tests/property_based.rs`

#### 3.3.1 Feature-Level Prerequisite

- [ ] P3-F1 (mutation testing harness) feature gate passed
- [ ] P3-F2 (generated test code) feature gate passed
- [ ] P3-F3 (property-based tests) feature gate passed
- [ ] P3-F4 (fixture database) feature gate passed
- [ ] P3-F5 (receipt repair) feature gate passed

#### 3.3.2 Integration Tests

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P3-I1 | `tests/e2e_mutation.rs` passes | All test functions in the file pass | `cargo test --test e2e_mutation` |
| P3-I2 | `tests/property_based.rs` passes on 100 random receipts | 100 proptest cases pass | `cargo test --test property_based` |
| P3-I3 | 0 broken pre-existing tests | Superset of Phase 1+2 tests still pass | `cargo test` |

#### 3.3.3 Mutation Testing Gates

| # | Gate | Pass Condition | Measurement |
|---|------|---------------|-------------|
| P3-MT1 | Mutation kill rate ≥ 90% | ≥ 9 of every 10 generated mutants are killed by existing tests | `cargo mutants 2>&1 \| grep "kill rate"` (≥ 90%) |
| P3-MT2 | Surviving mutants are documented | Each surviving mutant has a comment explaining why it survived | Manual review of `MUTATION_SURVIVORS.md` |
| P3-MT3 | Mutation harness itself has a test | The harness can generate and apply a mutation | Unit test in `src/verbs/mutate.rs` or dedicated test file |
| P3-MT4 | Mutations never corrupt the fixture DB | After mutation run, DB round-trips correctly | Post-mutation check in `e2e_mutation.rs` |

#### 3.3.4 Generated Test Code Gates

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P3-GT1 | Generated test code compiles | `rustc` accepts generated output without errors | Automated: `cargo test` includes generated test module |
| P3-GT2 | Generated tests pass | No failures in generated test suite | `cargo test generated_` (matches generated test prefix) |
| P3-GT3 | Generated code conforms to `cargo fmt` | `cargo fmt --check` on generated files passes | `cargo fmt --check` |
| P3-GT4 | Generated tests are deterministic | Same receipt → same test code on repeated runs | Unit test: run generator twice, assert identical output |

#### 3.3.5 Property-Based Test Gates

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P3-PB1 | 100 random receipts all verify correctly after assemble | 0 false positives / false negatives | `cargo test --test property_based -- --nocapture` |
| P3-PB2 | Any tampered receipt fails verification | Property: mutate any field → chain_hash mismatch | `cargo test property_tamper_always_fails` |
| P3-PB3 | Proptest regression cases are committed | Saved failing cases in `tests/proptest-regressions/` | File exists and is tracked in git |
| P3-PB4 | Property tests run in ≤ 60 seconds | Test suite time-bounded | `time cargo test --test property_based` (≤ 60s) |

#### 3.3.6 Fixture Database Gates

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P3-DB1 | Fixture DB insert + search round-trip | Inserted receipt is retrievable by chain_hash | Unit test in `src/verbs/catalog.rs` or fixture module |
| P3-DB2 | Fixture DB search by event_type works | Returns all receipts containing given event_type | Unit test: insert 3, search by type, get correct subset |
| P3-DB3 | Fixture DB handles 1000 records without performance regression | Search completes in < 500ms | Benchmark or assertion in test |
| P3-DB4 | Fixture DB is portable (no absolute paths in storage) | DB file relocatable | Integration test: move DB, open from new path |

#### 3.3.7 Receipt Repair Gates

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P3-R1 | Repair recomputes chain_hash correctly | Repaired receipt passes verify | `affi receipt repair tests/fixtures/broken_hash.json` && `affi verify <output>` |
| P3-R2 | Repair does NOT repair tampered event content | Tampered event content is not silently healed | Unit test: content mutation → repair → verify still fails |
| P3-R3 | Repair output is a valid receipt | `affi verify` exits 0 on repair output | Integration test in `e2e_mutation.rs` |
| P3-R4 | Repair is idempotent | Running repair twice produces identical output | `diff <(repair once) <(repair twice)` (empty diff) |

#### 3.3.8 Phase 3 Sign-off Checklist

```
Phase 3 Sign-off — Mutation
Reviewer: ___________________  Date: ___________

- [ ] All 5 feature gates individually passed (P3-F1 through P3-F5)
- [ ] e2e_mutation.rs: all tests pass (P3-I1)
- [ ] property_based.rs: 100 random receipts pass (P3-I2)
- [ ] 0 pre-existing test regressions (P3-I3)
- [ ] Mutation kill rate ≥ 90% (P3-MT1)
- [ ] Surviving mutants documented (P3-MT2)
- [ ] Generated test code compiles (P3-GT1)
- [ ] Generated tests pass (P3-GT2)
- [ ] Generated code is fmt-clean (P3-GT3)
- [ ] Generated tests are deterministic (P3-GT4)
- [ ] 100 random receipts verify correctly (P3-PB1)
- [ ] Tamper always fails property holds (P3-PB2)
- [ ] Proptest regressions committed (P3-PB3)
- [ ] Property tests complete in ≤ 60s (P3-PB4)
- [ ] Fixture DB round-trip passes (P3-DB1)
- [ ] Fixture DB search by event_type works (P3-DB2)
- [ ] Receipt repair recomputes chain_hash (P3-R1)
- [ ] Repair does not silently heal tampered content (P3-R2)
- [ ] Repair is idempotent (P3-R4)
- [ ] All Global Merge Requirements (G-01 through G-22) pass
```

---

### Phase 4 — Observability

**Features (5):** OTel spans for verifier stages, Prometheus metrics, SLI computation, Span hierarchy validation, OTel feature gate  
**Target tests:** `tests/e2e_observability.rs`

#### 3.4.1 Feature-Level Prerequisite

- [ ] P4-F1 (OTel spans for verifier stages) feature gate passed
- [ ] P4-F2 (Prometheus metrics) feature gate passed
- [ ] P4-F3 (SLI computation) feature gate passed
- [ ] P4-F4 (span hierarchy validation) feature gate passed
- [ ] P4-F5 (OTel feature gate isolation) feature gate passed

#### 3.4.2 Integration Tests

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P4-I1 | `tests/e2e_observability.rs` passes with `InMemorySpanExporter` | All test functions in the file pass | `cargo test --test e2e_observability --features otel` |
| P4-I2 | OTel code absent from default build | No OTel symbols without feature | `cargo check` (no features) succeeds |
| P4-I3 | 0 broken pre-existing tests | Superset of Phase 1+2+3 tests still pass | `cargo test` |
| P4-I4 | Tests pass with and without `otel` feature | No accidental feature coupling | `cargo test && cargo test --features otel` |

#### 3.4.3 Span Correctness Gates

| # | Gate | Pass Condition | Measurement |
|---|------|---------------|-------------|
| P4-S1 | All 7 verifier stage spans present | Span names match stage names exactly: `decode`, `check_format`, `chain_integrity`, `continuity`, `verify_commitments`, `evaluate_profile`, `emit_verdict` | `InMemorySpanExporter` assertion in `e2e_observability.rs` |
| P4-S2 | Stage spans have correct parent-child hierarchy | Each stage span's parent is the root `verify` span | Span parent assertion in `e2e_observability.rs` |
| P4-S3 | Root `verify` span wraps all stage spans | Root span start ≤ all stage starts; root span end ≥ all stage ends | Timing assertion in `e2e_observability.rs` |
| P4-S4 | Spans carry correct attributes | Each stage span has `receipt.path`, `stage.name`, `stage.passed` attributes | Attribute assertion in `e2e_observability.rs` |
| P4-S5 | Failed stages carry error status | REJECT verdict → failing stage span has `status = Error` | Error status assertion in `e2e_observability.rs` |
| P4-S6 | Spans are emitted in stage order | Span start times are monotonically non-decreasing | Order assertion in `e2e_observability.rs` |

#### 3.4.4 Prometheus Metrics Gates

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P4-P1 | Prometheus output format is valid | `promtool check metrics` accepts the output | `affi metrics --format=prometheus \| promtool check metrics` |
| P4-P2 | `verify_total` counter present | Metric name `affidavit_verify_total` in output | `affi metrics --format=prometheus \| grep "affidavit_verify_total"` |
| P4-P3 | `verify_accept_total` counter present | Metric name in output | `affi metrics --format=prometheus \| grep "affidavit_verify_accept_total"` |
| P4-P4 | `verify_reject_total` counter present | Metric name in output | `affi metrics --format=prometheus \| grep "affidavit_verify_reject_total"` |
| P4-P5 | `stage_duration_seconds` histogram present | Histogram with stage label in output | `affi metrics --format=prometheus \| grep "affidavit_stage_duration_seconds"` |
| P4-P6 | Metrics counters increment on each verify | Run verify twice, metrics increase by 1 | Integration test: `assert_eq!(metrics_after - metrics_before, 1)` |

#### 3.4.5 SLI Computation Gates

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P4-SLI1 | SLI returns value in range [0.0, 100.0] | Computed value within bounds | `affi sli --format=json \| jq '.value >= 0 and .value <= 100'` (must be `true`) |
| P4-SLI2 | SLI of 100% for all-ACCEPT run | After N accepts and 0 rejects, SLI = 100.0 | Integration test in `e2e_observability.rs` |
| P4-SLI3 | SLI of 0% for all-REJECT run | After 0 accepts and N rejects, SLI = 0.0 | Integration test in `e2e_observability.rs` |
| P4-SLI4 | SLI computation is deterministic | Same inputs → same SLI value | Unit test: assert idempotence |
| P4-SLI5 | SLI output is valid JSON with `value`, `window`, `total` fields | `jq` parses required fields | `affi sli --format=json \| jq '{value, window, total}'` (all non-null) |

#### 3.4.6 Feature Gate Isolation Gates

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P4-FG1 | `cargo build` (no features) has zero OTel import | `nm` / `grep` on binary finds no `opentelemetry` symbol | `cargo build && strings target/debug/affi \| grep -c "opentelemetry"` (must be 0) |
| P4-FG2 | `cargo build --features otel` compiles cleanly | No warnings or errors | `cargo build --features otel` |
| P4-FG3 | No OTel `use` statements outside `#[cfg(feature = "otel")]` blocks | No leakage into default code paths | `grep -rn "use opentelemetry" src/ \| grep -v "#\[cfg"` (must be 0 lines) |

#### 3.4.7 Phase 4 Sign-off Checklist

```
Phase 4 Sign-off — Observability
Reviewer: ___________________  Date: ___________

- [ ] All 5 feature gates individually passed (P4-F1 through P4-F5)
- [ ] e2e_observability.rs: all tests pass with otel feature (P4-I1)
- [ ] OTel code absent from default build (P4-I2)
- [ ] 0 pre-existing test regressions (P4-I3)
- [ ] Tests pass both with and without otel feature (P4-I4)
- [ ] All 7 verifier stage spans present (P4-S1)
- [ ] Stage spans have correct parent-child hierarchy (P4-S2)
- [ ] Root verify span wraps all stage spans (P4-S3)
- [ ] Spans carry required attributes (P4-S4)
- [ ] Failed stages have Error span status (P4-S5)
- [ ] Spans emitted in stage order (P4-S6)
- [ ] Prometheus format validates with promtool (P4-P1)
- [ ] All required metric names present (P4-P2 through P4-P5)
- [ ] Counters increment on each verify (P4-P6)
- [ ] SLI value is in [0.0, 100.0] (P4-SLI1)
- [ ] SLI boundary conditions correct (P4-SLI2, P4-SLI3)
- [ ] SLI is deterministic (P4-SLI4)
- [ ] SLI JSON has required fields (P4-SLI5)
- [ ] Zero OTel symbols in default binary (P4-FG1)
- [ ] No OTel use leakage outside cfg blocks (P4-FG3)
- [ ] All Global Merge Requirements (G-01 through G-22) pass
```

---

### Phase 5 — CLI

**Features (5):** Universal `--format=json`, Auto-generated examples, Shell REPL, Help text polish, Verb aliases  
**Target tests:** `tests/e2e_cli.rs`

#### 3.5.1 Feature-Level Prerequisite

- [ ] P5-F1 (universal `--format=json`) feature gate passed
- [ ] P5-F2 (auto-generated examples) feature gate passed
- [ ] P5-F3 (shell REPL) feature gate passed
- [ ] P5-F4 (help text polish) feature gate passed
- [ ] P5-F5 (verb aliases) feature gate passed

#### 3.5.2 Integration Tests

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P5-I1 | `tests/e2e_cli.rs` passes | All test functions in the file pass | `cargo test --test e2e_cli` |
| P5-I2 | 0 broken pre-existing tests | Superset of Phase 1–4 tests still pass | `cargo test` |

#### 3.5.3 Universal JSON Format Gates

Every verb that produces structured output MUST support `--format=json`. The following table defines the complete list:

| # | Verb | Gate | Command |
|---|------|------|---------|
| P5-J1 | `inspect` | `--format=json` produces valid JSON | `affi receipt inspect tests/fixtures/valid.json --format=json \| jq .` |
| P5-J2 | `diff` | `--format=json` produces valid JSON | `affi receipt diff tests/fixtures/a.json tests/fixtures/b.json --format=json \| jq .` |
| P5-J3 | `show` | `--format=json` produces valid JSON | `affi show tests/fixtures/valid.json --format=json \| jq .` |
| P5-J4 | `verify` | `--format=json` produces valid JSON | `affi verify tests/fixtures/valid.json --format=json \| jq .` |
| P5-J5 | `model` | `--format=json` produces valid JSON | `affi receipt model tests/fixtures/valid.json --format=json \| jq .` |
| P5-J6 | `conformance` | `--format=json` produces valid JSON | `affi receipt conformance tests/fixtures/valid.json --format=json \| jq .` |
| P5-J7 | All JSON outputs have a `status` field | Required top-level field present | Each command above: `\| jq '.status != null'` (all `true`) |
| P5-J8 | All JSON outputs are valid when piped | No ANSI color codes in JSON mode | `affi verify tests/fixtures/valid.json --format=json \| cat \| jq .` |

#### 3.5.4 Auto-Generated Examples Gates

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P5-E1 | All auto-generated examples exit code 0 | No example exits non-zero | `cargo run --examples 2>&1 \| grep -c "error"` (must be 0) |
| P5-E2 | Generated examples are reproducible | Running generator twice produces identical output | `diff <(generate-examples run-1) <(generate-examples run-2)` (empty) |
| P5-E3 | Generated examples are syntactically valid Rust | `rustfmt --check` accepts them | `cargo fmt --check` on generated example files |
| P5-E4 | Generated examples are committed to `examples/` | Files tracked in git | `git status examples/` shows no untracked generated files |

#### 3.5.5 Shell REPL Gates

The REPL must accept all 7 commands: `load`, `inspect`, `diff`, `mutate`, `trace`, `help`, `quit`.

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P5-R1 | REPL `load` command works | Loads a receipt and confirms | `echo "load tests/fixtures/valid.json" \| affi repl` |
| P5-R2 | REPL `inspect` command works | Prints inspection output | `echo -e "load tests/fixtures/valid.json\ninspect" \| affi repl` |
| P5-R3 | REPL `diff` command works | Prints diff between loaded and another receipt | `echo -e "load a.json\ndiff b.json" \| affi repl` |
| P5-R4 | REPL `mutate` command works | Applies mutation, shows result | `echo -e "load tests/fixtures/valid.json\nmutate" \| affi repl` |
| P5-R5 | REPL `trace` command works | Shows OTel-style trace output | `echo -e "load tests/fixtures/valid.json\ntrace" \| affi repl` |
| P5-R6 | REPL `help` command works | Prints help text listing all 7 commands | `echo "help" \| affi repl \| grep -c "quit"` (≥ 1) |
| P5-R7 | REPL `quit` command exits cleanly | Exit code 0 | `echo "quit" \| affi repl; [ $? -eq 0 ]` |
| P5-R8 | REPL handles unknown commands gracefully | Error message, no panic | `echo "notacommand" \| affi repl` (exits 0, prints error message) |
| P5-R9 | REPL does not exit on a single bad command | Session continues after error | `echo -e "notacommand\nhelp" \| affi repl \| grep -c "quit"` (≥ 1) |

#### 3.5.6 Help Text Gates

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P5-H1 | No `**bold**` markdown artifacts in help output | Zero `**` occurrences | `affi --help \| grep -c "\*\*"` (must be 0) |
| P5-H2 | No ` ``` ` code fence artifacts in help output | Zero triple-backtick occurrences | `affi --help \| grep -c '```'` (must be 0) |
| P5-H3 | No `[link](url)` markdown link artifacts in help output | Zero markdown link syntax | `affi --help \| grep -cP '\[.+\]\(.+\)'` (must be 0) |
| P5-H4 | All subcommands have non-empty `--help` output | Each subcommand's help is non-trivial | `affi receipt inspect --help \| wc -l` (≥ 5 lines) |
| P5-H5 | Help text checks apply to all verbs | No verb has stale or markdown-polluted help | Automated: iterate all subcommands in `e2e_cli.rs` |
| P5-H6 | Help text mentions `--format=json` for structured-output verbs | User-visible discoverability | `affi receipt inspect --help \| grep -c "format"` (≥ 1) |

#### 3.5.7 Alias Gates

| # | Gate | Pass Condition | Command |
|---|------|---------------|---------|
| P5-A1 | `affi r inspect <path>` routes to `affi receipt inspect <path>` | Outputs identical result | `diff <(affi r inspect tests/fixtures/valid.json) <(affi receipt inspect tests/fixtures/valid.json)` (empty diff) |
| P5-A2 | `affi r diff <p1> <p2>` routes to `affi receipt diff <p1> <p2>` | Outputs identical result | `diff <(affi r diff a.json b.json) <(affi receipt diff a.json b.json)` (empty diff) |
| P5-A3 | `affi r model <path>` routes to `affi receipt model <path>` | Outputs identical result | `diff <(affi r model tests/fixtures/valid.json) <(affi receipt model tests/fixtures/valid.json)` (empty diff) |
| P5-A4 | `affi r conform <path>` routes to `affi receipt conformance <path>` | Outputs identical result | `diff <(affi r conform tests/fixtures/valid.json) <(affi receipt conformance tests/fixtures/valid.json)` (empty diff) |
| P5-A5 | `affi r visualize <path>` routes to `affi receipt visualize <path>` | Outputs identical result | `diff <(affi r visualize tests/fixtures/valid.json) <(affi receipt visualize tests/fixtures/valid.json)` (empty diff) |
| P5-A6 | `affi r catalog <dir>` routes to `affi receipt catalog <dir>` | Outputs identical result | `diff <(affi r catalog tests/fixtures/) <(affi receipt catalog tests/fixtures/)` (empty diff) |
| P5-A7 | Alias help text lists the canonical form | `affi r --help` shows `receipt` is the full form | `affi r --help \| grep -c "receipt"` (≥ 1) |

#### 3.5.8 Phase 5 Sign-off Checklist

```
Phase 5 Sign-off — CLI
Reviewer: ___________________  Date: ___________

- [ ] All 5 feature gates individually passed (P5-F1 through P5-F5)
- [ ] e2e_cli.rs: all tests pass (P5-I1)
- [ ] 0 pre-existing test regressions (P5-I2)
- [ ] All 6 verbs support --format=json producing valid JSON (P5-J1 through P5-J6)
- [ ] All JSON outputs have a status field (P5-J7)
- [ ] No ANSI codes in JSON output (P5-J8)
- [ ] All auto-generated examples exit 0 (P5-E1)
- [ ] Generated examples are reproducible (P5-E2)
- [ ] Generated examples are fmt-clean (P5-E3)
- [ ] Generated examples are committed (P5-E4)
- [ ] REPL accepts all 7 commands: load, inspect, diff, mutate, trace, help, quit (P5-R1 through P5-R7)
- [ ] REPL handles unknown commands gracefully (P5-R8)
- [ ] REPL does not exit on a single bad command (P5-R9)
- [ ] 0 markdown artifacts (**) in help output (P5-H1)
- [ ] 0 markdown artifacts (```) in help output (P5-H2)
- [ ] 0 markdown link artifacts in help output (P5-H3)
- [ ] All subcommands have substantive --help output (P5-H4, P5-H5)
- [ ] Help text mentions --format=json for structured verbs (P5-H6)
- [ ] affi r inspect routes identically to affi receipt inspect (P5-A1)
- [ ] All 6 aliases route identically to their canonical forms (P5-A1 through P5-A6)
- [ ] Alias help shows canonical form (P5-A7)
- [ ] All Global Merge Requirements (G-01 through G-22) pass
```

---

## 4. Final Project Exit Gate

**The project is DONE when ALL of the following 52 conditions are simultaneously true. No conditions may be waived. No conditions may be "noted for follow-up." Every box must be checked.**

### 4.1 All Phases Complete

- [ ] FP-01 Phase 1 (Inspection) sign-off checklist fully checked
- [ ] FP-02 Phase 2 (Discovery) sign-off checklist fully checked
- [ ] FP-03 Phase 3 (Mutation) sign-off checklist fully checked
- [ ] FP-04 Phase 4 (Observability) sign-off checklist fully checked
- [ ] FP-05 Phase 5 (CLI) sign-off checklist fully checked

### 4.2 All 22 Features Complete

- [ ] FP-06 All 22 feature-level exit gate checklists are fully checked
- [ ] FP-07 All 22 features have at least 1 unit test each
- [ ] FP-08 All 22 features have at least 1 example demonstrating usage
- [ ] FP-09 All 22 features have doc comments on every public symbol

### 4.3 Test Suite Health

- [ ] FP-10 `cargo test` passes with total test count ≥ 30
- [ ] FP-11 All 4 e2e test files exist and pass: `e2e_inspection.rs`, `e2e_discovery.rs`, `e2e_mutation.rs`, `e2e_mutation.rs`, `e2e_observability.rs`, `e2e_cli.rs`
- [ ] FP-12 `tests/property_based.rs` passes on 100 random receipts
- [ ] FP-13 0 test failures across all test runs (lib + dispatch + e2e + UI)
- [ ] FP-14 0 panics in any test run
- [ ] FP-15 0 `unwrap()` / `expect()` calls in non-test production code
- [ ] FP-16 Mutation kill rate ≥ 90% (verified post Phase 3)
- [ ] FP-17 Proptest regression cases committed to `tests/proptest-regressions/`

### 4.4 CLI Completeness

- [ ] FP-18 All verbs in CLAUDE.md have working implementations: `emit`, `assemble`, `verify`, `show`, `inspect`, `diagnose`, `stats`, `graph`, `replay`, `model`, `conformance`
- [ ] FP-19 All 6 structured-output verbs support `--format=json` with valid JSON output
- [ ] FP-20 `affi r <subcommand>` alias works for all 6 receipt subcommands
- [ ] FP-21 Shell completion scripts generate for bash, zsh, and fish without error
- [ ] FP-22 Shell REPL accepts all 7 commands: `load`, `inspect`, `diff`, `mutate`, `trace`, `help`, `quit`
- [ ] FP-23 Help text for all verbs is free of markdown artifacts (`**`, ` ``` `, `[link]()`)
- [ ] FP-24 Exit codes are consistent: 0 = success/ACCEPT, 2 = REJECT, 1 = error

### 4.5 Compilation & Static Analysis

- [ ] FP-25 `cargo fmt --check` passes on the entire codebase
- [ ] FP-26 `cargo clippy -- -D warnings` passes with zero warnings
- [ ] FP-27 `cargo check --all-features` passes
- [ ] FP-28 `cargo check` (no features) passes
- [ ] FP-29 `RUSTFLAGS="-D warnings" cargo build` passes
- [ ] FP-30 `cargo doc --no-deps` builds without errors

### 4.6 Feature Gate Correctness

- [ ] FP-31 `cargo check --features otel` compiles cleanly; no OTel symbols in default binary
- [ ] FP-32 `cargo check --features lsp` compiles cleanly
- [ ] FP-33 `cargo check --features wasm4pm` compiles cleanly
- [ ] FP-34 Default build (`cargo check`) includes no feature-gated symbols

### 4.7 Observability Correctness

- [ ] FP-35 All 7 verifier stage spans present in OTel output (verified via `InMemorySpanExporter`)
- [ ] FP-36 Stage spans have correct parent-child hierarchy under root `verify` span
- [ ] FP-37 Prometheus metrics output validates with `promtool check metrics`
- [ ] FP-38 SLI computation returns values in [0.0, 100.0] and correctly reflects ACCEPT/REJECT history

### 4.8 Discovery & Conformance

- [ ] FP-39 `affi receipt model` produces valid Petri net JSON with non-empty `transitions` and `places`
- [ ] FP-40 Conformance fitness ≥ 0.9 for all receipts in `tests/fixtures/conforming/`
- [ ] FP-41 Conformance fitness < 0.5 for all receipts in `tests/fixtures/tampered/`
- [ ] FP-42 LSP hover responds with event details in ≤ 100ms (unit test)

### 4.9 Mutation & Property Testing

- [ ] FP-43 Fixture DB insert + search round-trips for 1000 records in < 500ms
- [ ] FP-44 Receipt repair correctly recomputes chain_hash for receipts with only hash corruption
- [ ] FP-45 Receipt repair does NOT heal tampered event content

### 4.10 Documentation

- [ ] FP-46 `CHANGELOG.md` has entries for all 22 features under appropriate version headers
- [ ] FP-47 `CLAUDE.md` architecture section reflects all new modules added during the initiative
- [ ] FP-48 `README.md` quick-start section still produces correct output when followed literally
- [ ] FP-49 All new `src/*.rs` modules are listed in `CLAUDE.md` High-Level Structure

### 4.11 Performance

- [ ] FP-50 Criterion benchmarks for `receipt_operations` are within 10% of pre-initiative baseline
- [ ] FP-51 `affi verify` on a 100-event receipt completes in < 100ms (measured in bench)
- [ ] FP-52 `affi assemble` on 100 events completes in < 75ms (measured in bench)

---

## 5. Escalation Protocol

### 5.1 Level 1 — Diagnose and Fix Locally

**Trigger:** A gate blocks; the cause appears to be a local implementation issue.  
**Time budget:** Up to 2 hours.

**Process:**

| Step | Action |
|------|--------|
| 1 | Run the failing gate command and capture full output |
| 2 | Run `affi diagnose <receipt>` if the failure involves a receipt |
| 3 | Run `cargo test -- --nocapture` to see verbose test output |
| 4 | Isolate the failing unit: `cargo test <test_name> -- --nocapture` |
| 5 | Fix and re-run the specific gate command to verify |
| 6 | Re-run the full phase gate checklist to confirm no regressions |
| 7 | If fixed within 2 hours, proceed with the PR |

**Do NOT escalate to Level 2 if:**
- The fix is a missing test
- The fix is a formatting issue (`cargo fmt`)
- The fix is a clippy lint
- The fix is a missing doc comment

### 5.2 Level 2 — Escalate to Architect

**Trigger:** The gate block cannot be resolved locally in 2 hours, OR the fix requires a design decision (e.g., changing a public API signature, restructuring a module, changing the verifier pipeline).

**Process:**

| Step | Action |
|------|--------|
| 1 | Open a GitHub Issue with label `gate-block` on `claude/zen-cerf-oq87br` |
| 2 | Include: gate ID, exact failure output, what was tried, what the design question is |
| 3 | Assign to the project architect (Sean Chatman) |
| 4 | Tag the PR as `blocked` — do NOT merge or bypass the gate |
| 5 | Architect responds with a design decision within 1 business day |
| 6 | Implement the decision, re-run the full gate, and resolve the issue |

**Issue template:**

```markdown
## Gate Block: [Gate ID]

**Phase:** [1–5]
**Feature:** [Feature ID and name]
**Gate:** [Exact gate name from this document]

### Failure Output

```
[Paste exact command and output here]
```

### What Was Tried

- [Attempt 1]
- [Attempt 2]

### Design Question

[State the decision that needs to be made]

### Options Considered

1. [Option A] — pros/cons
2. [Option B] — pros/cons

/cc @xpointsh
```

### 5.3 Level 3 — Feature Flag / Deferral

**Trigger:** A gate blocks due to an external dependency that is unavailable, broken, or blocked by a third-party release. The dependency cannot be resolved within the sprint.

**Process:**

| Step | Action |
|------|--------|
| 1 | Confirm the dependency block is external (not a local implementation gap) |
| 2 | Open a GitHub Issue with labels `gate-block` and `external-dependency` |
| 3 | Gate the feature behind a new feature flag in `Cargo.toml` |
| 4 | Ensure the default build compiles and passes all tests WITHOUT the feature |
| 5 | Document the deferral in `CHANGELOG.md` under `[Deferred]` |
| 6 | Update the Final Project Exit Gate: mark the condition as `DEFERRED (Issue #N)` |
| 7 | The feature is not counted toward the initiative's completion until the dependency resolves |
| 8 | When the dependency resolves, the feature re-enters the gate process from the feature-level gate |

**Deferral is NOT permitted for:**
- Global Merge Requirements (G-01 through G-22) — these have no exceptions
- Phase gates where the blocking feature is core to the phase's purpose
- The Final Project Exit Gate conditions FP-10 through FP-30 (compilation and test baseline)

**Deferral IS permitted for:**
- LSP integration if `lsp-max` has a breaking API change
- WASM4PM integration if the package manager is unavailable
- OTel spans if the upstream OTel SDK has a compatibility issue
- Prometheus metrics if `promtool` is not available in CI

---

## 6. Gate Verification Commands

**Reference:** Copy-paste these commands to verify each gate. All paths assume the project root is the working directory. All commands produce exit code 0 on pass.

### 6.1 Global Merge Requirements

```bash
# G-01: Formatting
cargo fmt --check

# G-02: Clippy
cargo clippy -- -D warnings

# G-03: All features compile
cargo check --all-features

# G-04: Default features compile
cargo check

# G-05: No new warnings
RUSTFLAGS="-D warnings" cargo build 2>&1 | grep -c "^warning:" # must be 0

# G-06: All tests pass
cargo test

# G-07: Test count >= 30
cargo test 2>&1 | grep "test result" | grep -oP '\d+ passed' | grep -oP '\d+' # must be >= 30

# G-08: No panics
cargo test 2>&1 | grep -c "panicked at" # must be 0

# G-09: No unwrap in non-test code
grep -rn "\.unwrap()" src/ --include="*.rs" | grep -v "#\[cfg(test)\]" | wc -l # must be 0

# G-10: No expect in non-test code
grep -rn "\.expect(" src/ --include="*.rs" | grep -v "#\[cfg(test)\]" | wc -l # must be 0

# G-13: All public functions have doc comments
RUSTDOCFLAGS="-D missing_docs" cargo doc --no-deps 2>&1 | grep -c "error" # must be 0

# G-14: Docs build without error
cargo doc --no-deps 2>&1 | grep -c "^error" # must be 0

# G-15: No todo!() in production code
grep -rn "todo!()" src/ --include="*.rs" | grep -v "#\[cfg(test)\]" | wc -l # must be 0

# G-16: No unimplemented!() in production code
grep -rn "unimplemented!()" src/ --include="*.rs" | grep -v "#\[cfg(test)\]" | wc -l # must be 0

# G-17: No dbg!()
grep -rn "dbg!(" src/ --include="*.rs" | wc -l # must be 0

# G-18: No println! in library code
grep -rn "println!" src/ --include="*.rs" | grep -v "src/bin/" | wc -l # must be 0
```

### 6.2 Phase 1 — Inspection

```bash
# P1-I1: e2e inspection tests
cargo test --test e2e_inspection

# P1-I2: No regressions
cargo test 2>&1 | grep "test result"

# P1-I3: Benchmarks within baseline
cargo bench --bench receipt_operations 2>&1 | grep "time:"

# P1-C1: inspect command
affi receipt inspect tests/fixtures/valid.json

# P1-C2: diff command
affi receipt diff tests/fixtures/a.json tests/fixtures/b.json

# P1-C3: visualize command
affi receipt visualize tests/fixtures/valid.json

# P1-C4: catalog command
affi receipt catalog tests/fixtures/

# P1-C5: inspect JSON
affi receipt inspect tests/fixtures/valid.json --format=json | jq .

# P1-C6: diff JSON
affi receipt diff tests/fixtures/a.json tests/fixtures/b.json --format=json | jq .

# P1-S1: bash completion
affi completion bash

# P1-S2: zsh completion
affi completion zsh

# P1-S3: fish completion
affi completion fish

# P1-S4: bash completion is valid
affi completion bash | bash -n

# P1-S5: zsh completion is valid
affi completion zsh | zsh -n
```

### 6.3 Phase 2 — Discovery

```bash
# P2-I1: e2e discovery tests
cargo test --test e2e_discovery

# P2-I3: wasm4pm feature compiles
cargo check --features wasm4pm

# P2-I4: wasm4pm absent from default
cargo check 2>&1 | grep -c "wasm4pm" # must be 0

# P2-M1: model command
affi receipt model tests/fixtures/valid.json

# P2-M2: model is valid JSON
affi receipt model tests/fixtures/valid.json --format=json | jq .

# P2-M3: model has transitions
affi receipt model tests/fixtures/valid.json --format=json | jq '.transitions | length > 0' # must be true

# P2-M4: model has places
affi receipt model tests/fixtures/valid.json --format=json | jq '.places | length > 0' # must be true

# P2-CF1: conformance on valid receipt
affi receipt conformance tests/fixtures/valid.json

# P2-CF2: fitness >= 0.9
affi receipt conformance tests/fixtures/valid.json --format=json | jq '.fitness >= 0.9' # must be true

# P2-CF3: non-zero exit for tampered
affi receipt conformance tests/fixtures/tampered.json; [ $? -ne 0 ]

# P2-CF4: fitness < 0.5 for tampered
affi receipt conformance tests/fixtures/tampered.json --format=json | jq '.fitness < 0.5' # must be true

# P2-L3, P2-L4: LSP hover fields (unit test)
cargo test --lib lsp

# P2-O4: OCEL example
cargo run --example ocel_events
```

### 6.4 Phase 3 — Mutation

```bash
# P3-I1: e2e mutation tests
cargo test --test e2e_mutation

# P3-I2: property-based tests
cargo test --test property_based

# P3-MT1: mutation kill rate
cargo mutants 2>&1 | grep "kill rate" # must show >= 90%

# P3-GT3: generated code is fmt-clean
cargo fmt --check

# P3-GT4: generator is deterministic
# (two runs produce identical output — implemented as a unit test)
cargo test test_generator_deterministic

# P3-PB3: proptest regressions exist
ls tests/proptest-regressions/ # must not be empty

# P3-PB4: property tests complete in time
time cargo test --test property_based # must be <= 60 seconds

# P3-DB1: fixture DB round-trip
cargo test test_fixture_db_roundtrip

# P3-DB3: fixture DB performance
cargo test test_fixture_db_1000_records

# P3-R1: repair then verify
REPAIRED=$(affi receipt repair tests/fixtures/broken_hash.json --out /tmp/repaired.json && echo /tmp/repaired.json)
affi verify "$REPAIRED"

# P3-R4: repair is idempotent
affi receipt repair tests/fixtures/broken_hash.json --out /tmp/r1.json
affi receipt repair tests/fixtures/broken_hash.json --out /tmp/r2.json
diff /tmp/r1.json /tmp/r2.json # must be empty
```

### 6.5 Phase 4 — Observability

```bash
# P4-I1: e2e observability tests (with otel feature)
cargo test --test e2e_observability --features otel

# P4-I2: OTel absent from default build
cargo build 2>&1 | grep -c "opentelemetry" # must be 0

# P4-I4: tests pass both ways
cargo test && cargo test --features otel

# P4-S1 through P4-S6: span tests (InMemorySpanExporter)
cargo test --test e2e_observability --features otel -- --nocapture

# P4-P1: prometheus format valid
affi metrics --format=prometheus | promtool check metrics

# P4-P2: verify_total metric
affi metrics --format=prometheus | grep "affidavit_verify_total"

# P4-P3: verify_accept_total metric
affi metrics --format=prometheus | grep "affidavit_verify_accept_total"

# P4-P4: verify_reject_total metric
affi metrics --format=prometheus | grep "affidavit_verify_reject_total"

# P4-P5: stage duration histogram
affi metrics --format=prometheus | grep "affidavit_stage_duration_seconds"

# P4-SLI1: SLI in range
affi sli --format=json | jq '.value >= 0 and .value <= 100' # must be true

# P4-SLI5: SLI has required fields
affi sli --format=json | jq '{value, window, total}' # all non-null

# P4-FG1: no OTel symbols in default binary
cargo build
strings target/debug/affi | grep -c "opentelemetry" # must be 0

# P4-FG2: otel feature builds
cargo build --features otel

# P4-FG3: no OTel use leakage
grep -rn "use opentelemetry" src/ | grep -v "#\[cfg(feature" | wc -l # must be 0
```

### 6.6 Phase 5 — CLI

```bash
# P5-I1: e2e CLI tests
cargo test --test e2e_cli

# P5-J1 through P5-J6: all verbs JSON valid
affi receipt inspect tests/fixtures/valid.json --format=json | jq .
affi receipt diff tests/fixtures/a.json tests/fixtures/b.json --format=json | jq .
affi show tests/fixtures/valid.json --format=json | jq .
affi verify tests/fixtures/valid.json --format=json | jq .
affi receipt model tests/fixtures/valid.json --format=json | jq .
affi receipt conformance tests/fixtures/valid.json --format=json | jq .

# P5-J8: no ANSI codes in JSON output
affi verify tests/fixtures/valid.json --format=json | cat | jq . # jq must succeed

# P5-E1: all generated examples exit 0
for example in examples/*.rs; do
  name=$(basename "$example" .rs)
  cargo run --example "$name" 2>&1 || echo "FAILED: $name"
done

# P5-R1 through P5-R7: REPL commands
echo "load tests/fixtures/valid.json" | affi repl
echo -e "load tests/fixtures/valid.json\ninspect" | affi repl
echo -e "load tests/fixtures/a.json\ndiff tests/fixtures/b.json" | affi repl
echo -e "load tests/fixtures/valid.json\nmutate" | affi repl
echo -e "load tests/fixtures/valid.json\ntrace" | affi repl
echo "help" | affi repl | grep -c "quit" # >= 1
echo "quit" | affi repl; [ $? -eq 0 ]

# P5-R8, P5-R9: REPL error handling
echo "notacommand" | affi repl # must not panic
echo -e "notacommand\nhelp" | affi repl | grep -c "quit" # >= 1

# P5-H1: no ** in help
affi --help | grep -c "\*\*" # must be 0

# P5-H2: no ``` in help
affi --help | grep -c '```' # must be 0

# P5-H3: no [link]() in help
affi --help | grep -cP '\[.+\]\(.+\)' # must be 0

# P5-A1 through P5-A6: aliases route identically
diff <(affi r inspect tests/fixtures/valid.json) <(affi receipt inspect tests/fixtures/valid.json)
diff <(affi r diff tests/fixtures/a.json tests/fixtures/b.json) <(affi receipt diff tests/fixtures/a.json tests/fixtures/b.json)
diff <(affi r model tests/fixtures/valid.json) <(affi receipt model tests/fixtures/valid.json)
diff <(affi r conform tests/fixtures/valid.json) <(affi receipt conformance tests/fixtures/valid.json)
diff <(affi r visualize tests/fixtures/valid.json) <(affi receipt visualize tests/fixtures/valid.json)
diff <(affi r catalog tests/fixtures/) <(affi receipt catalog tests/fixtures/)

# P5-A7: alias help mentions canonical
affi r --help | grep -c "receipt" # >= 1
```

### 6.7 Final Project Exit Gate

```bash
# FP-10: test count >= 30
cargo test 2>&1 | grep "test result" | grep -oP '\d+ passed'

# FP-11: all e2e test files exist
ls tests/e2e_inspection.rs tests/e2e_discovery.rs tests/e2e_mutation.rs tests/e2e_observability.rs tests/e2e_cli.rs

# FP-24: exit code consistency
affi verify tests/fixtures/valid.json; [ $? -eq 0 ]    # ACCEPT = 0
affi verify tests/fixtures/tampered.json; [ $? -eq 2 ] # REJECT = 2

# FP-30: docs build
cargo doc --no-deps 2>&1 | grep -c "^error" # must be 0

# FP-46: CHANGELOG has all 22 features
grep -c "P[1-5]-F[1-5]" CHANGELOG.md # must be >= 22

# FP-50: benchmarks within baseline
cargo bench --bench receipt_operations 2>&1 | grep "time:"

# FP-51: verify performance
cargo bench --bench receipt_operations -- verify_100_events 2>&1 | grep "time:" # must be < 100ms

# FP-52: assemble performance
cargo bench --bench receipt_operations -- assemble_100_events 2>&1 | grep "time:" # must be < 75ms

# Full sweep — run everything
cargo fmt --check && \
cargo clippy -- -D warnings && \
cargo check --all-features && \
RUSTFLAGS="-D warnings" cargo build && \
cargo test && \
cargo doc --no-deps
```

---

*This document is the authoritative specification for what "done" means on the affidavit DX/QOL 1000x initiative. Any ambiguity in a gate's pass condition is resolved by re-reading the gate's stated criterion literally and implementing a test that makes the criterion unambiguous. If the test itself becomes the question, escalate to Level 2.*

---

**Document owner:** Sean Chatman (xpointsh@gmail.com)  
**Last updated:** 2026-06-14  
**Branch:** `claude/zen-cerf-oq87br`  
**Status:** ACTIVE — governs all merges to `main`
