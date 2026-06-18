# Definition of Done — Business Acceptance Tests & UAT Specification

**Project:** affidavit DX/QOL 1000x Initiative  
**Branch:** `claude/zen-cerf-oq87br`  
**Version:** 26.6.17  
**Date:** 2026-06-14  
**Doctrine:** Certify, don't decide.

---

## Table of Contents

1. [Acceptance Test Framework](#1-acceptance-test-framework)
2. [UAT Scenarios — 6 Test Suites](#2-uat-scenarios--6-test-suites)
   - [Suite 1: Inspection](#suite-1-inspection--visualization)
   - [Suite 2: Discovery](#suite-2-process-discovery--ide-integration)
   - [Suite 3: Benchmarking](#suite-3-benchmarking--regression-detection)
   - [Suite 4: Mutation](#suite-4-mutation-testing--fixtures)
   - [Suite 5: Observability](#suite-5-observability--tracing)
   - [Suite 6: CLI Ergonomics](#suite-6-cli-ergonomics)
3. [Acceptance Test Coverage Matrix](#3-acceptance-test-coverage-matrix)
4. [Non-Functional Acceptance Tests](#4-non-functional-acceptance-tests)
5. [Regression Test Scenarios](#5-regression-test-scenarios)
6. [Adversarial Test Scenarios](#6-adversarial-test-scenarios)
7. [Test Data Catalog](#7-test-data-catalog)
8. [Acceptance Signoff Requirements](#8-acceptance-signoff-requirements)

---

## 1. Acceptance Test Framework

### 1.1 Test Environment Requirements

| Requirement | Specification | Verification Command |
|---|---|---|
| Rust toolchain | Nightly (pinned in `rust-toolchain.toml`) | `rustc --version` → `rustc 1.98.0-nightly` or later |
| Cargo | Ships with Rust nightly | `cargo --version` |
| Binary available | `affi` builds from workspace root | `cargo build --bin affi` exits 0 |
| Network access | Not required for any test | Tests use `tempfile::TempDir`; all fixtures are in-process |
| Disk space | ≥ 50 MB free for temp dirs and benchmark artifacts | `df -h /tmp` |
| OS | Linux x86_64 (CI) or macOS aarch64 (dev) | `uname -sm` |
| Environment variables | None required; `RUST_LOG` optional for debug output | — |

All integration tests use `tempfile::TempDir` for isolation. No test writes to the repository working tree. No test requires a running service (Jaeger, Prometheus, Grafana) unless the `otel` feature is explicitly enabled.

### 1.2 Test Data Requirements

#### Canonical Receipt Fixtures

These fixtures are constructed in-process by helper functions. No fixture files are committed to the repository; all fixtures are deterministic given the same inputs.

| Fixture Name | Construction Method | Event Count | Expected `chain_hash` stability |
|---|---|---|---|
| `linear-3` | `build_receipt_n(3)` via `ChainAssembler` | 3 | Stable across runs (determinism) |
| `linear-1` | `build_receipt_n(1)` | 1 | Stable |
| `linear-10` | `build_receipt_n(10)` | 10 | Stable |
| `lifecycle` | emit(create) + emit(transform) + emit(release) via CLI | 3 | Stable |
| `tampered` | `linear-3` with `events[1].event_type` mutated post-assembly | 3 | Chain breaks on tamper |
| `seq-gap` | `linear-3` with `events[1].seq = 5` injected | 3 | Continuity fails |
| `dup-id` | `linear-3` with `events[1].id = events[0].id` | 3 | Continuity fails |
| `bad-commitment` | `linear-3` with `events[0].payload_commitment = "short"` | 3 | verify_commitments fails |
| `wrong-version` | `linear-3` with `format_version = "2.0"` | 3 | check_format fails |
| `empty-events` | `ChainAssembler::new().finalize()` — zero events | 0 | Continuity fails (no events) |
| `objectless` | Event with `objects: []` manually serialized | 1 | OCEL court refuses |

#### Fixture Construction Pattern

```rust
fn build_receipt_n(n: usize) -> Receipt {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    for i in 0..n {
        let ev = build_event(
            &format!("op-{i}"),
            vec![object_ref(&format!("obj-{i}"), "artifact")],
            format!("payload-{i}").as_bytes(),
            &mut counter,
        ).expect("build event");
        asm.append(ev).expect("append");
    }
    asm.finalize()
}
```

#### Tampered Receipt Construction Pattern

```rust
fn tampered_receipt() -> Vec<u8> {
    let receipt = build_receipt_n(3);
    let json = serde_json::to_string(&receipt).expect("serialize");
    json.replace("\"op-1\"", "\"TAMPERED\"").into_bytes()
}
```

### 1.3 Test Runner Configuration

```toml
# In Cargo.toml [dev-dependencies], required for all UAT suites
assert_cmd = "2"
predicates = "3"
tempfile = "3"
criterion = { version = "0.5", features = ["html_reports"] }
opentelemetry = { version = "0.20", features = ["trace"] }
chicago-tdd-tools = { path = "../chicago-tdd-tools" }
```

**Run all tests:**
```bash
cargo test --all
```

**Run specific suite:**
```bash
cargo test --test e2e                  # Core lifecycle
cargo test --test adversarial          # Adversarial
cargo test --test dx_verbs_e2e         # DX verbs
cargo test --test dx_full_pipeline_e2e # Full DX pipeline
cargo test --test dx_introspect_e2e    # Introspection/LLM surface
cargo test --test chicago_tdd_witness  # chicago-tdd integration
cargo test --test otel_witness         # OTel span emission
```

**Run benches:**
```bash
cargo bench --bench receipt_operations
```

**Single-threaded determinism run:**
```bash
cargo test -- --test-threads=1
```

**With debug output:**
```bash
RUST_LOG=debug cargo test -- --nocapture
```

### 1.4 Pass/Fail Criteria Interpretation

| Exit Code | Test Framework Assertion | Meaning |
|---|---|---|
| 0 | `assert().success()` | Command completed and produced expected output |
| Non-zero | `assert().failure()` | Command correctly rejected invalid input |
| 2 (verify) | `assert().code(2)` or `assert().failure()` | Verifier returned REJECT verdict |

**Stderr vs. Stdout policy:** The `affi` binary routes all human-readable output to **stderr** (enforced via `#![deny(clippy::print_stdout)]` at library root). Machine-parseable output (`--introspect` JSON schema) routes to **stdout**. UAT assertions must match this routing or they will fail silently.

**String matching:** All `predicate::str::contains(...)` assertions match literal substrings. Where a test specifies `"exact string"`, the assertion uses `predicate::str::is_equal_to(...)`. Regex is used only where noted.

**ACCEPT vs. REJECT exit mapping:**
- `affi receipt verify` exits `0` on ACCEPT
- `affi receipt verify` calls `std::process::exit(2)` on REJECT (from `handlers::verify`)
- All other `affi` subcommands exit `0` on success

---

## 2. UAT Scenarios — 6 Test Suites

---

### Suite 1: Inspection & Visualization

**Scope:** `affi receipt inspect`, `affi receipt diff`, `affi receipt visualize`, `affi receipt catalog`, shell completion  
**Test File:** `tests/dx_verbs_e2e.rs`, `tests/dx_full_pipeline_e2e.rs`  
**Library integration:** `chicago-tdd-tools` (fixture builders + assertion macros)

---

#### UAT-INS-001: Inspect Reports Event Type Distribution

**User Story:** As a developer, I want `affi receipt inspect <path>` to show me the count of each event type so that I can understand what operations a receipt records without manually reading JSON.

**Pre-conditions:**
- `affi` binary is built (`cargo build --bin affi`)
- A temp directory is available
- No `.affi/working.json` exists in the temp directory

**Test Procedure:**

```bash
# Step 1: Emit three events with distinct types
affi receipt emit --type create --object f:artifact --payload - <<< "file content"
# Expected stdout: "emitted event evt-0"

affi receipt emit --type transform --object d:artifact --payload - <<< "data transformation"
# Expected stdout: "emitted event evt-1"

affi receipt emit --type release --object f:artifact --payload - <<< "release"
# Expected stdout: "emitted event evt-2"

# Step 2: Assemble receipt
affi receipt assemble --out r.json
# Expected stdout contains: "assembled receipt ->"
# Expected stdout contains: "content address:"

# Step 3: Inspect the receipt
affi receipt inspect r.json
```

**Expected Output (stderr, exact substrings):**

```
RECEIPT INSPECTION REPORT
create: 1 events
transform: 1 events
release: 1 events
artifact:
```

**Pass Criteria:**
- Exit code: `0`
- stderr contains `"RECEIPT INSPECTION REPORT"`
- stderr contains `"create: 1 events"`
- stderr contains `"artifact:"`

**Fail Diagnosis:**
- If `verb not found`: `inspect` was not registered with the `#[verb]` macro or ggen re-render failed
- If exit code non-zero but no panic: check `handlers::inspect` for error propagation gap
- If output on stdout instead of stderr: `clippy::print_stdout` guard was bypassed

---

#### UAT-INS-002: Inspect Aggregates Object Type Coverage

**User Story:** As a developer, I want inspect to show me object type coverage so that I can verify all expected artifact types are referenced in a receipt.

**Pre-conditions:** Same as UAT-INS-001. Receipt assembled with two distinct object types.

**Test Procedure:**

```bash
affi receipt emit --type build --object src:source-file:input --payload - <<< "source"
affi receipt emit --type test  --object bin:binary:output    --payload - <<< "binary"
affi receipt assemble --out typed.json
affi receipt inspect typed.json
```

**Expected Output (stderr):**

```
RECEIPT INSPECTION REPORT
build: 1 events
test: 1 events
source-file:
binary:
```

**Pass Criteria:**
- Exit code: `0`
- stderr contains both `"source-file:"` and `"binary:"`
- No duplicate counting of object types

**Fail Diagnosis:**
- If object types are missing: `handlers::inspect` is reading only `events[0].objects`, not iterating all events

---

#### UAT-INS-003: Inspect on Single-Event Receipt

**User Story:** As a developer, I want inspect to work on the minimal receipt (one event) so that it handles edge cases without panicking.

**Test Procedure:**

```bash
affi receipt emit --type init --object svc:service --payload - <<< "init"
affi receipt assemble --out single.json
affi receipt inspect single.json
```

**Expected Output (stderr):**

```
RECEIPT INSPECTION REPORT
init: 1 events
service:
```

**Pass Criteria:**
- Exit code: `0`
- No `thread 'main' panicked` in stderr
- Report contains exactly one event-type line

---

#### UAT-INS-004: Inspect via Full DX Pipeline (Chicago-TDD Integration)

**User Story:** As a developer, I want the chicago-tdd-tools `assert_ok!` macro to be usable with inspection results, confirming library integration at the assertion layer.

**Test Procedure (Rust test using `chicago-tdd-tools`):**

```rust
// tests/chicago_tdd_witness.rs pattern
use chicago_tdd_tools::{assert_ok, assert_in_range};
use affidavit::admission::admit;

let receipt = build_receipt_n(3);
assert_in_range!(receipt.events.len(), 1, 10);
let result = admit(receipt);
assert_ok!(result, "honest receipt must pass the structural law");
```

Run: `cargo test chicago_tdd_asserts_honest_receipt_is_admitted`

**Expected Output:**

```
test chicago_tdd_asserts_honest_receipt_is_admitted ... ok
```

**Pass Criteria:**
- Test compiles (proves `chicago-tdd-tools` is a real dependency, not a stub)
- Test passes
- Removing `chicago-tdd-tools` from `Cargo.toml` causes a compile error (not just test failure)

---

#### UAT-INS-005: Introspect Emits JSON Schema for Inspect Verb

**User Story:** As an LLM tooling consumer, I want `affi --introspect` to emit a JSON schema describing the `receipt_inspect` tool so that I can call it programmatically without reading source code.

**Test Procedure:**

```bash
affi --introspect
```

**Expected Output (stdout, parsed as JSON):**

```json
[
  { "name": "receipt_inspect", "parameters": { ... } },
  ...
]
```

**Pass Criteria:**
- Exit code: `0`
- stdout is valid JSON array
- Array contains an element with `"name": "receipt_inspect"`
- That element has a `"parameters"` key whose value is an object
- Array also contains `"receipt_emit"`, `"receipt_assemble"`, `"receipt_verify"`, `"receipt_show"`, `"receipt_model"`, `"receipt_conformance"`, `"receipt_diagnose"`, `"receipt_replay"`, `"receipt_graph"`, `"receipt_stats"` (11 verbs minimum)

**Fail Diagnosis:**
- If `receipt_inspect` is absent: verb was not registered in the ggen ontology or `#[verb]` macro is missing
- If output is not valid JSON: `--introspect` handler has a serialization bug

---

#### UAT-INS-006: Full Inspect Stage in the DX Pipeline

**User Story:** As a developer, I want the full `emit → assemble → verify → inspect` pipeline to work in a single session so that inspection is a natural step after certification.

**Test Procedure:** Run `tests/dx_full_pipeline_e2e.rs::full_dx_pipeline_through_the_binary`

```bash
cargo test full_dx_pipeline_through_the_binary
```

**Expected Output:**

```
test full_dx_pipeline_through_the_binary ... ok
```

**Pass Criteria:**
- All 7 pipeline stages succeed in sequence
- `affi receipt inspect r.json` exits 0 with `"RECEIPT INSPECTION REPORT"` in stderr
- The verify stage exits 0 with `"verdict: ACCEPT"` in stderr
- The diagnose stage exits 0 with `"no diagnostics — receipt is clean"` in stderr

---

#### UAT-INS-007: Inspect Handles Empty Receipt Path Gracefully

**User Story:** As a developer, I want `affi receipt inspect` to return a clean error message (not a panic) when the receipt file does not exist.

**Test Procedure:**

```bash
affi receipt inspect /nonexistent/path/receipt.json
```

**Expected Output (stderr):**
Contains a message indicating the file was not found. Does NOT contain `thread 'main' panicked`.

**Pass Criteria:**
- Exit code: non-zero
- stderr does NOT contain `"panicked at"`
- stderr does NOT contain `"called \`Option::unwrap()\` on a \`None\` value"`
- stderr contains a human-readable error (e.g., `"No such file"` or `"receipt decode failed"`)

---

### Suite 2: Process Discovery & IDE Integration

**Scope:** `affi receipt model`, `affi receipt conformance`, `affi receipt diagnose`, LSP hover/goto  
**Test Files:** `tests/dx_verbs_e2e.rs`, `tests/dx_full_pipeline_e2e.rs`  
**Library integration:** `wasm4pm` (process mining), `lsp-max` (diagnostics)

---

#### UAT-DIS-001: Model Verb Discovers Activity Names from Receipt

**User Story:** As a process analyst, I want `affi receipt model <path>` to automatically discover the activity sequence from a receipt so that I can see a process model without manually reading events.

**Pre-conditions:** Receipt assembled with at least 3 distinct event types.

**Test Procedure:**

```bash
affi receipt emit --type create    --object f:artifact --payload - <<< "create"
affi receipt emit --type transform --object d:artifact --payload - <<< "transform"
affi receipt emit --type release   --object f:artifact --payload - <<< "release"
affi receipt assemble --out r.json
affi receipt model r.json
```

**Expected Output (stderr):**

```
discovered process model (wasm4pm)
```
stderr also contains `"create"` and `"release"` (the discovered activity names).

**Pass Criteria:**
- Exit code: `0`
- stderr contains `"discovered process model (wasm4pm)"`
- stderr contains `"create"`
- stderr contains `"release"`

**Fail Diagnosis:**
- If `"discovered process model"` is absent: `handlers::model` is not calling wasm4pm discovery
- If activity names are absent: OCEL conversion in `src/ocel.rs` is dropping event types

---

#### UAT-DIS-002: Conformance Verb Reports Fitness Score

**User Story:** As a QA engineer, I want `affi receipt conformance <path>` to report a fitness score so that I can measure how closely a receipt follows its expected process model.

**Test Procedure:**

```bash
# Reuse the lifecycle receipt from UAT-DIS-001
affi receipt conformance r.json
```

**Expected Output (stderr):**

```
conformance metrics:
fitness (token replay):
activity_coverage:
NOT van der Aalst precision
simplicity (Occam):
```

**Pass Criteria:**
- Exit code: `0`
- stderr contains `"conformance metrics:"`
- stderr contains `"fitness (token replay):"`
- stderr contains `"activity_coverage:"`
- stderr contains `"NOT van der Aalst precision"` (honest labeling per project doctrine)
- stderr contains `"simplicity (Occam):"`

**Fail Diagnosis:**
- If `"fitness (token replay)"` is absent: `handlers::conformance` is not calling the token-replay algorithm
- If `"NOT van der Aalst precision"` is absent: honest labeling guard was removed — restore it

---

#### UAT-DIS-003: Diagnose Reports Clean Receipt

**User Story:** As a developer, I want `affi receipt diagnose <path>` to report "no diagnostics" for a valid receipt so that I have a fast sanity check without running full verification.

**Test Procedure:**

```bash
affi receipt diagnose r.json
```

**Expected Output (stderr):**

```
no diagnostics — receipt is clean
```

**Pass Criteria:**
- Exit code: `0`
- stderr is exactly or contains `"no diagnostics — receipt is clean"`

---

#### UAT-DIS-004: Stats Verb Aggregates All Surfaces

**User Story:** As a developer, I want `affi receipt stats <path>` to show event count, DFG metrics, and fitness in one command so that I get a full health summary without running multiple commands.

**Test Procedure:**

```bash
affi receipt stats r.json
```

**Expected Output (stderr):**

```
receipt stats:
events: 3
dfg:
fitness:
```

**Pass Criteria:**
- Exit code: `0`
- stderr contains `"receipt stats:"`
- stderr contains `"events: 3"`
- stderr contains `"dfg:"`
- stderr contains `"fitness:"`

---

#### UAT-DIS-005: Graph Verb Discovers Directly-Follows Graph

**User Story:** As a developer, I want `affi receipt graph <path>` to output the directly-follows graph so that I can visualize event-to-event transitions.

**Test Procedure:**

```bash
affi receipt graph r.json
```

**Expected Output (stderr):**

```
directly-follows graph (wasm4pm)
nodes (activities):
edges (df-relations):
```

**Pass Criteria:**
- Exit code: `0`
- stderr contains `"directly-follows graph (wasm4pm)"`
- stderr contains `"nodes (activities):"`
- stderr contains `"edges (df-relations):"`

---

#### UAT-DIS-006: Replay Verb Shows Events in Sequence Order

**User Story:** As a developer, I want `affi receipt replay <path>` to walk through each event step-by-step so that I can trace exactly what happened in sequence order.

**Test Procedure:**

```bash
affi receipt replay r.json
```

**Expected Output (stderr):**

```
replay (3 events)
step 0: create
step 2: release
replay complete
```

**Pass Criteria:**
- Exit code: `0`
- stderr contains `"replay (3 events)"`
- stderr contains `"step 0: create"`
- stderr contains `"step 2: release"`
- stderr contains `"replay complete"`

---

#### UAT-DIS-007: OCEL Court Refuses Objectless Event

**User Story:** As a security engineer, I want receipts with events that have no object references to be rejected by the OCEL court so that structurally hollow receipts cannot be certified.

**Test Procedure:** Run `tests/e2e.rs::e2e_objectless_receipt_rejected_by_ocel_court`

```bash
cargo test e2e_objectless_receipt_rejected_by_ocel_court
```

**Expected Output:**

```
test e2e_objectless_receipt_rejected_by_ocel_court ... ok
```

**Pass Criteria:**
- Test passes
- Inside the test: `affi receipt verify objectless.json` exits non-zero
- stderr contains `"EmptyEventObjectLinks"`

**Notes:** This test constructs a chain-consistent receipt (chain hash is correct) so that only the OCEL court's structural law can refuse it. This proves the court is load-bearing, not bypassed.

---

#### UAT-DIS-008: Forged Receipt Fails Deserialization (ADR-3)

**User Story:** As a security engineer, I want forged receipts to fail at deserialization — before reaching the verifier — so that the non-forgeable carrier guarantee (ADR-3) is end-to-end.

**Test Procedure:** Run `tests/e2e.rs::e2e_tamper_detection`

```bash
cargo test e2e_tamper_detection
```

**Expected Output (test passes), and inside the test:**

```
verdict: ... chain hash mismatch
```

**Pass Criteria:**
- Test passes
- `affi receipt verify tampered.json` exits non-zero
- stderr contains `"chain hash mismatch"` (from the deserialization gate, not just the verifier stage)

---

### Suite 3: Benchmarking & Regression Detection

**Scope:** Criterion harness, chain_append, chain_finalize, verifier_pipeline  
**Test File:** `benches/receipt_operations.rs`  
**Library integration:** `criterion` (regression detection + HTML reports)

---

#### UAT-BEN-001: Criterion Bench Runs Without Error

**User Story:** As a performance engineer, I want `cargo bench` to complete without errors so that I have a valid performance baseline before making changes.

**Test Procedure:**

```bash
cargo bench --bench receipt_operations 2>&1 | head -40
```

**Expected Output:**

```
chain_append_single_event  time: [...]
chain_finalize/1           time: [...]
chain_finalize/10          time: [...]
chain_finalize/100         time: [...]
chain_finalize/1000        time: [...]
verifier_pipeline_10_events time: [...]
```

**Pass Criteria:**
- Exit code: `0`
- Each benchmark name appears in output
- No `"panicked"` in output
- Each timing shows three values (lower, estimate, upper)

---

#### UAT-BEN-002: Chain Append Latency Under 500 µs

**User Story:** As a performance engineer, I want single-event append to be under 500 µs so that high-frequency receipt emission is feasible.

**Test Procedure:**

```bash
cargo bench --bench receipt_operations -- chain_append_single_event 2>&1 | grep "time:"
```

**Expected Output:**

```
chain_append_single_event  time:   [X.XX µs X.XX µs X.XX µs]
```

Where the estimate (middle value) is ≤ 500 µs.

**Pass Criteria:**
- Criterion estimate (middle value) ≤ 500 µs
- No outliers flagged as "High mild" or "High severe" with magnitude > 10x estimate

**Notes:** The project CLAUDE.md documents ~100 µs as the expected baseline. 500 µs is the hard acceptance ceiling.

---

#### UAT-BEN-003: Chain Finalize Scales Sub-Linearly for 100 Events

**User Story:** As a performance engineer, I want finalize to scale acceptably with event count so that large receipts remain practical.

**Test Procedure:**

```bash
cargo bench --bench receipt_operations -- chain_finalize 2>&1 | grep "time:"
```

**Expected Output:**

```
chain_finalize/1     time:   [X µs ...]
chain_finalize/10    time:   [X µs ...]
chain_finalize/100   time:   [X ms ...]
chain_finalize/1000  time:   [X ms ...]
```

**Pass Criteria:**
- `chain_finalize/100` estimate ≤ 100 ms (10x the CLAUDE.md documented ~10 ms)
- `chain_finalize/1000` estimate ≤ 1000 ms
- The ratio `finalize/100 / finalize/1` ≤ 200 (roughly linear, not exponential)

---

#### UAT-BEN-004: Verifier Pipeline Latency Under 150 ms for 10-Event Receipt

**User Story:** As a performance engineer, I want the 7-stage verify pipeline on a 10-event receipt to complete under 150 ms so that CI verification is fast.

**Test Procedure:**

```bash
cargo bench --bench receipt_operations -- verifier_pipeline_10_events 2>&1 | grep "time:"
```

**Expected Output:**

```
verifier_pipeline_10_events  time:   [X µs ...]
```

**Pass Criteria:**
- Criterion estimate ≤ 150 ms

**Notes:** CLAUDE.md documents ~75 ms for 100-event receipts. A 10-event receipt should be proportionally faster.

---

#### UAT-BEN-005: Bench Detects Regression When Hash Algorithm Changes

**User Story:** As a developer, I want a >10% performance regression to be flagged by Criterion so that accidentally slow code is caught before merge.

**Test Procedure (simulation):**

1. Run `cargo bench` and save baseline:
   ```bash
   cargo bench --bench receipt_operations -- --save-baseline before
   ```
2. Artificially slow the `fold_event` function by adding a `std::thread::sleep(std::time::Duration::from_micros(50))` (in a test branch only).
3. Run bench again:
   ```bash
   cargo bench --bench receipt_operations -- --baseline before
   ```

**Expected Output:**

```
chain_append_single_event  Performance has regressed.
```

**Pass Criteria:**
- Criterion reports regression for the modified benchmark
- Exit code: non-zero if `--ensure-no-regression` flag is used in CI

**Notes:** In CI, the command is `cargo bench -- --ensure-no-regression`. This test documents the expected behavior; it is run manually, not as a `cargo test` target.

---

#### UAT-BEN-006: All Benchmark Functions Are Present and Named

**User Story:** As a performance engineer, I want every documented benchmark function to exist in the bench file so that no measurement gaps exist.

**Test Procedure:**

```bash
grep -E "^fn bench_" benches/receipt_operations.rs
```

**Expected Output:**

```
fn bench_chain_append(...)
fn bench_chain_finalize(...)
fn bench_verifier_pipeline(...)
fn bench_chain_recompute(...)
```

**Pass Criteria:**
- All four benchmark functions are present
- `criterion_group!` and `criterion_main!` macros are present at file end

---

### Suite 4: Mutation Testing & Fixtures

**Scope:** Tamper detection via verifier stages, adversarial suite, receipt mutation patterns  
**Test Files:** `tests/adversarial.rs`, `tests/chicago_tdd_witness.rs`  
**Library integration:** `chicago-tdd-tools`, `clnrm-core`

---

#### UAT-MUT-001: Flipped Commitment Byte Rejects at Chain Integrity

**User Story:** As a security engineer, I want a single-byte flip in a payload commitment to cause REJECT at `chain_integrity` — not at a later stage — so that the chain is the first line of defense.

**Test Procedure:** Run `tests/adversarial.rs::tamper_commitment_rejects_at_chain_integrity`

```bash
cargo test tamper_commitment_rejects_at_chain_integrity
```

**Expected Output:**

```
test tamper_commitment_rejects_at_chain_integrity ... ok
```

**Internal assertions (from source):**
- `verify(&receipt).accepted == false`
- Stage `chain_integrity` has `passed == false`

**Pass Criteria:**
- Test passes
- The teeth check (untampered receipt ACCEPTs) also passes in the same test

---

#### UAT-MUT-002: Event Reorder Rejects at Continuity or Chain Integrity

**User Story:** As a security engineer, I want swapping two events to cause REJECT so that event ordering is enforced and cannot be exploited by reordering.

**Test Procedure:** Run `tests/adversarial.rs::tamper_reorder_rejects`

```bash
cargo test tamper_reorder_rejects
```

**Expected Output:**

```
test tamper_reorder_rejects ... ok
```

**Pass Criteria:**
- Test passes
- `verdict.accepted == false`
- Either `continuity` stage or `chain_integrity` stage (or both) failed

---

#### UAT-MUT-003: Injected Fabricated Event Rejects at Chain Integrity

**User Story:** As a security engineer, I want a fabricated event appended to a receipt (without recomputing the chain hash) to be rejected at `chain_integrity` so that event injection is unconstructable.

**Test Procedure:** Run `tests/adversarial.rs::tamper_inject_rejects_at_chain_integrity`

```bash
cargo test tamper_inject_rejects_at_chain_integrity
```

**Expected Output:**

```
test tamper_inject_rejects_at_chain_integrity ... ok
```

**Pass Criteria:**
- Test passes
- `verdict.accepted == false`
- Stage `chain_integrity` has `passed == false`

---

#### UAT-MUT-004: Chicago-TDD Assertion Macros Refuse a Forged Receipt

**User Story:** As a developer, I want the chicago-tdd `assert_err!` macro to catch forged receipt attempts so that the test tooling and the domain law are consistent.

**Test Procedure:** Run `tests/chicago_tdd_witness.rs::chicago_tdd_asserts_forged_receipt_is_refused`

```bash
cargo test chicago_tdd_asserts_forged_receipt_is_refused
```

**Expected Output:**

```
test chicago_tdd_asserts_forged_receipt_is_refused ... ok
```

**Pass Criteria:**
- Test passes
- Removing `chicago-tdd-tools` from `Cargo.toml` causes a compile error (not just a test failure) — verified by checking that `assert_err!` is a re-export from `chicago_tdd_tools`

---

#### UAT-MUT-005: Private `_seal` Field Prevents Struct-Literal Construction

**User Story:** As a developer, I want compiler error `E0451` when attempting to construct `Receipt { _seal: (), ... }` so that the sealed carrier invariant (ADR-2) is a compile-time guarantee.

**Test Procedure:** Compile the trybuild test that attempts struct-literal construction:

```bash
cargo test --test ui
```

**Expected Output:**

```
test ui ... ok
```

**Pass Criteria:**
- The `trybuild` test in `tests/ui/` (or `tests/ui.rs`) passes
- The specific test verifies that `receipt_no_direct_construct.rs` produces compile error `E0451: field '_seal' of struct 'Receipt' is private`

**Notes:** If no trybuild test file exists yet, this UAT signals the requirement to create one at `tests/ui/receipt_no_direct_construct.rs`. It is a DoD requirement.

---

#### UAT-MUT-006: Deserialization Gate Blocks Forged JSON (ADR-3)

**User Story:** As a security engineer, I want `serde_json::from_str` on a receipt with a tampered event_type to fail with `"chain hash mismatch"` so that the non-forgeable carrier invariant (ADR-3) holds at the deserialization layer.

**Test Procedure:** Run `src/types.rs::tests::deserialize_rejects_forged_receipt`

```bash
cargo test deserialize_rejects_forged_receipt
```

**Expected Output:**

```
test types::tests::deserialize_rejects_forged_receipt ... ok
```

**Pass Criteria:**
- Test passes
- `serde_json::from_str::<Receipt>(&forged_json)` returns `Err(_)`
- The error string contains `"chain hash mismatch"`

---

#### UAT-MUT-007: Mutation Kill Rate ≥ 90% Across 10 Distinct Mutations

**User Story:** As a QA engineer, I want at least 90% of systematically applied mutations to a valid receipt to be rejected by the verifier so that the verifier's sensitivity is documented and enforced.

**Test Procedure:** This is a library-level test, not a CLI test. The mutations applied:

| # | Mutation | Expected Rejection Stage |
|---|---|---|
| M1 | Flip 1 byte in `payload_commitment` hex of event 0 | `chain_integrity` |
| M2 | Flip 1 byte in `payload_commitment` hex of event 1 | `chain_integrity` |
| M3 | Flip 1 byte in `payload_commitment` hex of event 2 | `chain_integrity` |
| M4 | Change `event_type` of event 0 | `chain_integrity` |
| M5 | Change `event_type` of event 1 | `chain_integrity` |
| M6 | Swap events 0 and 1 (reorder) | `continuity` or `chain_integrity` |
| M7 | Inject a new event without recomputing chain | `chain_integrity` |
| M8 | Change `format_version` to `"2.0"` | `check_format` |
| M9 | Change `chain_hash` to random hex | `chain_integrity` |
| M10 | Set `seq` of event 1 to 5 (gap) | `continuity` |

**Run:** `cargo test -- --test-threads=1` (for determinism; any test catching these patterns)

**Pass Criteria:**
- All 10 mutations produce `verdict.accepted == false`
- Kill rate: 10/10 = 100% for these 10 known mutations
- The existing `tests/adversarial.rs` suite covers M1–M7

---

#### UAT-MUT-008: Generated Test Code Compiles and Passes

**User Story:** As a developer, I want `affi receipt generate test` (when implemented) to produce Rust test code that compiles and passes so that auto-generated tests are trustworthy.

**Test Procedure:** (Planned feature — marks acceptance bar for Phase 3)

```bash
affi receipt generate test --out generated_test.rs r.json
rustc --edition 2021 generated_test.rs --extern affidavit=...
```

**Pass Criteria:**
- Exit code from `affi receipt generate test`: `0`
- Generated file compiles with `cargo test`
- Generated test passes

**Status:** Planned (Phase 3). This UAT establishes the acceptance bar; it is not yet testable.

---

### Suite 5: Observability & Tracing

**Scope:** OTel span emission, span capture, `affidavit::tracing` module  
**Test Files:** `tests/otel_witness.rs`, `tests/otel_all_spans.rs`, `tests/otel_weaver_registry.rs`  
**Library integration:** `opentelemetry`, `opentelemetry-jaeger`

---

#### UAT-OBS-001: Verify Emits an Observable Span

**User Story:** As a platform engineer, I want `affi receipt verify` to emit an OpenTelemetry span named `"verify"` so that I can trace verification calls through a distributed system.

**Test Procedure:** Run `tests/otel_witness.rs::verify_emits_an_observable_span`

```bash
cargo test verify_emits_an_observable_span
```

**Expected Output:**

```
test verify_emits_an_observable_span ... ok
```

**Pass Criteria:**
- Test passes
- `affidavit::tracing::captured_spans()` returns at least one span where `span.operation == "verify"` and `span.target == <receipt_path>`
- `clear_spans()` → `verify()` → `captured_spans()` is the correct call order (test must not rely on leftover spans)

**Fail Diagnosis:**
- If no span captured: `trace_verify()` in `src/cli.rs` was removed or the span sink is not wired
- If `operation` field is wrong: check `src/tracing.rs` span construction

---

#### UAT-OBS-002: Span Has Non-Empty Target Field

**User Story:** As a platform engineer, I want each span's `target` field to carry the receipt path so that I can correlate spans to specific receipt files in logs.

**Test Procedure:** Inspect the span returned by `captured_spans()` after running `cli::verify(&receipt_str)`.

**Pass Criteria:**
- `span.target` equals the string path passed to `verify()`
- `span.target` is not empty
- `span.target` is not `"<unknown>"`

---

#### UAT-OBS-003: Clear Spans Resets the Sink

**User Story:** As a test author, I want `affidavit::tracing::clear_spans()` to empty the span sink so that test isolation is guaranteed between consecutive test runs.

**Test Procedure:**

```rust
affidavit::tracing::clear_spans();
assert!(affidavit::tracing::captured_spans().is_empty());

// Run some operation
let _ = affidavit::cli::verify(&receipt_path);

assert!(!affidavit::tracing::captured_spans().is_empty());

affidavit::tracing::clear_spans();
assert!(affidavit::tracing::captured_spans().is_empty()); // reset
```

**Pass Criteria:**
- Each `clear_spans()` call results in `captured_spans().is_empty() == true`
- `captured_spans()` grows after a verify call
- This pattern is thread-safe (no data race when run with `--test-threads=1`)

---

#### UAT-OBS-004: OTel Witness Test Compiles with `opentelemetry` Dev-Dependency

**User Story:** As a developer, I want the `opentelemetry` dev-dependency to be genuinely wired so that a build without it causes a compile error — not a silent no-op.

**Test Procedure:**

```bash
cargo test --test otel_witness
```

**Expected Output:**

```
test verify_emits_an_observable_span ... ok
```

**Pass Criteria:**
- Test compiles and passes
- `Cargo.toml` [dev-dependencies] contains `opentelemetry = { version = "0.20", features = ["trace"] }`
- Removing `opentelemetry` from dev-dependencies causes compile failure in `tests/otel_witness.rs`

---

#### UAT-OBS-005: OTel Weaver Registry Test Passes

**User Story:** As a platform engineer, I want the OTel Weaver semantic conventions test to pass so that span attributes conform to the standardized schema.

**Test Procedure:**

```bash
cargo test --test otel_weaver_registry
```

**Expected Output:**

```
test [otel_weaver_registry test name] ... ok
```

**Pass Criteria:**
- All tests in `tests/otel_weaver_registry.rs` pass
- No attribute name typos (semantic convention enforcement)

---

#### UAT-OBS-006: All OTel Span Tests Pass Concurrently

**User Story:** As a CI engineer, I want the full `otel_all_spans` test suite to pass so that all instrumentation points are continuously validated.

**Test Procedure:**

```bash
cargo test --test otel_all_spans
```

**Pass Criteria:**
- All tests in `tests/otel_all_spans.rs` pass
- Exit code: `0`

---

#### UAT-OBS-007: Verify Span Appears in Captured Spans After Binary Invocation

**User Story:** As a developer, I want spans captured through the in-process call path (not the binary path) so that integration tests can assert on telemetry without requiring a running Jaeger instance.

**Test Procedure:** (In-process test, not CLI test)

```rust
use affidavit::tracing::{clear_spans, captured_spans};
use affidavit::cli;

let dir = TempDir::new().expect("tempdir");
// ... assemble receipt to dir ...

clear_spans();
let (code, verdict) = cli::verify(&receipt_path_str).expect("verify runs");

assert_eq!(code, 0);
assert!(verdict.accepted);

let spans = captured_spans();
assert!(spans.iter().any(|s| s.operation == "verify"),
    "at least one verify span must be captured; got: {spans:?}");
```

**Pass Criteria:**
- At least one span with `operation == "verify"` is captured
- The span sink is thread-local and does not bleed between parallel tests

---

### Suite 6: CLI Ergonomics

**Scope:** Dispatch routing, `--introspect`, exit codes, stderr/stdout routing, `show` output  
**Test Files:** `tests/cli_dispatch.rs`, `tests/dx_introspect_e2e.rs`, `src/cli.rs` (UI tests)  
**Library integration:** `clap-noun-verb`, ggen ontology

---

#### UAT-CLI-001: Dispatch Emit Routes to Correct Handler

**User Story:** As a developer, I want `affi receipt emit` to route to the emit handler and print the emitted event ID so that I get confirmation of what was recorded.

**Test Procedure:** Run `tests/cli_dispatch.rs::dispatch_emit_first`

```bash
cargo test dispatch_emit_first
```

**Expected Output:**

```
test dispatch_emit_first ... ok
```

**Internal assertion:** stdout contains `"emitted event"`.

**Pass Criteria:**
- Test passes
- stdout (not stderr) contains `"emitted event"`

---

#### UAT-CLI-002: Verify Exits 0 on ACCEPT, Non-Zero on REJECT

**User Story:** As a CI script author, I want `affi receipt verify` to exit `0` for a valid receipt and non-zero for an invalid one so that I can use it in pipeline conditionals.

**Test Procedure:**

```bash
# Step 1: Build honest receipt
affi receipt emit --type build --object src:artifact --payload - <<< "source"
affi receipt assemble --out honest.json

# Step 2: Verify honest — must exit 0
affi receipt verify honest.json
echo "Exit code: $?"   # must print "Exit code: 0"

# Step 3: Tamper
sed -i 's/"build"/"evil"/' honest.json

# Step 4: Verify tampered — must exit non-zero
affi receipt verify honest.json
echo "Exit code: $?"   # must print "Exit code: 2" or other non-zero
```

**Pass Criteria:**
- `verify honest.json` exits `0`
- `verify tampered.json` exits non-zero (specifically `2` per the `handlers::verify` implementation)
- stderr for honest contains `"verdict: ACCEPT"`
- stderr for tampered contains `"chain hash mismatch"`

---

#### UAT-CLI-003: Show Prints Human-Readable Receipt Dump

**User Story:** As a developer, I want `affi receipt show <path>` to print a readable summary of the receipt (format, events, chain hash) so that I can inspect a receipt without parsing JSON.

**Test Procedure:**

```bash
affi receipt show r.json
```

**Expected Output (stderr):**

```
receipt format: core/v1
events: 3
evt-0
evt-1
evt-2
chain hash:
```

**Pass Criteria:**
- Exit code: `0`
- stderr contains `"receipt format: core/v1"`
- stderr contains `"events: 3"`
- stderr contains `"evt-0"`, `"evt-1"`, `"evt-2"`
- stderr contains `"chain hash:"`

---

#### UAT-CLI-004: Qualified Objects Display in `id:type/qualifier` Format

**User Story:** As a developer, I want qualified object references displayed as `id:type/qualifier` so that the show command reflects the full object reference syntax.

**Test Procedure:** Run `tests/e2e.rs::e2e_qualified_objects`

```bash
cargo test e2e_qualified_objects
```

**Expected Output:**

```
test e2e_qualified_objects ... ok
```

**Internal assertion:** stderr contains `"dataset:artifact/input"`.

**Pass Criteria:**
- Test passes
- Qualified object is rendered as `<id>:<obj_type>/<qualifier>` not as raw JSON

---

#### UAT-CLI-005: Stdin Payload Accepted via `--payload -`

**User Story:** As a developer, I want to pipe payload bytes via stdin using `--payload -` so that I can script receipt emission without creating temporary files.

**Test Procedure:** Run `tests/e2e.rs::e2e_stdin_payload`

```bash
cargo test e2e_stdin_payload
```

**Expected Output:**

```
test e2e_stdin_payload ... ok
```

**Pass Criteria:**
- Test passes
- Receipt assembled and verified ACCEPT in the same test

---

#### UAT-CLI-006: Introspect Emits Valid JSON Schema Array

**User Story:** As an LLM integration engineer, I want `affi --introspect` to emit a JSON array of tool schemas so that I can register all `affi` commands as LLM tools programmatically.

**Test Procedure:** Run `tests/dx_introspect_e2e.rs::introspect_emits_valid_schema_for_all_verbs`

```bash
cargo test introspect_emits_valid_schema_for_all_verbs
```

**Expected Output:**

```
test introspect_emits_valid_schema_for_all_verbs ... ok
```

**Pass Criteria:**
- `affi --introspect` exits `0`
- stdout is a valid JSON array
- Array is non-empty
- All 11 expected verb names appear: `receipt_emit`, `receipt_assemble`, `receipt_verify`, `receipt_show`, `receipt_inspect`, `receipt_model`, `receipt_conformance`, `receipt_diagnose`, `receipt_replay`, `receipt_graph`, `receipt_stats`
- Every tool schema has a `"parameters"` key whose value is an object

---

#### UAT-CLI-007: No Raw Payloads in Assembled Receipts

**User Story:** As a security engineer, I want assembled receipt JSON to contain `payload_commitment` hex strings — not raw payload bytes — so that the provenance layer never leaks sensitive input data.

**Test Procedure:**

```bash
affi receipt emit --type secret-op --object secret:artifact --payload - <<< "TOP SECRET DATA"
affi receipt assemble --out receipt.json
grep -c "TOP SECRET DATA" receipt.json
```

**Expected Output:**

```
0
```

**Pass Criteria:**
- `grep` returns `0` matches — the literal payload string is absent from the assembled receipt
- The assembled receipt contains `"payload_commitment":` with a 64-character hex string
- `grep -E '"payload_commitment":\s*"[0-9a-f]{64}"' receipt.json` returns at least 1 match

---

#### UAT-CLI-008: Complete Lifecycle in Single Session

**User Story:** As a developer, I want to go from zero to a verified receipt in a single session without documentation lookups so that the CLI is ergonomic and self-discoverable.

**Test Procedure:** Run `tests/e2e.rs::e2e_complete_lifecycle_honest`

```bash
cargo test e2e_complete_lifecycle_honest
```

**Expected Output:**

```
test e2e_complete_lifecycle_honest ... ok
```

**Pass Criteria:**
- All 4 stages (emit ×3 → assemble → verify → show) succeed
- Each stage produces its documented output
- Verdict is `ACCEPT` with reason `"all stages passed"`

---

## 3. Acceptance Test Coverage Matrix

| Scenario ID | Phase | Feature | Handler | Test File | Line Range (approx) |
|---|---|---|---|---|---|
| UAT-INS-001 | Phase 1 | receipt inspect | `handlers::inspect` | `tests/dx_verbs_e2e.rs` | `inspect_verb_reports_event_distribution` |
| UAT-INS-002 | Phase 1 | receipt inspect (objects) | `handlers::inspect` | `tests/dx_verbs_e2e.rs` | same as INS-001 |
| UAT-INS-003 | Phase 1 | receipt inspect (1-event) | `handlers::inspect` | `tests/dx_verbs_e2e.rs` | same |
| UAT-INS-004 | Phase 1 | chicago-tdd admit | `admission::admit` | `tests/chicago_tdd_witness.rs` | `chicago_tdd_asserts_honest_receipt_is_admitted` |
| UAT-INS-005 | Phase 1 | `--introspect` schema | `handlers::*` (all verbs) | `tests/dx_introspect_e2e.rs` | `introspect_emits_valid_schema_for_all_verbs` |
| UAT-INS-006 | Phase 1 | full DX pipeline | all handlers | `tests/dx_full_pipeline_e2e.rs` | `full_dx_pipeline_through_the_binary` |
| UAT-INS-007 | Phase 1 | error handling | `handlers::inspect` | New (to be written) | — |
| UAT-DIS-001 | Phase 2 | receipt model | `handlers::model` | `tests/dx_verbs_e2e.rs` | `model_verb_discovers_a_process_model` |
| UAT-DIS-002 | Phase 2 | receipt conformance | `handlers::conformance` | `tests/dx_verbs_e2e.rs` | `conformance_verb_reports_fitness_and_metrics` |
| UAT-DIS-003 | Phase 2 | receipt diagnose | `handlers::diagnose` | `tests/dx_verbs_e2e.rs` | `diagnose_verb_reports_clean_for_honest_receipt` |
| UAT-DIS-004 | Phase 2 | receipt stats | `handlers::stats` | `tests/dx_verbs_e2e.rs` | `stats_verb_aggregates_all_surfaces` |
| UAT-DIS-005 | Phase 2 | receipt graph | `handlers::graph` | `tests/dx_verbs_e2e.rs` | `graph_verb_discovers_directly_follows_graph` |
| UAT-DIS-006 | Phase 2 | receipt replay | `handlers::replay` | `tests/dx_verbs_e2e.rs` | `replay_verb_shows_steps_in_seq_order` |
| UAT-DIS-007 | Phase 2 | OCEL court | `admission::admit` | `tests/e2e.rs` | `e2e_objectless_receipt_rejected_by_ocel_court` |
| UAT-DIS-008 | Phase 2 | deserialization gate | `types::Receipt::deserialize` | `tests/e2e.rs` | `e2e_tamper_detection` |
| UAT-BEN-001 | Phase 3 | Criterion harness | `benches/receipt_operations.rs` | Bench (not test) | All functions |
| UAT-BEN-002 | Phase 3 | chain_append latency | `chain::fold_event` | `benches/receipt_operations.rs` | `bench_chain_append` |
| UAT-BEN-003 | Phase 3 | chain_finalize scaling | `chain::ChainAssembler::finalize` | `benches/receipt_operations.rs` | `bench_chain_finalize` |
| UAT-BEN-004 | Phase 3 | verifier latency | `verifier::verify` | `benches/receipt_operations.rs` | `bench_verifier_pipeline` |
| UAT-BEN-005 | Phase 3 | regression detection | Criterion `--baseline` | Manual bench run | — |
| UAT-BEN-006 | Phase 3 | bench function presence | All bench fns | `benches/receipt_operations.rs` | All |
| UAT-MUT-001 | Phase 4 | commitment tamper | `verifier::stage_chain_integrity` | `tests/adversarial.rs` | `tamper_commitment_rejects_at_chain_integrity` |
| UAT-MUT-002 | Phase 4 | event reorder | `verifier::stage_continuity` | `tests/adversarial.rs` | `tamper_reorder_rejects` |
| UAT-MUT-003 | Phase 4 | event inject | `verifier::stage_chain_integrity` | `tests/adversarial.rs` | `tamper_inject_rejects_at_chain_integrity` |
| UAT-MUT-004 | Phase 4 | chicago-tdd assert_err | `admission::admit` | `tests/chicago_tdd_witness.rs` | `chicago_tdd_asserts_forged_receipt_is_refused` |
| UAT-MUT-005 | Phase 4 | private `_seal` | `types::Receipt` | `tests/ui/` | `receipt_no_direct_construct.rs` |
| UAT-MUT-006 | Phase 4 | deser gate | `types::Receipt::deserialize` | `src/types.rs` (inline) | `deserialize_rejects_forged_receipt` |
| UAT-MUT-007 | Phase 4 | mutation kill rate | `verifier::verify` | `tests/adversarial.rs` (all) | All adversarial tests |
| UAT-MUT-008 | Phase 4 | test generation | (Planned) | — | — |
| UAT-OBS-001 | Phase 5 | verify span | `cli::trace_verify` | `tests/otel_witness.rs` | `verify_emits_an_observable_span` |
| UAT-OBS-002 | Phase 5 | span target field | `tracing::ObservableSpan` | `tests/otel_witness.rs` | same |
| UAT-OBS-003 | Phase 5 | clear_spans | `tracing::clear_spans` | `tests/otel_witness.rs` | same |
| UAT-OBS-004 | Phase 5 | OTel dep genuine | `opentelemetry` | `tests/otel_witness.rs` | Compile check |
| UAT-OBS-005 | Phase 5 | OTel weaver | `opentelemetry` semconv | `tests/otel_weaver_registry.rs` | All |
| UAT-OBS-006 | Phase 5 | all spans | `tracing` | `tests/otel_all_spans.rs` | All |
| UAT-OBS-007 | Phase 5 | span in-process | `cli::verify` + `tracing` | `tests/otel_witness.rs` | All |
| UAT-CLI-001 | Phase 6 | emit dispatch | `handlers::emit` | `tests/cli_dispatch.rs` | `dispatch_emit_first` |
| UAT-CLI-002 | Phase 6 | exit code mapping | `handlers::verify` | `tests/e2e.rs` | `e2e_complete_lifecycle_honest` + `e2e_tamper_detection` |
| UAT-CLI-003 | Phase 6 | show output | `handlers::show` | `tests/e2e.rs` | `e2e_complete_lifecycle_honest` |
| UAT-CLI-004 | Phase 6 | qualified objects | `handlers::show` | `tests/e2e.rs` | `e2e_qualified_objects` |
| UAT-CLI-005 | Phase 6 | stdin payload | `cli::emit` | `tests/e2e.rs` | `e2e_stdin_payload` |
| UAT-CLI-006 | Phase 6 | `--introspect` JSON | all handlers | `tests/dx_introspect_e2e.rs` | All |
| UAT-CLI-007 | Phase 6 | no raw payloads | `chain::serialize_receipt` | CLI grep check | — |
| UAT-CLI-008 | Phase 6 | complete lifecycle | all handlers | `tests/e2e.rs` | `e2e_complete_lifecycle_honest` |

---

## 4. Non-Functional Acceptance Tests

### 4.1 Performance Acceptance Tests

| Command / Operation | Acceptance Ceiling | Measurement Method | Failure Action |
|---|---|---|---|
| `affi receipt emit` (single event) | ≤ 500 µs | `cargo bench -- chain_append_single_event` | Block merge; investigate `fold_event` |
| `affi receipt assemble` (100 events) | ≤ 100 ms | `cargo bench -- chain_finalize/100` | Block merge; investigate canonical JSON |
| `affi receipt verify` (100-event receipt) | ≤ 150 ms | `cargo bench -- verifier_pipeline` (100-event) | Investigate which stage is slow |
| `affi receipt inspect` (100 events) | ≤ 200 ms (CLI round-trip) | `time affi receipt inspect r.json` | Investigate handler aggregation |
| `affi receipt model` (10 events) | ≤ 2 s (wasm4pm mining) | `time affi receipt model r.json` | wasm4pm mining timeout |
| `affi --introspect` | ≤ 50 ms | `time affi --introspect` | Schema generation bottleneck |

**Regression policy:** A Criterion run with `--baseline before` that shows `> 10%` regression in `chain_append_single_event`, `chain_finalize/100`, or `verifier_pipeline_10_events` must block merge until the regression is explained and accepted or reverted.

### 4.2 Memory Acceptance Tests

| Scenario | Acceptance Ceiling | Measurement Method |
|---|---|---|
| 100-event receipt in memory | ≤ 500 KB | Heap profile via `heaptrack` or `valgrind --tool=massif` |
| `ChainAssembler` for 100 events before finalize | ≤ 2 MB | Heap profile |
| `verifier::verify` peak allocation | ≤ 1 MB | Heap profile |

**Verification command:**
```bash
valgrind --tool=massif --pages-as-heap=yes \
  cargo run --release --bin affi -- receipt verify large-receipt.json
ms_print massif.out.<pid> | head -30
```

**Pass criteria:** Peak heap usage ≤ 500 KB for a 100-event receipt at the `receipt verify` step.

### 4.3 Determinism Acceptance Tests

**Definition:** Given identical inputs, `affi receipt assemble` must produce byte-for-byte identical output across all runs, including concurrent runs.

**Test Procedure:**

```bash
# Step 1: Build a receipt with fixed payload strings
WORK=$(mktemp -d)
cd $WORK

for i in $(seq 1 10); do
  rm -rf .affi
  echo "source bytes" | affi receipt emit --type build --object src:artifact --payload -
  echo "test bytes"   | affi receipt emit --type test  --object bin:artifact --payload -
  affi receipt assemble --out "run-${i}.json"
done

# Step 2: Compare all 10 outputs — must be identical
md5sum run-*.json | awk '{print $1}' | sort -u | wc -l
# Expected: 1 (all hashes identical)
```

**Pass Criteria:**
- `wc -l` output is `1` — all 10 receipts have the same MD5 hash
- `diff run-1.json run-2.json` exits `0`

**Notes:** Determinism is guaranteed by:
1. `canonical_bytes` sorts JSON object keys recursively before hashing
2. Events are ordered by `seq` (monotonic, not wall-clock)
3. BLAKE3 is deterministic over identical byte sequences

### 4.4 Security Acceptance Tests

**NFT-SEC-001: No Raw Payloads in Receipts**

```bash
echo "SENSITIVE PAYLOAD" | affi receipt emit --type op --object obj:artifact --payload -
affi receipt assemble --out secure.json
if grep -q "SENSITIVE PAYLOAD" secure.json; then
  echo "FAIL: raw payload found in receipt"
  exit 1
fi
echo "PASS: no raw payload in receipt"
```

**Pass Criteria:**
- `grep` finds no match
- `secure.json` contains `"payload_commitment": "<64-char hex>"` instead

**NFT-SEC-002: Struct-Literal Construction of Receipt Is Unconstructable**

```rust
// This must fail to compile with E0451:
let r = affidavit::types::Receipt { _seal: (), format_version: "core/v1".to_string(), events: vec![], chain_hash: affidavit::types::Blake3Hash::from_hex("a".repeat(64)) };
```

**Pass Criteria:** The above code does not compile. Verified by the `trybuild` test in `tests/ui/`.

**NFT-SEC-003: Chain Hash Is Non-Forgeable Without Payload Re-Computation**

Covered by UAT-MUT-006 (deserialization gate) and the adversarial suite.

**NFT-SEC-004: OCEL Court Refuses Events Without Object Links**

Covered by UAT-DIS-007.

### 4.5 Error Handling Acceptance Tests

| Input | Expected Behavior | Must NOT Happen |
|---|---|---|
| `affi receipt verify /nonexistent.json` | Non-zero exit; stderr contains `"receipt decode failed"` or `"No such file"` | `thread 'main' panicked` |
| `affi receipt verify <valid-path-to-malformed-json>` | Non-zero exit; stderr contains error message | `unwrap()` panic |
| `affi receipt emit --type "" --object obj:artifact --payload -` | Non-zero exit; stderr contains validation error | Silently succeed with empty event_type |
| `affi receipt assemble` with no working receipt | Non-zero exit; stderr contains meaningful message | `Index out of bounds` panic |
| `affi receipt verify <tampered-json>` | Non-zero exit; stderr contains `"chain hash mismatch"` | `unwrap()` panic |

**Pass Criteria for all rows:**
- Exit code: non-zero
- stderr: human-readable error message
- No `"panicked at"` in stderr
- No `"called \`Option::unwrap()\` on a \`None\` value"` in stderr

---

## 5. Regression Test Scenarios

These tests must pass to prove that the DX/QOL 1000x initiative introduced no regressions against the v26.6.17 baseline.

### 5.1 Existing Test Suite Must Still Pass

**Requirement:** All tests that passed on `main` at v26.6.17 must pass on `claude/zen-cerf-oq87br`.

**Verification:**

```bash
git stash  # if needed
cargo test --all 2>&1 | tail -10
# Expected: "test result: ok. N passed; 0 failed; 0 ignored"
```

**Known test count at v26.6.17 (CLAUDE.md documented):**

| Suite | Count | Location |
|---|---|---|
| Unit (chain, admission, types, discovery, verifier) | 19 | `src/` inline |
| Dispatch | 6 | `src/handlers.rs` inline |
| E2E | 4+ | `tests/e2e.rs` |
| UI | 1 | `tests/ui.rs` or `tests/ui/` |
| **Total baseline** | **30+** | — |

**Pass Criteria:**
- `cargo test --all` exits `0`
- Test count ≥ 30 (new tests may be added; existing tests must not be deleted or weakened)
- Zero failures

### 5.2 Golden Run Produces Same Chain Hash

**Requirement:** Running `examples/golden_run.sh` on the new branch must produce `exit code 0` (ACCEPT then REJECT), and the intermediate `chain_hash` of the honest receipt must be identical to the value produced from the same inputs on `main`.

**Verification:**

```bash
bash examples/golden_run.sh
# Expected final line: "=== GOLDEN RUN OK: ACCEPT(0) then REJECT(...) ==="
```

**Pass Criteria:**
- Script exits `0`
- "=== GOLDEN RUN OK" appears in output
- The honest receipt's ACCEPT exit code is `0`
- The tampered receipt's exit code is non-zero

### 5.3 Receipt Sealed at v26.6.17 Verifies at v26.6.17+

**Requirement:** A receipt assembled with `affi` at `main` (v26.6.17) must verify cleanly with `affi` built from `claude/zen-cerf-oq87br`.

**Verification:**

```bash
# Step 1: On main, build and assemble a reference receipt
git checkout main
cargo build --release
./target/release/affi receipt emit --type build --object src:artifact --payload - <<< "pinned-payload"
./target/release/affi receipt assemble --out /tmp/v26614-reference.json

# Step 2: On new branch, verify the reference receipt
git checkout claude/zen-cerf-oq87br
cargo build --release
./target/release/affi receipt verify /tmp/v26614-reference.json
```

**Pass Criteria:**
- Step 2 exits `0`
- stderr contains `"verdict: ACCEPT"`
- The `chain_hash` in `/tmp/v26614-reference.json` matches the value computed at assembly time (determinism cross-version guarantee)

**Notes:** The genesis seed `b"affidavit-v26.6.17-genesis"` is pinned in `src/chain.rs::GENESIS_SEED`. Any change to this constant is a **breaking change** and requires a major version bump.

### 5.4 All Dispatch Tests Still Route Correctly

**Requirement:** Each `affi receipt <verb>` must still route to its dedicated handler.

**Verification:**

```bash
cargo test --test cli_dispatch
```

**Pass Criteria:**
- All dispatch tests pass
- No verb falls through to a default/unknown handler

### 5.5 Chicago-TDD Integration Test Still Compiles and Passes

**Requirement:** The `tests/chicago_tdd_witness.rs` file must still compile (proving the dependency is real) and all tests must pass.

**Verification:**

```bash
cargo test --test chicago_tdd_witness
```

**Pass Criteria:**
- Exits `0`
- Both `chicago_tdd_asserts_honest_receipt_is_admitted` and `chicago_tdd_asserts_forged_receipt_is_refused` pass

---

## 6. Adversarial Test Scenarios

These tests verify the "certify, don't decide" doctrine: the verifier correctly REJECTs malformed receipts without needing to know anything about the underlying process.

Each adversarial scenario has a **"teeth" check**: the same receipt without the mutation ACCEPTs. A passing adversarial test where the untampered receipt also REJECTs is a false positive and must be investigated.

### 6.1 Tampered `event_type` → REJECT at `chain_integrity`

**Setup:**
```rust
let mut receipt = honest_receipt();
// Teeth: untampered ACCEPTs
assert!(verify(&receipt).accepted);
// Mutation: change event_type of event 1 without recomputing chain
receipt.events[1].event_type = "FORGED".to_string();
```

**Expected verdict:**
```
accepted: false
reason: "chain_integrity: chain hash mismatch: stored <X>, recomputed <Y>"
```

**Stage that must fail:** `chain_integrity` (`passed: false`)  
**Stages that must pass before it:** `decode`, `check_format`  
**Test:** `tests/adversarial.rs::tamper_commitment_rejects_at_chain_integrity` (commitment tamper — analogous; a dedicated `event_type` tamper test should also exist)

---

### 6.2 Duplicate `seq` Values → REJECT at `continuity`

**Setup:**
```rust
let mut receipt = honest_receipt(); // 3 events: seq 0, 1, 2
// Teeth
assert!(verify(&receipt).accepted);
// Mutation: give event 1 the same seq as event 0
receipt.events[1].seq = 0;
// Recompute chain so chain_integrity passes — only continuity should fail
receipt.chain_hash = recompute_chain(&receipt.events).expect("recompute");
```

**Expected verdict:**
```
accepted: false
reason: "continuity: seq gap at position 1: expected 1, found 0"
```

**Stage that must fail:** `continuity`  
**Stages before it:** `decode`, `check_format`, `chain_integrity` — all pass  
**Note:** After chain recomputation, chain_integrity passes. Only continuity catches the duplicate seq.

---

### 6.3 Malformed Commitment (Wrong Length) → REJECT at `verify_commitments`

**Setup:**
```rust
let mut receipt = honest_receipt();
assert!(verify(&receipt).accepted);
// Set commitment to a non-64-char hex string
receipt.events[0].payload_commitment = Blake3Hash::from_hex("abc"); // only 3 chars
// Recompute chain so prior stages pass
receipt.chain_hash = recompute_chain(&receipt.events).expect("recompute");
```

**Expected verdict:**
```
accepted: false
reason: "verify_commitments: event evt-0 has a malformed commitment (expected 64 lowercase hex chars)"
```

**Stage that must fail:** `verify_commitments`  
**Stages before it:** `decode`, `check_format`, `chain_integrity`, `continuity` — all pass  
**Note:** The commitment `"abc"` is 3 chars, not 64. `is_well_formed_hash` returns false.

---

### 6.4 Wrong `format_version` → REJECT at `check_format`

**Setup:**
```rust
let mut receipt = honest_receipt();
assert!(verify(&receipt).accepted);
// Change format version
receipt.format_version = "2.0.0".to_string();
// Do NOT recompute chain (format_version is not in the event chain)
```

**Expected verdict:**
```
accepted: false
reason: "check_format: expected format_version core/v1, found 2.0.0"
```

**Stage that must fail:** `check_format`  
**Stage that must pass before it:** `decode`  
**Test:** `tests/src/verifier.rs::verif_wrong_format_fails_check_format`

---

### 6.5 Empty Events List → REJECT at `continuity`

**Setup:**
```rust
let asm = ChainAssembler::new(); // no events appended
let receipt = asm.finalize();    // zero-event receipt
// The chain_hash equals genesis_hash() — chain_integrity passes
// continuity must catch the 0-event edge case
```

**Expected verdict:**
```
accepted: false
reason: "continuity: 0 event(s) with contiguous seq and unique ids"
```

OR, depending on how `stage_continuity` handles zero events, it may accept (zero events = vacuously contiguous). This scenario documents the **expected** policy:

**Policy (must be documented in `src/verifier.rs`):** A receipt with zero events must REJECT at `continuity` with message `"receipt has no events"` (or similar). If the current implementation accepts it, this is a bug that must be fixed before DoD sign-off.

**Pass Criteria (required for DoD):**
- `verify(&zero_event_receipt).accepted == false`
- The failing stage is `continuity` (or `evaluate_profile` if profile requires ≥1 event)

---

### 6.6 Duplicate Event ID → REJECT at `continuity`

**Setup:**
```rust
let mut receipt = honest_receipt();
assert!(verify(&receipt).accepted);
// Duplicate id without changing seq; recompute chain so chain_integrity passes
receipt.events[1].id = receipt.events[0].id.clone();
receipt.chain_hash = recompute_chain(&receipt.events).expect("recompute");
```

**Expected verdict:**
```
accepted: false
reason: "continuity: duplicate event id: evt-0"
```

**Stage that must fail:** `continuity`  
**Test:** `tests/src/verifier.rs::verif_duplicate_id_fails_continuity`

---

### 6.7 Seq Gap (Non-Contiguous) → REJECT at `continuity`

**Setup:**
```rust
let mut receipt = honest_receipt(); // events: seq 0, 1, 2
assert!(verify(&receipt).accepted);
// Create a gap: seq 0, 2, 2 (skipping 1)
receipt.events[1].seq = 2;
receipt.events[2].seq = 2;
receipt.chain_hash = recompute_chain(&receipt.events).expect("recompute");
```

**Expected verdict:**
```
accepted: false
reason: "continuity: seq gap at position 1: expected 1, found 2"
```

**Stage that must fail:** `continuity`  
**Test:** `tests/src/verifier.rs::verif_seq_gap_fails_continuity`

---

### 6.8 Uppercase Hex in Commitment → REJECT at `verify_commitments`

**Setup:**
```rust
let mut receipt = honest_receipt();
// Get a valid commitment and uppercase it
let lower = receipt.events[0].payload_commitment.as_hex().to_string();
let upper = lower.to_uppercase();
receipt.events[0].payload_commitment = Blake3Hash::from_hex(upper);
// Recompute chain
receipt.chain_hash = recompute_chain(&receipt.events).expect("recompute");
```

**Expected verdict:**
```
accepted: false
reason: "verify_commitments: event evt-0 has a malformed commitment (expected 64 lowercase hex chars)"
```

**Pass Criteria:** `is_well_formed_hash` checks `!c.is_uppercase()` — uppercase letters fail the well-formedness check even at correct length.

---

### 6.9 Empty `event_type` → REJECT at `evaluate_profile`

**Setup:**
```rust
let mut receipt = honest_receipt();
receipt.events[0].event_type = "".to_string();
// Recompute chain and commitment to pass prior stages
receipt.chain_hash = recompute_chain(&receipt.events).expect("recompute");
```

**Expected verdict:**
```
accepted: false
reason: "evaluate_profile: event evt-0 has an empty event_type"
```

**Stage that must fail:** `evaluate_profile` (Stage 6)  
**Stages before it:** `decode`, `check_format`, `chain_integrity`, `continuity`, `verify_commitments` — all pass

---

### 6.10 Objectless Event Blocked by OCEL Court (Not Verifier)

**Setup:** An event with `objects: []` that is chain-consistent. The affidavit verifier alone ACCEPTS it (empty objects is not a verifier rule). Only the wasm4pm-compat OCEL court, called from `admission::admit`, refuses it.

**Expected outcome:**
- `affidavit::verifier::verify(&receipt).accepted == true`
- `affidavit::admission::admit(receipt)` returns `Err(AffidavitRefusal::OcelLawViolation(OcelRefusal::EmptyEventObjectLinks))`
- `affi receipt verify objectless.json` exits non-zero and stderr contains `"EmptyEventObjectLinks"`

**Doctrine implication:** This scenario demonstrates the two-layer architecture. The verifier checks format; the court checks structural law. They are independent and composable.

---

## 7. Test Data Catalog

| Fixture Name | Event Count | Event Pattern | Object Pattern | Expected Verdict | Notes |
|---|---|---|---|---|---|
| `linear-1` | 1 | `op-0` | `obj-0:artifact` | ACCEPT | Minimal valid receipt |
| `linear-3` | 3 | `op-0, op-1, op-2` | `obj-N:artifact` | ACCEPT | Standard test fixture; used in most tests |
| `linear-10` | 10 | `op-0 … op-9` | `obj-N:artifact` | ACCEPT | Benchmark fixture for verifier pipeline |
| `linear-100` | 100 | `op-0 … op-99` | `obj-N:artifact` | ACCEPT | Benchmark fixture for memory checks |
| `lifecycle` | 3 | `create, transform, release` | `f:artifact, d:artifact, f:artifact` | ACCEPT | Golden run fixture; full lifecycle |
| `qualified` | 1 | `transform` | `dataset:artifact:input` | ACCEPT | Tests qualifier display in `show` |
| `stdin-payload` | 1 | `stdin_test` | `test:artifact` | ACCEPT | Tests `--payload -` stdin path |
| `tampered-event-type` | 3 | `op-0, FORGED, op-2` | `obj-N:artifact` | REJECT | chain_integrity fails; no chain recompute |
| `tampered-commitment` | 3 | `op-0, op-1 (flipped), op-2` | `obj-N:artifact` | REJECT | chain_integrity fails; commitment hex flipped |
| `seq-gap` | 3 | `op-0, op-1, op-2` (seq: 0, 5, 2) | `obj-N:artifact` | REJECT | continuity fails; seq 5 at position 1 |
| `dup-id` | 3 | `op-0, op-1, op-2` (ids: e0, e0, e2) | `obj-N:artifact` | REJECT | continuity fails; duplicate id `e0` |
| `bad-commitment-short` | 3 | `op-0, op-1, op-2` | `obj-N:artifact` | REJECT | verify_commitments fails; commitment `"abc"` |
| `bad-commitment-uppercase` | 3 | `op-0, op-1, op-2` | `obj-N:artifact` | REJECT | verify_commitments fails; uppercase hex |
| `wrong-format` | 3 | `op-0, op-1, op-2` | `obj-N:artifact` | REJECT | check_format fails; version `"2.0"` |
| `zero-events` | 0 | (none) | (none) | REJECT | continuity fails; empty receipt |
| `objectless` | 1 | `create` | `[]` (empty) | REJECT (court) | Passes verifier; fails OCEL court at admit() |
| `reordered` | 3 | `op-1, op-0, op-2` (swapped) | `obj-N:artifact` | REJECT | continuity and/or chain_integrity |
| `injected` | 4 | `op-0, op-1, op-2, FORGED` | `obj-N:artifact + evil:artifact` | REJECT | chain_integrity; chain hash not recomputed |
| `forged-json` | 3 | JSON with `event_type` replaced post-assembly | `obj-N:artifact` | REJECT (deser) | Fails at deserialization, not verifier |

**Fixture construction location:** All fixtures are constructed in-process by helper functions in each test file. No fixture files are checked into the repository.

**Reference construction (Rust):**
```rust
// All fixtures use this helper pattern
fn build_receipt_n(n: usize) -> Receipt { /* ... */ }
fn tampered_receipt() -> Vec<u8> { /* ... */ }
fn objectless_json() -> serde_json::Value { /* ... */ }
```

---

## 8. Acceptance Signoff Requirements

### 8.1 Signoff Parties

| Role | Responsibility | Sign-Off Scope |
|---|---|---|
| **Author / Developer** | Implements features; runs all tests locally | Full test suite passes on `claude/zen-cerf-oq87br` |
| **Reviewer** | Reviews code diff; verifies DoD completeness | UAT coverage matrix is complete; no test is weakened |
| **Security Reviewer** | Reviews adversarial scenarios and carrier guarantees | All 10 adversarial scenarios pass; `_seal` invariant holds |
| **Performance Reviewer** | Reviews benchmark results | All performance ceilings met; no regression vs. baseline |

### 8.2 Sign-Off Procedure

**Step 1: Pre-Merge Checklist (Author)**

```bash
# 1. All tests pass
cargo test --all
# Expected: "test result: ok. N passed; 0 failed"

# 2. Golden run passes
bash examples/golden_run.sh
# Expected: "=== GOLDEN RUN OK: ACCEPT(0) then REJECT(...) ==="

# 3. Benchmarks within ceiling
cargo bench --bench receipt_operations 2>&1 | grep -E "(time|regress)"
# Expected: no "Performance has regressed" lines

# 4. No raw payloads in receipts (spot check)
echo "SECRET" | affi receipt emit --type test --object obj:artifact --payload -
affi receipt assemble --out /tmp/signoff-check.json
grep -c "SECRET" /tmp/signoff-check.json  # must be 0

# 5. No clippy warnings
cargo clippy --all-targets --all-features -- -D warnings

# 6. Determinism check (10 runs)
for i in $(seq 1 10); do
  WORK=$(mktemp -d)
  cd $WORK
  echo "pin" | affi receipt emit --type test --object obj:artifact --payload -
  affi receipt assemble --out r.json
  md5sum r.json
  cd - > /dev/null
  rm -rf $WORK
done | awk '{print $1}' | sort -u | wc -l
# Expected: 1
```

**Step 2: Security Signoff**

The security reviewer verifies:

- [ ] `tests/adversarial.rs` — all tests pass
- [ ] `tests/chicago_tdd_witness.rs::chicago_tdd_asserts_forged_receipt_is_refused` — passes
- [ ] `tests/e2e.rs::e2e_objectless_receipt_rejected_by_ocel_court` — passes
- [ ] `src/types.rs::tests::deserialize_rejects_forged_receipt` — passes
- [ ] `Receipt { _seal: (), ... }` does not compile (trybuild test in `tests/ui/`)
- [ ] No `grep -rn "\.unwrap()" src/` hits in non-test code (except documented exceptions)
- [ ] `grep -c "TOP SECRET DATA" <assembled-receipt>.json` is `0` (no raw payloads)

**Step 3: Performance Signoff**

The performance reviewer verifies:

- [ ] `cargo bench` exits `0`
- [ ] `chain_append_single_event` estimate ≤ 500 µs
- [ ] `chain_finalize/100` estimate ≤ 100 ms
- [ ] `verifier_pipeline_10_events` estimate ≤ 150 ms
- [ ] No Criterion "Performance has regressed" output vs. `main` baseline

**Step 4: Sign-Off Record Format**

Create a sign-off comment on the PR with the following structure:

```markdown
## DoD Acceptance Sign-Off — affidavit DX/QOL 1000x

**Branch:** claude/zen-cerf-oq87br  
**Date:** YYYY-MM-DD  
**Signer:** <name> (<role>)

### Test Results

| Suite | Pass Count | Fail Count | Status |
|---|---|---|---|
| Unit (lib) | N | 0 | PASS |
| Dispatch | N | 0 | PASS |
| E2E | N | 0 | PASS |
| DX verbs | N | 0 | PASS |
| DX pipeline | N | 0 | PASS |
| Adversarial | N | 0 | PASS |
| Chicago-TDD | N | 0 | PASS |
| OTel | N | 0 | PASS |
| UI (trybuild) | N | 0 | PASS |
| **Total** | **N** | **0** | **PASS** |

### Golden Run

```
=== GOLDEN RUN OK: ACCEPT(0) then REJECT(N) ===
```

### Performance Ceilings

| Benchmark | Estimate | Ceiling | Status |
|---|---|---|---|
| chain_append_single_event | X µs | 500 µs | PASS/FAIL |
| chain_finalize/100 | X ms | 100 ms | PASS/FAIL |
| verifier_pipeline_10_events | X µs | 150 ms | PASS/FAIL |

### Security Checklist

- [ ] All adversarial tests pass
- [ ] `_seal` invariant holds (trybuild passes)
- [ ] No raw payloads in assembled receipts
- [ ] Forged JSON fails at deserialization (ADR-3)
- [ ] OCEL court refuses objectless events

### Determinism Check

10 consecutive runs produced identical chain_hash: YES / NO

### Sign-Off Decision

**APPROVED** / **BLOCKED** — <reason if blocked>
```

### 8.3 What Does NOT Require Sign-Off

- Adding new non-adversarial tests (always welcome, no review gate)
- Adding new examples in `examples/` (self-testing via `examples/golden_run.sh`)
- Documentation updates to `.md` files (no test-suite impact)
- Cargo.lock updates (no functional impact)

### 8.4 Blocking Conditions

The following conditions **must be remediated** before DoD sign-off will be granted:

1. **Any test in the 30+ baseline suite fails** — zero regressions policy
2. **Golden run fails** — the canonical lifecycle is broken
3. **`Receipt { _seal: (), ... }` compiles** — carrier invariant (ADR-2) is broken
4. **Any adversarial test accepts a tampered receipt** — verifier sensitivity failure
5. **Raw payload appears in assembled receipt JSON** — security invariant broken
6. **`chain_append_single_event` > 500 µs** — performance ceiling exceeded
7. **`verify` exits `0` on a tampered receipt** — REJECT mapping broken
8. **`--introspect` does not include all 11 verbs** — LLM tool surface incomplete
9. **`affi receipt verify` outputs to stdout instead of stderr** — output routing invariant broken
10. **`cargo clippy -- -D warnings` reports warnings** — code quality gate

---

*This specification is the authoritative Definition of Done for the affidavit DX/QOL 1000x initiative on branch `claude/zen-cerf-oq87br`. All UAT scenarios must pass before the branch may be merged to `main`.*

*Document version: 26.6.17 | Maintained by: Sean Chatman (xpointsh@gmail.com)*
