# Definition of Done — Phase 3: Test Generation & Mutation Testing

**Project:** affidavit — Provenance Layer  
**Phase:** 3 of the DX/QOL 1000x Initiative  
**Branch:** `claude/zen-cerf-oq87br`  
**Version:** 26.6.14  
**Date:** 2026-06-14  
**Priority:** P2 — Quality Assurance  
**Estimated Effort:** 16 hours across 5 features  

---

## Table of Contents

1. [Phase Overview](#phase-overview)
2. [Feature 1: `affi mutate receipt --count=N`](#feature-1-affi-mutate-receipt---countn)
3. [Feature 2: `affi generate test`](#feature-2-affi-generate-test)
4. [Feature 3: `affi generate snippet --pattern=<name>`](#feature-3-affi-generate-snippet---patternname)
5. [Feature 4: Property-Based Testing](#feature-4-property-based-testing)
6. [Feature 5: Test Fixture Database](#feature-5-test-fixture-database)
7. [Cross-Feature Integration DoD](#cross-feature-integration-dod)
8. [E2E Test Template: `tests/e2e_mutation.rs`](#e2e-test-template-testse2e_mutationrs)
9. [Tera Template: Test Code Generation](#tera-template-test-code-generation)
10. [Kill Rate Computation Formula](#kill-rate-computation-formula)
11. [Performance Baseline](#performance-baseline)
12. [Phase 3 Exit Gate Checklist](#phase-3-exit-gate-checklist)

---

## Phase Overview

Phase 3 hardens the verifier's **certify pipeline** by proving that malicious or accidental mutations to a receipt are caught. It introduces:

- Mutation operators (applying deliberate damage to receipts to test the verifier).
- Codegen tooling (auto-generating Rust test cases from chicago-tdd fixture patterns).
- A property-based test harness (quickcheck `Arbitrary` impls guaranteeing universal invariants).
- A fixture database (persistent, indexed receipt fixtures for reuse across tests and tools).

The doctrine: **certify, don't decide.** The verifier is not honest — it is decidable. Phase 3 proves every decidable check is **tight**: rejecting exactly what is wrong, and nothing more.

### New Modules

| Module | Purpose |
|--------|---------|
| `src/mutation.rs` | `MutationOperator` trait + 4 operator impls |
| `src/fixture_db.rs` | `FixtureDatabase` struct (insert/search/index) |
| `tests/property_based.rs` | quickcheck property tests for `Receipt` + `OperationEvent` |
| `tests/e2e_mutation.rs` | E2E test for mutation CLI command |

### Cargo.toml Changes

```toml
[features]
default = []
otel = []
mutation = []         # NEW: gates affi mutate verb + MutationOperator impls
fixture-db = []       # NEW: gates FixtureDatabase (JSON backend); enables sqlite sub-feature
fixture-db-sqlite = ["fixture-db", "dep:rusqlite"]  # optional SQLite backend

[dependencies]
# ... existing ...
tera = { version = "1", optional = true }          # Tera templates for codegen
rusqlite = { version = "0.31", optional = true }   # SQLite fixture backend

[dev-dependencies]
# ... existing ...
quickcheck = "1"
quickcheck_macros = "1"
```

---

## Feature 1: `affi mutate receipt --count=N`

### Scope

The `affi mutate receipt --count=N` command applies N independent mutations to an assembled receipt using 4 operators sourced from clnrm, then collects a verdict on each. Its primary use is **kill rate measurement**: proving the verifier rejects mutated receipts at ≥90%.

### MutationOperator Trait Definition

```rust
// src/mutation.rs

use crate::types::{OperationEvent, Receipt};
use anyhow::Result;

/// The four mutation classes, used as discriminants in diagnostic output.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MutationKind {
    /// One event was removed from the chain.
    EventDrop,
    /// Two adjacent events had their positions exchanged.
    EventReorder,
    /// One event's `event_type` field was replaced with a different string.
    TypeChange,
    /// One event's `payload_commitment` was replaced with a different hash.
    PayloadFlip,
}

/// A single applied mutation: the operator kind, the seq of the targeted event,
/// and the resulting receipt (chain_hash recomputed by the operator, not the
/// original receipt's chain_hash).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppliedMutation {
    /// Which operator was applied.
    pub kind: MutationKind,
    /// The `seq` of the event primarily affected (or the lower seq for reorder).
    pub target_seq: u64,
    /// The receipt produced after applying the mutation.
    /// Note: `chain_hash` in this receipt is the RECOMPUTED hash over the
    /// mutated events — not the original. The verifier must still detect this
    /// because structural invariants (continuity, commitments, profile) are
    /// broken by the mutation, independent of whether the chain_hash is fresh.
    pub mutated_receipt: Receipt,
}

/// A decidable, deterministic mutation of a Receipt into a new Receipt.
///
/// Operators receive a receipt and a seed (for deterministic operator selection
/// and target selection) and produce exactly one `AppliedMutation`.
///
/// # Contract
///
/// - **Deterministic:** Same `(receipt, seed)` → same `AppliedMutation`.
/// - **Total:** Must return `Ok` for any receipt with at least the minimum number
///   of events required by the operator (see `min_events`).
/// - **Chain-consistent:** The `mutated_receipt.chain_hash` is recomputed over
///   the mutated events via `chain::recompute_chain`. The mutation is structural,
///   not a hash-only forgery.
/// - **Distinct output:** `mutated_receipt != original` always holds (the mutation
///   actually changes the receipt bytes).
pub trait MutationOperator: Send + Sync + 'static {
    /// Human-readable name for diagnostic output (e.g., `"EventDrop"`).
    fn name(&self) -> &'static str;

    /// The `MutationKind` discriminant for this operator.
    fn kind(&self) -> MutationKind;

    /// Minimum number of events in the source receipt for this operator to apply.
    /// `EventDrop` requires ≥1; `EventReorder` requires ≥2; others require ≥1.
    fn min_events(&self) -> usize;

    /// Apply the mutation. `seed` determines which event is targeted.
    ///
    /// `seed % receipt.events.len()` selects the target event index.
    /// For `EventReorder`, `(seed % (receipt.events.len() - 1))` selects the
    /// lower of the two adjacent events to swap.
    fn apply(&self, receipt: &Receipt, seed: u64) -> Result<AppliedMutation>;
}

// --- Concrete operators ---

/// Drop one event at index `seed % len`, recompute seq for all remaining events
/// (re-number from 0), and recompute the chain hash.
pub struct EventDropOperator;

/// Swap adjacent events at indices `seed % (len-1)` and `(seed % (len-1)) + 1`.
/// Re-number their `seq` fields to their new positions, recompute chain hash.
pub struct EventReorderOperator;

/// Replace `events[seed % len].event_type` with `"mutated-type-<seed>"`.
/// Recompute chain hash over modified events.
pub struct TypeChangeOperator;

/// Replace `events[seed % len].payload_commitment` with
/// `Blake3Hash::from_bytes(b"mutated-payload-<seed>")`.
/// Recompute chain hash over modified events.
pub struct PayloadFlipOperator;
```

### Exact Operator Semantics

#### EventDrop

```
Input:  receipt with N events [e0, e1, ..., eN-1]
Target: i = seed % N
Action:
  1. Remove event at index i from the events list.
  2. Re-number seq fields: for all remaining events at new index j, set seq = j.
  3. Recompute chain_hash = chain::recompute_chain(&new_events).
  4. Build mutated_receipt via Receipt::sealed(FORMAT_VERSION, new_events, chain_hash).
Postcondition:
  - mutated_receipt.events.len() == N - 1
  - seq numbers are contiguous [0, N-2]
  - chain_hash is fresh (the verifier's chain_integrity check will PASS)
  - BUT: seq numbering is correct, so continuity will PASS too
  - Killed by: evaluate_profile if dropped event was only typed event;
    OR by chain_integrity in the ORIGINAL receipt context (wrong event count);
    OR by continuity if the original receipt had unique IDs that are now missing
    from external cross-references (out-of-scope for in-receipt checks).
  IMPORTANT: After EventDrop the mutated receipt may PASS the verifier if it
  has ≥1 valid remaining events. EventDrop is a SEMANTIC mutation (provenance
  gap), not a structural mutation caught by the 7-stage pipeline. This is by
  design: it represents a mutation that the verifier CANNOT catch (because the
  verifier certifies format, not completeness). EventDrop receipts ARE expected
  to PASS the verifier — this tests that EventDrop is a "surviving" mutation
  and the kill rate denominator accounts for it correctly.
```

#### EventReorder

```
Input:  receipt with N events [e0, e1, ..., eN-1], N ≥ 2
Target: i = seed % (N - 1)  [lower swap index]
Action:
  1. Swap events[i] and events[i+1] in the list.
  2. Re-number seq: events[i].seq = i, events[i+1].seq = i+1
     (seq tracks list position, not original identity).
  3. Recompute chain_hash = chain::recompute_chain(&swapped_events).
  4. Build mutated_receipt via Receipt::sealed(...).
Postcondition:
  - chain_hash is fresh.
  - seq numbers are contiguous.
  - event IDs at positions i and i+1 are swapped.
  - chain_integrity: PASS (hash was recomputed).
  - continuity: PASS (seq is contiguous, IDs are still unique).
  - evaluate_profile: PASS (types and commitments unchanged).
  IMPORTANT: EventReorder is also a SEMANTIC mutation — the ordering of events
  in the provenance record is wrong, but the verifier certifies format not
  ordering semantics. EventReorder receipts PASS the verifier. Accounted for
  in kill rate as a surviving mutation (see §Kill Rate Requirements).
```

#### TypeChange

```
Input:  receipt with N events
Target: i = seed % N
Action:
  1. Clone all events.
  2. Set events[i].event_type = format!("mutated-type-{seed}").
  3. Recompute chain_hash = chain::recompute_chain(&mutated_events).
  4. Build mutated_receipt via Receipt::sealed(...).
Postcondition:
  - event_type is now a non-empty string (so evaluate_profile stage PASSES).
  - chain_hash is fresh (chain_integrity PASSES).
  IMPORTANT: TypeChange is a SEMANTIC mutation that also passes the verifier.
  The verifier does not validate event_type against a whitelist (CoreV1 profile
  only requires event_type to be non-empty). TypeChange is a surviving mutation.
```

#### PayloadFlip

```
Input:  receipt with N events
Target: i = seed % N
Action:
  1. Clone all events.
  2. Set events[i].payload_commitment = Blake3Hash::from_bytes(
         format!("mutated-payload-{seed}").as_bytes()
     ).
  3. Recompute chain_hash = chain::recompute_chain(&mutated_events).
  4. Build mutated_receipt via Receipt::sealed(...).
Postcondition:
  - The new commitment is a valid BLAKE3 hex string (64 chars, lowercase).
  - chain_integrity: PASS (chain_hash was recomputed).
  - verify_commitments: PASS (well-formed hex).
  IMPORTANT: PayloadFlip is a SEMANTIC mutation. The commitment no longer
  matches the original payload bytes, but the verifier does not have access
  to raw payloads — it only checks format. PayloadFlip is a surviving mutation.
```

### Kill Rate Requirements

> **All four operators produce structurally valid mutated receipts** that pass the 7-stage format verifier. This is a fundamental property of the verifier's design: it certifies format, not honesty. The "kill rate" concept is redefined for affidavit as follows:

**Structural Kill Rate (SKR):** The fraction of mutations that are caught by the verifier, expected to be **~0%** for all four operators on well-formed receipts. This proves the verifier does NOT over-reject (false-positive-free).

**Semantic Survival Rate (SSR):** The fraction of mutations that survive the verifier (i.e., produce ACCEPT), expected to be **100%** for all four operators. This proves the verifier correctly certifies format without deciding honesty.

**Admission Kill Rate (AKR):** When mutated receipts are run through `admission::admit()` (which includes the OCEL structural law), the kill rate for EventDrop is expected to be **≥90%** because dropping an event may cause dangling object links — caught by the OCEL court, not the certify pipeline.

The kill rate computation formula is fully specified in the [Kill Rate Computation Formula](#kill-rate-computation-formula) section.

**Requirement Summary:**

| Operator | Certify Pipeline (7-stage) | Admission (OCEL + certify) |
|----------|---------------------------|---------------------------|
| EventDrop | ACCEPT (survival 100%) | REJECT ≥90% (OCEL court) |
| EventReorder | ACCEPT (survival 100%) | ACCEPT (no OCEL violation) |
| TypeChange | ACCEPT (survival 100%) | ACCEPT (no OCEL violation) |
| PayloadFlip | ACCEPT (survival 100%) | ACCEPT (no OCEL violation) |

### Acceptance Criteria

> Given/When/Then format. All 10 criteria must be met for Feature 1 to be done.

**AC-1.1: Feature gate compiles clean**
- **Given** the codebase without `--features mutation`
- **When** `cargo build` runs
- **Then** `src/mutation.rs` is conditionally compiled out and the build succeeds with zero errors or warnings

**AC-1.2: EventDrop produces one fewer event**
- **Given** a valid assembled receipt with 5 events
- **When** `EventDropOperator::apply(&receipt, seed=0)` is called
- **Then** `applied.mutated_receipt.events.len() == 4` and seq numbers are `[0, 1, 2, 3]`

**AC-1.3: EventReorder swaps two events at deterministic indices**
- **Given** a valid assembled receipt with 4 events having IDs `["e0","e1","e2","e3"]`
- **When** `EventReorderOperator::apply(&receipt, seed=1)` is called
- **Then** `applied.mutated_receipt.events[1].id == "e0"` and `applied.mutated_receipt.events[0].id == "e1"` (swap at index `1 % 3 == 1`)

**AC-1.4: TypeChange replaces event_type deterministically**
- **Given** a valid assembled receipt
- **When** `TypeChangeOperator::apply(&receipt, seed=42)` is called
- **Then** `applied.mutated_receipt.events[42 % len].event_type == "mutated-type-42"`

**AC-1.5: PayloadFlip replaces commitment and recomputes chain**
- **Given** a valid assembled receipt with original `chain_hash = H`
- **When** `PayloadFlipOperator::apply(&receipt, seed=0)` is called
- **Then** `applied.mutated_receipt.chain_hash != H` (chain changed) and `applied.mutated_receipt.events[0].payload_commitment.as_hex().len() == 64`

**AC-1.6: All mutated receipts pass the 7-stage verifier**
- **Given** a valid 5-event assembled receipt
- **When** all 4 operators are applied with seed 0 and the resulting receipts are passed through `verifier::verify()`
- **Then** all 4 verdicts have `accepted == true` (structural validity preserved)

**AC-1.7: CLI command emits JSON output with `--format=json`**
- **Given** a valid receipt at `./test.json`
- **When** `affi mutate receipt --count=4 --format=json ./test.json` is executed
- **Then** stdout is valid JSON with a top-level array of 4 objects, each containing `"kind"`, `"target_seq"`, and `"verdict"` fields

**AC-1.8: CLI `--count=N` applies exactly N mutations (one per operator cycling round-robin)**
- **Given** a valid receipt and `N = 8`
- **When** `affi mutate receipt --count=8 ./test.json`
- **Then** stdout lists 8 mutation results with kinds cycling `[EventDrop, EventReorder, TypeChange, PayloadFlip, EventDrop, EventReorder, TypeChange, PayloadFlip]`

**AC-1.9: Operator `min_events()` guard is enforced**
- **Given** a receipt with 1 event
- **When** `EventReorderOperator::apply(&receipt, 0)` is called
- **Then** returns `Err(...)` with a message containing `"requires at least 2 events"`

**AC-1.10: `AppliedMutation` serializes to canonical JSON**
- **Given** an `AppliedMutation` produced by any operator
- **When** `serde_json::to_string(&mutation)` is called
- **Then** the JSON contains `"kind"`, `"target_seq"`, and `"mutated_receipt"` fields with valid nested structure

### Test Evidence Required

- `tests/e2e_mutation.rs` — full E2E test (see template below)
- `src/mutation.rs` unit tests covering all 4 operators with `#[cfg(test)]` block
- `cargo test --features mutation mutation` passes
- `cargo test --features mutation e2e_mutation` passes
- `cargo clippy --features mutation -- -D warnings` passes

---

## Feature 2: `affi generate test`

### Scope

`affi generate test` reads chicago-tdd fixture definitions (patterns) and outputs compilable Rust test code using Tera templates. The generated code contains `#[test]` functions that build receipts from fixtures, run the verifier, and assert the expected verdict.

### Codegen Spec

#### Template Location

```
src/templates/test_fn.tera     # Tera template for a single #[test] function
src/templates/test_module.tera # Tera template for a full test module
```

#### Tera Template Example

```
{# src/templates/test_fn.tera #}
{# Context variables: pattern_name, event_count, expected_verdict, event_types, object_ids #}

#[test]
fn test_{{ pattern_name | replace(from="-", to="_") | replace(from=".", to="_") }}() {
    use affidavit::chain::ChainAssembler;
    use affidavit::ocel::{build_event, object_ref, SeqCounter};
    use affidavit::verifier::verify;

    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();

    {% for event in events %}
    {%- set obj_refs = event.objects | join(sep=", ") %}
    let event_{{ loop.index0 }} = build_event(
        "{{ event.event_type }}",
        vec![{% for obj in event.objects %}object_ref("{{ obj.id }}", "{{ obj.obj_type }}"){% if not loop.last %}, {% endif %}{% endfor %}],
        b"{{ event.payload }}",
        &mut counter,
    ).expect("build event {{ loop.index0 }}");
    asm.append(event_{{ loop.index0 }}).expect("append event {{ loop.index0 }}");
    {% endfor %}

    let receipt = asm.finalize();
    let verdict = verify(&receipt);

    {% if expected_verdict == "ACCEPT" %}
    assert!(verdict.accepted, "pattern {{ pattern_name }} must ACCEPT; reason: {}", verdict.reason);
    assert_eq!(verdict.reason, "all stages passed");
    {% else %}
    assert!(!verdict.accepted, "pattern {{ pattern_name }} must REJECT");
    assert!(verdict.reason.contains("{{ expected_failure_stage }}"),
        "expected failure at stage '{{ expected_failure_stage }}', got: {}", verdict.reason);
    {% endif %}
}
```

#### Test Module Template

```
{# src/templates/test_module.tera #}
//! Auto-generated test module from chicago-tdd fixtures.
//! Source pattern: {{ fixture_set_name }}
//! Generated by: affi generate test
//! Do not edit manually — re-run `affi generate test` to regenerate.

{% for test in tests %}
{{ test }}

{% endfor %}
```

#### Generated Output Requirements

A generated test function MUST contain:

1. `use affidavit::chain::ChainAssembler;`
2. `use affidavit::verifier::verify;`
3. `ChainAssembler::new()` call
4. At least one `build_event(...)` call
5. `asm.finalize()` call
6. `verify(&receipt)` call
7. An assertion on `verdict.accepted`
8. A function name matching `test_<pattern_name>` where `<pattern_name>` is the fixture pattern name with `-` and `.` replaced by `_`

#### CLI Interface

```bash
# Generate tests from all fixtures in the default fixture set
affi generate test

# Generate tests from a specific fixture file
affi generate test --fixture-file ./fixtures/chain_patterns.json

# Output to a specific file
affi generate test --out ./tests/generated_receipt_tests.rs

# Preview without writing (stdout)
affi generate test --dry-run
```

### Acceptance Criteria

**AC-2.1: Generated code compiles**
- **Given** `affi generate test --out ./tests/generated_tests.rs` is run against any fixture set
- **When** `cargo test --test generated_tests` is run
- **Then** compilation succeeds with zero errors

**AC-2.2: Generated ACCEPT tests pass**
- **Given** a fixture that produces a structurally valid receipt
- **When** `affi generate test` generates a test function for it
- **Then** the generated `#[test]` function passes (verdict.accepted == true)

**AC-2.3: Generated REJECT tests pass**
- **Given** a fixture that intentionally produces a malformed receipt (e.g., wrong format_version)
- **When** `affi generate test` generates a test function for it
- **Then** the generated `#[test]` function passes (verdict.accepted == false and expected stage is named)

**AC-2.4: Pattern name sanitization**
- **Given** a fixture pattern named `"chain-integrity.basic"`
- **When** a test function is generated for it
- **Then** the function is named `test_chain_integrity_basic` (dashes and dots become underscores)

**AC-2.5: `--dry-run` emits to stdout only**
- **Given** `affi generate test --dry-run` is run
- **When** the command exits
- **Then** no file is written to disk, and the generated Rust code appears on stdout

**AC-2.6: `--out` writes to the specified path**
- **Given** `affi generate test --out /tmp/generated.rs` is run
- **When** the command exits
- **Then** `/tmp/generated.rs` exists and contains valid Rust test code

**AC-2.7: Template errors are reported with location**
- **Given** a malformed Tera template (e.g., unclosed `{% if %}`)
- **When** `affi generate test` runs
- **Then** stderr shows `"template error at test_fn.tera line N: ..."` and exits non-zero

**AC-2.8: `--fixture-file` accepts a custom fixture JSON path**
- **Given** a custom fixture JSON at `./my_fixtures.json`
- **When** `affi generate test --fixture-file ./my_fixtures.json` is run
- **Then** the generated tests reflect the patterns defined in that file

**AC-2.9: Generator is idempotent**
- **Given** a stable fixture set
- **When** `affi generate test` is run twice
- **Then** the output files are byte-for-byte identical (deterministic generation)

**AC-2.10: Feature gate `tera` required**
- **Given** `cargo build` without `--features tera` (or equivalent feature enabling Tera)
- **When** the CLI runs `affi generate test`
- **Then** the command is either unavailable or returns a clear error `"feature 'tera' is required for test generation"`

### Test Evidence Required

- `tests/e2e_generate.rs` — E2E test running `affi generate test --dry-run` and asserting stdout contains `#[test]` and `fn test_`
- Compile-time test: generated output from the golden fixture set is committed to `tests/generated/` and `cargo test --test generated_receipt_tests` passes in CI
- `cargo test generate` passes

---

## Feature 3: `affi generate snippet --pattern=<name>`

### Scope

`affi generate snippet` searches the chicago-tdd example library by pattern name and emits a copy-paste-ready Rust code snippet. Snippets are shorter than full test functions: they show the key API calls for a pattern without the full test harness.

### Snippet Library Schema

Snippets are stored in `src/snippets/` as a JSON registry:

```json
{
  "snippets": [
    {
      "name": "chain-build-2-events",
      "tags": ["chain", "basic", "receipt"],
      "description": "Build a 2-event receipt using ChainAssembler",
      "language": "rust",
      "imports": [
        "affidavit::chain::ChainAssembler",
        "affidavit::ocel::{build_event, object_ref, SeqCounter}"
      ],
      "code": "let mut asm = ChainAssembler::new();\nlet mut counter = SeqCounter::new();\nlet e0 = build_event(\"build\", vec![object_ref(\"repo:main\", \"git\")], b\"payload-0\", &mut counter)?;\nasm.append(e0)?;\nlet receipt = asm.finalize();"
    },
    {
      "name": "verify-receipt",
      "tags": ["verify", "pipeline", "verdict"],
      "description": "Run the 7-stage certify pipeline on a receipt",
      "language": "rust",
      "imports": ["affidavit::verifier::verify"],
      "code": "let verdict = verify(&receipt);\nassert!(verdict.accepted, \"reason: {}\", verdict.reason);"
    },
    {
      "name": "tamper-detection",
      "tags": ["tamper", "chain-integrity", "mutation"],
      "description": "Demonstrate that tampering a commitment breaks chain integrity",
      "language": "rust",
      "imports": [
        "affidavit::types::Blake3Hash",
        "affidavit::verifier::verify"
      ],
      "code": "let mut tampered = receipt.clone();\ntampered.events[0].payload_commitment = Blake3Hash::from_bytes(b\"evil\");\nlet verdict = verify(&tampered);\nassert!(!verdict.accepted);\nassert_eq!(verdict.outcomes[2].stage, \"chain_integrity\");"
    }
  ]
}
```

### Acceptance Criteria

**AC-3.1: `--pattern=<name>` matches exact name**
- **Given** a snippet with name `"chain-build-2-events"` in the registry
- **When** `affi generate snippet --pattern=chain-build-2-events` is run
- **Then** stdout contains the snippet `code` field and the `imports` list prefixed with `use`

**AC-3.2: Partial pattern match**
- **Given** snippets with names `"chain-build-2-events"` and `"chain-build-5-events"`
- **When** `affi generate snippet --pattern=chain-build`
- **Then** stdout lists both matches with their names and descriptions (disambiguation mode)

**AC-3.3: No match returns helpful error**
- **Given** no snippet named `"nonexistent-pattern"` in the registry
- **When** `affi generate snippet --pattern=nonexistent-pattern`
- **Then** exit code is non-zero and stderr contains `"no snippet matching 'nonexistent-pattern'; try: affi generate snippet --list"`

**AC-3.4: `--list` enumerates all snippets**
- **Given** a registry with N snippets
- **When** `affi generate snippet --list`
- **Then** stdout lists N entries, each showing `name`, `tags`, and `description`

**AC-3.5: `--tag=<tag>` filters by tag**
- **Given** snippets tagged `["chain"]` and `["verify"]`
- **When** `affi generate snippet --tag=chain`
- **Then** only snippets with `"chain"` in their `tags` array are shown

**AC-3.6: Snippet output includes `use` declarations**
- **Given** a snippet with `imports: ["affidavit::verifier::verify"]`
- **When** `affi generate snippet --pattern=verify-receipt`
- **Then** stdout begins with `use affidavit::verifier::verify;`

**AC-3.7: `--format=json` emits raw snippet JSON**
- **Given** `affi generate snippet --pattern=verify-receipt --format=json`
- **When** the command exits
- **Then** stdout is valid JSON matching the snippet registry schema

**AC-3.8: Search is case-insensitive**
- **Given** a snippet named `"Tamper-Detection"` (mixed case)
- **When** `affi generate snippet --pattern=tamper-detection`
- **Then** the snippet is found and displayed

**AC-3.9: New snippets added to registry are discoverable immediately**
- **Given** a new snippet JSON object appended to `src/snippets/registry.json`
- **When** `affi generate snippet --list`
- **Then** the new snippet appears (no recompilation required for JSON-backed registry)

**AC-3.10: Registry schema validates at startup**
- **Given** a malformed `src/snippets/registry.json` (missing required `name` field)
- **When** any `affi generate snippet` command runs
- **Then** stderr shows `"snippet registry is malformed: ..."` and exits non-zero

### Test Evidence Required

- `tests/e2e_snippet.rs` — E2E test for `affi generate snippet --pattern=chain-build-2-events`
- `tests/e2e_snippet.rs` — Test for `--list` flag
- `tests/e2e_snippet.rs` — Test for no-match error message
- All registered snippets in `src/snippets/registry.json` must compile when wrapped in a test harness (verified by a compile-test in `tests/snippet_compile_test.rs`)

---

## Feature 4: Property-Based Testing

### Scope

`tests/property_based.rs` defines quickcheck `Arbitrary` implementations for `Receipt` and `OperationEvent`, then asserts universal properties (invariants that must hold for all valid inputs).

### Module: `tests/property_based.rs`

```rust
// tests/property_based.rs
// Property-based tests for affidavit Receipt and OperationEvent.
// Run: cargo test --test property_based

use affidavit::chain::{recompute_chain, ChainAssembler};
use affidavit::types::{Blake3Hash, ObjectRef, OperationEvent, Receipt};
use affidavit::verifier::verify;
use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};
use quickcheck_macros::quickcheck;
```

### Arbitrary Implementations

```rust
/// Arbitrary OperationEvent. Always generates well-formed events:
/// - non-empty `id` (prefix "evt-" + arbitrary u32)
/// - `seq` is provided by caller (not from Arbitrary, because seq must be
///   coordinated across a receipt; see ArbitraryReceipt below)
/// - non-empty `event_type` drawn from a small vocabulary
/// - ≥1 ObjectRef
/// - valid BLAKE3 commitment from_bytes of a random payload
impl Arbitrary for OperationEvent {
    fn arbitrary(g: &mut Gen) -> Self {
        let id = format!("evt-{}", u32::arbitrary(g));
        let seq = 0u64; // placeholder; overridden by receipt builder
        let vocab = ["build", "test", "audit", "deploy", "emit"];
        let type_idx = usize::arbitrary(g) % vocab.len();
        let event_type = vocab[type_idx].to_string();
        let payload = Vec::<u8>::arbitrary(g);
        let commitment = Blake3Hash::from_bytes(if payload.is_empty() { b"empty" } else { &payload });
        let obj = ObjectRef {
            id: format!("obj-{}", u32::arbitrary(g)),
            obj_type: "artifact".to_string(),
            qualifier: None,
        };
        OperationEvent { id, seq, event_type, objects: vec![obj], payload_commitment: commitment }
    }
}

/// Arbitrary Receipt. Builds a well-formed sealed receipt from 1–10 events.
/// Seq numbers are assigned monotonically [0, N-1].
/// chain_hash is computed via ChainAssembler::finalize.
pub struct ArbitraryReceipt(pub Receipt);

impl Arbitrary for ArbitraryReceipt {
    fn arbitrary(g: &mut Gen) -> Self {
        let n = (usize::arbitrary(g) % 9) + 1; // 1–10 events
        let mut asm = ChainAssembler::new();
        let mut seen_ids = std::collections::HashSet::new();
        for i in 0..n {
            let mut ev = OperationEvent::arbitrary(g);
            ev.seq = i as u64;
            // Guarantee unique IDs within the receipt
            ev.id = format!("evt-{i}-{}", u32::arbitrary(g));
            while seen_ids.contains(&ev.id) {
                ev.id = format!("evt-{i}-{}", u32::arbitrary(g));
            }
            seen_ids.insert(ev.id.clone());
            asm.append(ev).expect("append");
        }
        ArbitraryReceipt(asm.finalize())
    }
}
```

### Property Invariants

Every property below must hold for **all** inputs generated by the `Arbitrary` impls. Properties are grouped by concern.

#### Group A: Verifier Decidability

**PROP-A1: `verify` always returns (never panics)**
- For all `ArbitraryReceipt(r)`, `verify(&r)` terminates and returns a `Verdict`.
- No input causes a panic, infinite loop, or stack overflow.

**PROP-A2: `verify` is pure (same input → same output)**
- For all `ArbitraryReceipt(r)`, `verify(&r) == verify(&r)`.

**PROP-A3: `verify` on a canonically assembled receipt always ACCEPTs**
- For all `ArbitraryReceipt(r)` (built via `ChainAssembler::finalize`), `verify(&r).accepted == true`.

**PROP-A4: `verdict.outcomes` has exactly 6 entries**
- For all `ArbitraryReceipt(r)`, `verify(&r).outcomes.len() == 6`.
- (Stages 1–6; stage 7 is the terminal verdict, not a separate outcome.)

**PROP-A5: `verdict.accepted` is the conjunction of all `outcome.passed` values**
- For all `ArbitraryReceipt(r)`:
  `verify(&r).accepted == verify(&r).outcomes.iter().all(|o| o.passed)`.

#### Group B: Chain Integrity

**PROP-B1: `recompute_chain` on a canonical receipt matches stored `chain_hash`**
- For all `ArbitraryReceipt(r)`:
  `recompute_chain(&r.events).unwrap() == r.chain_hash`.

**PROP-B2: Mutating any event byte causes `recompute_chain` to differ from stored hash**
- For all `ArbitraryReceipt(r)` with `r.events.len() >= 1`:
  Let `mut r2 = r.clone(); r2.events[0].payload_commitment = Blake3Hash::from_bytes(b"evil");`
  Then `recompute_chain(&r2.events).unwrap() != r.chain_hash`.

**PROP-B3: `recompute_chain` is deterministic**
- For all `Vec<OperationEvent>` slice `evs`:
  `recompute_chain(&evs) == recompute_chain(&evs)`.

**PROP-B4: Empty event list produces genesis hash**
- `recompute_chain(&[]).unwrap() == Blake3Hash::from_bytes(chain::GENESIS_SEED)`.

#### Group C: Fixture Receipts Pass, Tampered Receipts Fail

**PROP-C1: All ArbitraryReceipt instances pass**
- For all `ArbitraryReceipt(r)`, `verify(&r).accepted == true`.
- (Restates PROP-A3 for clarity as a separate test target.)

**PROP-C2: PayloadFlip-mutated receipts produce fresh chain hash that differs from original**
- For all `ArbitraryReceipt(r)`:
  Let `flipped = PayloadFlipOperator.apply(&r, 0).unwrap();`
  Then `flipped.mutated_receipt.chain_hash != r.chain_hash`.

**PROP-C3: Tampered receipts (commitment replaced, chain_hash NOT recomputed) REJECT**
- For all `ArbitraryReceipt(r)` with `r.events.len() >= 1`:
  Build `tampered` by cloning `r`, replacing `events[0].payload_commitment`,
  and leaving `chain_hash` as the original.
  Then `verify(&tampered).accepted == false`.

**PROP-C4: Receipt with wrong `format_version` always REJECTs**
- For all `ArbitraryReceipt(r)`:
  Let `mut r2 = r.clone(); r2.format_version = "wrong/v99".to_string();`
  Then `verify(&r2).accepted == false`.

#### Group D: Continuity

**PROP-D1: Receipts from `ChainAssembler` always have contiguous seq from 0**
- For all `ArbitraryReceipt(r)`:
  For all `(i, e)` in `r.events.iter().enumerate()`: `e.seq == i as u64`.

**PROP-D2: Receipts from `ChainAssembler` always have unique event IDs**
- For all `ArbitraryReceipt(r)`:
  Let ids = r.events.iter().map(|e| &e.id).collect::<BTreeSet<_>>();
  `ids.len() == r.events.len()`.

#### Group E: FixtureDatabase Properties (when `fixture-db` feature enabled)

**PROP-E1: Insert then search by ID always returns the original**
- For all `ArbitraryReceipt(r)`:
  Insert `r` into a fresh `FixtureDatabase`.
  Search by `r.chain_hash.as_hex()` returns exactly one result equal to `r`.

**PROP-E2: Search on empty database returns empty**
- For all `String query`:
  A fresh `FixtureDatabase` returns an empty result set for any query.

### Acceptance Criteria

**AC-4.1: All property tests pass with 100 quickcheck iterations (default)**
- `cargo test --test property_based` exits 0

**AC-4.2: Properties A1–A5 are tested**
- Functions `prop_verify_always_returns`, `prop_verify_pure`, `prop_verify_accepts_canonical`, `prop_verify_six_outcomes`, `prop_accepted_is_conjunction` exist and pass

**AC-4.3: Properties B1–B4 are tested**
- Functions `prop_recompute_matches_stored`, `prop_mutation_changes_recomputed`, `prop_recompute_deterministic`, `prop_empty_is_genesis` exist and pass

**AC-4.4: Properties C1–C4 are tested**
- Functions `prop_arbitrary_receipt_passes`, `prop_payload_flip_changes_chain`, `prop_tampered_no_recompute_rejects`, `prop_wrong_format_rejects` exist and pass

**AC-4.5: Properties D1–D2 are tested**
- Functions `prop_seq_is_contiguous`, `prop_ids_are_unique` exist and pass

**AC-4.6: `ArbitraryReceipt` generates 1–10 events**
- Verified by a `#[test]` in `tests/property_based.rs` that runs 1000 samples and asserts `1 <= r.events.len() <= 10`

**AC-4.7: `#[quickcheck]` macro is used for at least 8 properties**
- The `quickcheck_macros` crate is listed in `[dev-dependencies]` and `#[quickcheck]` attribute appears on ≥8 property functions

**AC-4.8: `QUICKCHECK_TESTS=500 cargo test` passes**
- Running with `QUICKCHECK_TESTS=500` (5x default) does not produce any failure or flake

**AC-4.9: Panic-freedom property is explicitly tested**
- A test uses `std::panic::catch_unwind` to confirm `verify(&r)` never panics for arbitrary receipts

**AC-4.10: `tests/property_based.rs` has a module-level doc comment**
- The file begins with `//! Property-based tests...` explaining what invariants are covered

### Test Evidence Required

- `cargo test --test property_based` passes
- `QUICKCHECK_TESTS=500 cargo test --test property_based` passes
- All property function names listed in PROP-A through PROP-D appear in `tests/property_based.rs`
- No `#[allow(unused)]` suppression on any property function

---

## Feature 5: Test Fixture Database

### Scope

`src/fixture_db.rs` provides a `FixtureDatabase` struct that stores assembled receipts as named fixtures, indexes them by event count and event type, and supports search. The JSON backend is always available; the SQLite backend is gated behind `feature = "fixture-db-sqlite"`.

### Fixture DB Schema

#### JSON Backend (`src/fixture_db.rs` default)

The JSON backend stores fixtures in a single file, typically `<working-dir>/.affi/fixtures.json`.

**Top-level schema:**

```json
{
  "schema_version": "1",
  "fixtures": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "basic-2-event",
      "tags": ["basic", "chain", "core/v1"],
      "event_count": 2,
      "event_types": ["build", "test"],
      "chain_hash": "2aae6c69...",
      "inserted_at": "2026-06-14T00:00:00Z",
      "receipt": { ... full Receipt JSON ... }
    }
  ],
  "indexes": {
    "by_name": { "basic-2-event": 0 },
    "by_chain_hash": { "2aae6c69...": 0 },
    "by_event_count": { "2": [0] },
    "by_event_type": { "build": [0], "test": [0] }
  }
}
```

**Field specification:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | UUID string | yes | Stable surrogate key; never changes after insert |
| `name` | string | yes | Human-readable fixture name; must be unique within the DB |
| `tags` | `[string]` | yes | Searchable tags; may be empty array |
| `event_count` | integer | yes | `receipt.events.len()` — denormalized for index efficiency |
| `event_types` | `[string]` | yes | Sorted unique event types from all events — denormalized |
| `chain_hash` | hex string (64 chars) | yes | `receipt.chain_hash.as_hex()` — indexed for dedup |
| `inserted_at` | ISO 8601 UTC | yes | Timestamp of insertion (informational only) |
| `receipt` | `Receipt` JSON | yes | Full serialized receipt |

**Index design:**

| Index | Key | Value | Purpose |
|-------|-----|-------|---------|
| `by_name` | fixture name | array index | O(1) lookup by name |
| `by_chain_hash` | chain_hash hex | array index | O(1) dedup check |
| `by_event_count` | event count as string | `[array indices]` | Filter by size |
| `by_event_type` | event type string | `[array indices]` | Filter by what events happened |

#### SQLite Backend (optional, `feature = "fixture-db-sqlite"`)

```sql
CREATE TABLE IF NOT EXISTS fixtures (
    id          TEXT PRIMARY KEY,      -- UUID
    name        TEXT NOT NULL UNIQUE,  -- human-readable key
    event_count INTEGER NOT NULL,
    chain_hash  TEXT NOT NULL UNIQUE,  -- 64-char hex; dedup key
    inserted_at TEXT NOT NULL,         -- ISO 8601 UTC
    receipt_json TEXT NOT NULL         -- full Receipt serialized as JSON
);

CREATE TABLE IF NOT EXISTS fixture_tags (
    fixture_id  TEXT NOT NULL REFERENCES fixtures(id) ON DELETE CASCADE,
    tag         TEXT NOT NULL,
    PRIMARY KEY (fixture_id, tag)
);

CREATE TABLE IF NOT EXISTS fixture_event_types (
    fixture_id  TEXT NOT NULL REFERENCES fixtures(id) ON DELETE CASCADE,
    event_type  TEXT NOT NULL,
    PRIMARY KEY (fixture_id, event_type)
);

CREATE INDEX IF NOT EXISTS idx_fixtures_event_count ON fixtures (event_count);
CREATE INDEX IF NOT EXISTS idx_fixture_tags_tag ON fixture_tags (tag);
CREATE INDEX IF NOT EXISTS idx_fixture_event_types_type ON fixture_event_types (event_type);
```

### Fixture DB Query Interface

```rust
// src/fixture_db.rs

use crate::types::Receipt;
use anyhow::Result;

/// A stored fixture: metadata + the full receipt.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Fixture {
    pub id: String,
    pub name: String,
    pub tags: Vec<String>,
    pub event_count: usize,
    pub event_types: Vec<String>,
    pub chain_hash: String,
    pub inserted_at: String,
    pub receipt: Receipt,
}

/// Search filter for fixture queries.
#[derive(Debug, Default)]
pub struct FixtureQuery {
    /// Filter by exact fixture name (case-insensitive substring match).
    pub name_contains: Option<String>,
    /// Filter by exact tag membership.
    pub tag: Option<String>,
    /// Filter by event count (inclusive range).
    pub min_events: Option<usize>,
    pub max_events: Option<usize>,
    /// Filter by event type membership (any event in receipt has this type).
    pub event_type: Option<String>,
    /// Maximum number of results to return.
    pub limit: Option<usize>,
}

/// An append-indexed, searchable store of receipt fixtures.
///
/// # JSON Backend (default)
///
/// Backed by a single JSON file. Load the database from disk with [`FixtureDatabase::open`],
/// mutate it with [`FixtureDatabase::insert`], and flush with [`FixtureDatabase::save`].
///
/// # SQLite Backend (`feature = "fixture-db-sqlite"`)
///
/// Backed by a SQLite file. Mutations are committed immediately.
pub struct FixtureDatabase {
    path: std::path::PathBuf,
    #[cfg(not(feature = "fixture-db-sqlite"))]
    inner: JsonFixtureStore,
    #[cfg(feature = "fixture-db-sqlite")]
    inner: SqliteFixtureStore,
}

impl FixtureDatabase {
    /// Open (or create) a fixture database at `path`.
    pub fn open(path: impl Into<std::path::PathBuf>) -> Result<Self>;

    /// Insert a receipt as a named fixture.
    ///
    /// Returns `Err` if a fixture with the same `chain_hash` already exists
    /// (dedup guard). Tags are optional.
    pub fn insert(&mut self, name: &str, tags: &[&str], receipt: Receipt) -> Result<Fixture>;

    /// Find fixtures matching `query`. Returns results in insertion order.
    pub fn search(&self, query: &FixtureQuery) -> Result<Vec<Fixture>>;

    /// Retrieve a fixture by exact name.
    pub fn get_by_name(&self, name: &str) -> Result<Option<Fixture>>;

    /// Retrieve a fixture by chain_hash hex string.
    pub fn get_by_chain_hash(&self, chain_hash: &str) -> Result<Option<Fixture>>;

    /// Return all fixtures, in insertion order.
    pub fn all(&self) -> Result<Vec<Fixture>>;

    /// Number of stored fixtures.
    pub fn len(&self) -> usize;

    /// Whether the database is empty.
    pub fn is_empty(&self) -> bool;

    /// Flush in-memory state to disk (JSON backend only; SQLite commits immediately).
    pub fn save(&self) -> Result<()>;

    /// Delete a fixture by name. Returns `Ok(true)` if it was found and deleted.
    pub fn delete_by_name(&mut self, name: &str) -> Result<bool>;

    /// Rebuild all indexes from the stored fixtures. Call after external edits to the JSON file.
    pub fn reindex(&mut self) -> Result<()>;
}
```

### Acceptance Criteria

**AC-5.1: Insert and retrieve by name**
- **Given** a fresh `FixtureDatabase`
- **When** `db.insert("my-fixture", &["basic"], receipt.clone())` is called
- **Then** `db.get_by_name("my-fixture").unwrap().unwrap().receipt == receipt`

**AC-5.2: Dedup by `chain_hash`**
- **Given** a `FixtureDatabase` with fixture `"fx-1"` having `chain_hash = H`
- **When** `db.insert("fx-2", &[], another_receipt_with_same_chain_hash)` is called
- **Then** returns `Err(...)` containing `"already exists"`

**AC-5.3: Search by tag returns only matching fixtures**
- **Given** fixtures `A` (tags: `["basic"]`) and `B` (tags: `["advanced"]`)
- **When** `db.search(&FixtureQuery { tag: Some("basic".into()), ..Default::default() })`
- **Then** result contains `A` only

**AC-5.4: Search by `event_type` returns only matching fixtures**
- **Given** fixture `A` with events `["build", "test"]` and fixture `B` with events `["deploy"]`
- **When** `db.search(&FixtureQuery { event_type: Some("build".into()), ..Default::default() })`
- **Then** result contains `A` only

**AC-5.5: Search by `min_events` / `max_events` range**
- **Given** fixtures with event counts `[1, 3, 7, 10]`
- **When** `db.search(&FixtureQuery { min_events: Some(3), max_events: Some(7), ..Default::default() })`
- **Then** result contains the fixtures with counts `[3, 7]`

**AC-5.6: `save` and `open` round-trip (JSON backend)**
- **Given** a `FixtureDatabase` with 3 inserted fixtures saved to `path`
- **When** `FixtureDatabase::open(path)` is called on a fresh process
- **Then** `db.len() == 3` and all fixtures match the originals

**AC-5.7: `limit` in `FixtureQuery` is respected**
- **Given** a database with 10 fixtures matching a query
- **When** `db.search(&FixtureQuery { limit: Some(3), ..Default::default() })`
- **Then** result.len() == 3

**AC-5.8: `delete_by_name` removes fixture and updates indexes**
- **Given** a `FixtureDatabase` with fixture `"fx-to-delete"` stored and indexed
- **When** `db.delete_by_name("fx-to-delete")` is called
- **Then** `db.get_by_name("fx-to-delete").unwrap() == None` and `db.search(...)` does not return it

**AC-5.9: `reindex` recovers from a corrupted index**
- **Given** a `FixtureDatabase` whose `indexes` section was cleared (e.g., by manual JSON edit)
- **When** `db.reindex()` is called
- **Then** all searches work correctly again (indexes rebuilt from `fixtures` array)

**AC-5.10: Search performance: <10ms for 1000 fixtures**
- **Given** a `FixtureDatabase` with 1000 inserted fixtures
- **When** `db.search(&FixtureQuery { tag: Some("basic".into()), ..Default::default() })`
- **Then** the call completes in <10ms (measured by `std::time::Instant`)

### Test Evidence Required

- `src/fixture_db.rs` unit test block covering AC-5.1 through AC-5.9
- `tests/fixture_db_perf.rs` — performance test for AC-5.10
- `cargo test --features fixture-db fixture_db` passes
- `cargo test --features fixture-db-sqlite fixture_db_sqlite` passes (when SQLite backend is implemented)

---

## Cross-Feature Integration DoD

These criteria apply to Phase 3 as a whole and must be satisfied before the phase is marked complete.

### CI / Cargo

- [ ] `cargo build --features mutation,fixture-db` succeeds with zero warnings (`-D warnings` clean)
- [ ] `cargo test` (no feature flags) succeeds — no feature-gated code leaks into default build
- [ ] `cargo test --features mutation,fixture-db` — all new tests pass
- [ ] `cargo clippy --features mutation,fixture-db -- -D warnings` is clean
- [ ] `cargo fmt --check` passes on all new source files

### Documentation

- [ ] `src/mutation.rs` has module-level `//!` doc comment explaining doctrine
- [ ] `src/fixture_db.rs` has module-level `//!` doc comment explaining backends
- [ ] `tests/property_based.rs` has module-level `//!` doc comment listing all invariant groups
- [ ] All public types and functions in new modules have `///` doc comments
- [ ] `cargo doc --no-deps --features mutation,fixture-db` succeeds with no warnings

### Security / Correctness

- [ ] No `unwrap()` in non-test code (enforced by `clippy::unwrap_used` lint in `mutation.rs` and `fixture_db.rs`)
- [ ] No unsafe code in new modules
- [ ] `MutationOperator::apply` never modifies the original receipt (takes `&Receipt`, not `&mut Receipt`)
- [ ] `FixtureDatabase::insert` stores a serialized copy of the receipt, not a borrow

### Integration Between Features

- [ ] `affi mutate receipt` can accept the output path of a fixture from `FixtureDatabase` (CLI: `affi mutate receipt $(affi fixture get-path my-fixture)`)
- [ ] `affi generate test` can generate test cases from fixtures stored in a `FixtureDatabase`
- [ ] `tests/property_based.rs` imports `ArbitraryReceipt` (not a raw `Receipt` literal), ensuring the quickcheck properties cover the full `ChainAssembler`-produced space

---

## E2E Test Template: `tests/e2e_mutation.rs`

This is the canonical E2E test file for Feature 1. It must exist at this exact path, compile under `--features mutation`, and pass when run with `cargo test --features mutation --test e2e_mutation`.

```rust
//! E2E tests for `affi mutate receipt --count=N`.
//!
//! These tests invoke the compiled `affi` binary via `assert_cmd` and validate
//! that:
//! - mutations are applied and output correctly
//! - all mutated receipts pass the 7-stage certify pipeline (structural validity)
//! - the CLI exits 0 on success, non-zero on error
//! - JSON output conforms to the AppliedMutation schema

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use affidavit::chain::ChainAssembler;
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::{Blake3Hash, OperationEvent};
use affidavit::verifier::verify;

/// Build a valid N-event receipt and return the path to its JSON file.
fn write_receipt(dir: &TempDir, n: usize) -> std::path::PathBuf {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    for i in 0..n {
        let event_type = ["build", "test", "audit", "deploy"][i % 4];
        let event = build_event(
            event_type,
            vec![object_ref(&format!("obj-{i}"), "artifact")],
            format!("payload-{i}").as_bytes(),
            &mut counter,
        ).expect("build event");
        asm.append(event).expect("append");
    }
    let receipt = asm.finalize();
    let path = dir.path().join(format!("receipt_{n}.json"));
    let bytes = affidavit::chain::serialize_receipt(&receipt).expect("serialize");
    std::fs::write(&path, bytes).expect("write receipt");
    path
}

// ---------------------------------------------------------------------------
// Test: basic invocation produces output
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "mutation")]
fn e2e_mutate_count_4_produces_4_lines() {
    let dir = TempDir::new().unwrap();
    let receipt_path = write_receipt(&dir, 5);

    Command::cargo_bin("affi")
        .unwrap()
        .args(["mutate", "receipt", "--count=4", receipt_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("EventDrop"))
        .stdout(predicate::str::contains("EventReorder"))
        .stdout(predicate::str::contains("TypeChange"))
        .stdout(predicate::str::contains("PayloadFlip"));
}

// ---------------------------------------------------------------------------
// Test: --format=json produces valid JSON array
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "mutation")]
fn e2e_mutate_json_output_is_valid() {
    let dir = TempDir::new().unwrap();
    let receipt_path = write_receipt(&dir, 3);

    let output = Command::cargo_bin("affi")
        .unwrap()
        .args(["mutate", "receipt", "--count=4", "--format=json", receipt_path.to_str().unwrap()])
        .output()
        .expect("run affi");

    assert!(output.status.success(), "exit code must be 0");

    let stdout = std::str::from_utf8(&output.stdout).expect("valid utf8");
    let parsed: serde_json::Value = serde_json::from_str(stdout)
        .expect("stdout must be valid JSON");

    let arr = parsed.as_array().expect("top-level must be JSON array");
    assert_eq!(arr.len(), 4, "must have exactly 4 mutation results");

    for item in arr {
        assert!(item.get("kind").is_some(), "each item must have 'kind'");
        assert!(item.get("target_seq").is_some(), "each item must have 'target_seq'");
        assert!(item.get("verdict").is_some(), "each item must have 'verdict'");
    }
}

// ---------------------------------------------------------------------------
// Test: all mutated receipts pass the 7-stage verifier (structural validity)
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "mutation")]
fn e2e_mutated_receipts_pass_verifier() {
    use affidavit::mutation::{
        EventDropOperator, EventReorderOperator, MutationOperator,
        PayloadFlipOperator, TypeChangeOperator,
    };
    use affidavit::chain::deserialize_receipt;

    let dir = TempDir::new().unwrap();
    let receipt_path = write_receipt(&dir, 5);
    let bytes = std::fs::read(&receipt_path).expect("read receipt");
    let receipt = deserialize_receipt(&bytes).expect("deserialize");

    let operators: Vec<Box<dyn MutationOperator>> = vec![
        Box::new(EventDropOperator),
        Box::new(EventReorderOperator),
        Box::new(TypeChangeOperator),
        Box::new(PayloadFlipOperator),
    ];

    for (seed, op) in operators.iter().enumerate() {
        let applied = op.apply(&receipt, seed as u64).expect("apply mutation");
        let verdict = verify(&applied.mutated_receipt);
        assert!(
            verdict.accepted,
            "operator {} (seed={}) produced a structurally invalid receipt: {}",
            op.name(), seed, verdict.reason
        );
    }
}

// ---------------------------------------------------------------------------
// Test: operators cycle round-robin when --count exceeds 4
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "mutation")]
fn e2e_mutate_count_8_cycles_operators() {
    let dir = TempDir::new().unwrap();
    let receipt_path = write_receipt(&dir, 5);

    let output = Command::cargo_bin("affi")
        .unwrap()
        .args(["mutate", "receipt", "--count=8", "--format=json", receipt_path.to_str().unwrap()])
        .output()
        .expect("run affi");

    assert!(output.status.success());
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let arr: serde_json::Value = serde_json::from_str(stdout).unwrap();
    let arr = arr.as_array().unwrap();

    assert_eq!(arr.len(), 8);
    let expected_kinds = [
        "EventDrop", "EventReorder", "TypeChange", "PayloadFlip",
        "EventDrop", "EventReorder", "TypeChange", "PayloadFlip",
    ];
    for (i, item) in arr.iter().enumerate() {
        let kind = item["kind"].as_str().unwrap();
        assert_eq!(kind, expected_kinds[i], "position {i}: expected {}, got {kind}", expected_kinds[i]);
    }
}

// ---------------------------------------------------------------------------
// Test: receipt with 1 event — EventDrop produces empty events list
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "mutation")]
fn e2e_event_drop_on_single_event_receipt() {
    use affidavit::mutation::{EventDropOperator, MutationOperator};
    use affidavit::chain::deserialize_receipt;

    let dir = TempDir::new().unwrap();
    let receipt_path = write_receipt(&dir, 1);
    let bytes = std::fs::read(&receipt_path).unwrap();
    let receipt = deserialize_receipt(&bytes).unwrap();

    let op = EventDropOperator;
    let applied = op.apply(&receipt, 0).expect("apply drop on 1-event receipt");
    assert_eq!(applied.mutated_receipt.events.len(), 0, "dropped the only event");
    assert_eq!(applied.target_seq, 0, "targeted seq 0");
    assert_eq!(applied.kind, affidavit::mutation::MutationKind::EventDrop);
}

// ---------------------------------------------------------------------------
// Test: EventReorder on 1-event receipt returns Err (min_events guard)
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "mutation")]
fn e2e_event_reorder_min_events_guard() {
    use affidavit::mutation::{EventReorderOperator, MutationOperator};
    use affidavit::chain::deserialize_receipt;

    let dir = TempDir::new().unwrap();
    let receipt_path = write_receipt(&dir, 1);
    let bytes = std::fs::read(&receipt_path).unwrap();
    let receipt = deserialize_receipt(&bytes).unwrap();

    let op = EventReorderOperator;
    let result = op.apply(&receipt, 0);
    assert!(result.is_err(), "EventReorder on 1-event receipt must return Err");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("requires at least 2 events"),
        "error must mention min_events: got '{msg}'"
    );
}

// ---------------------------------------------------------------------------
// Test: TypeChange mutation changes event_type and preserves valid hex commitment
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "mutation")]
fn e2e_type_change_produces_valid_commitment() {
    use affidavit::mutation::{MutationKind, MutationOperator, TypeChangeOperator};
    use affidavit::chain::deserialize_receipt;

    let dir = TempDir::new().unwrap();
    let receipt_path = write_receipt(&dir, 3);
    let bytes = std::fs::read(&receipt_path).unwrap();
    let receipt = deserialize_receipt(&bytes).unwrap();

    let original_type = receipt.events[0].event_type.clone();
    let op = TypeChangeOperator;
    let applied = op.apply(&receipt, 0).unwrap();

    assert_eq!(applied.kind, MutationKind::TypeChange);
    let mutated_type = &applied.mutated_receipt.events[0].event_type;
    assert_ne!(mutated_type, &original_type, "type must change");
    assert!(mutated_type.starts_with("mutated-type-"), "type format: {mutated_type}");

    // Commitment must still be valid hex
    let hex = applied.mutated_receipt.events[0].payload_commitment.as_hex();
    assert_eq!(hex.len(), 64, "commitment must be 64 hex chars");
    assert!(hex.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
        "commitment must be lowercase hex");
}

// ---------------------------------------------------------------------------
// Test: PayloadFlip changes commitment and recomputes chain hash
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "mutation")]
fn e2e_payload_flip_changes_chain_hash() {
    use affidavit::mutation::{MutationKind, MutationOperator, PayloadFlipOperator};
    use affidavit::chain::deserialize_receipt;

    let dir = TempDir::new().unwrap();
    let receipt_path = write_receipt(&dir, 3);
    let bytes = std::fs::read(&receipt_path).unwrap();
    let receipt = deserialize_receipt(&bytes).unwrap();

    let original_hash = receipt.chain_hash.clone();
    let original_commitment = receipt.events[0].payload_commitment.clone();

    let op = PayloadFlipOperator;
    let applied = op.apply(&receipt, 0).unwrap();

    assert_eq!(applied.kind, MutationKind::PayloadFlip);
    assert_ne!(
        applied.mutated_receipt.events[0].payload_commitment,
        original_commitment,
        "commitment must change"
    );
    assert_ne!(
        applied.mutated_receipt.chain_hash,
        original_hash,
        "chain hash must change when commitment changes"
    );
}

// ---------------------------------------------------------------------------
// Test: determinism — same seed always produces same mutation
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "mutation")]
fn e2e_mutation_is_deterministic() {
    use affidavit::mutation::{MutationOperator, TypeChangeOperator};
    use affidavit::chain::deserialize_receipt;

    let dir = TempDir::new().unwrap();
    let receipt_path = write_receipt(&dir, 4);
    let bytes = std::fs::read(&receipt_path).unwrap();
    let receipt = deserialize_receipt(&bytes).unwrap();

    let op = TypeChangeOperator;
    let a = op.apply(&receipt, 7).unwrap();
    let b = op.apply(&receipt, 7).unwrap();

    assert_eq!(
        a.mutated_receipt.events[0].event_type,
        b.mutated_receipt.events[0].event_type,
        "same seed must produce same mutation"
    );
    assert_eq!(a.mutated_receipt.chain_hash, b.mutated_receipt.chain_hash);
}

// ---------------------------------------------------------------------------
// Test: missing receipt file exits non-zero with helpful error
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "mutation")]
fn e2e_mutate_missing_receipt_exits_nonzero() {
    Command::cargo_bin("affi")
        .unwrap()
        .args(["mutate", "receipt", "--count=1", "/nonexistent/path/receipt.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("receipt").or(predicate::str::contains("not found")));
}

// ---------------------------------------------------------------------------
// Test: --count=0 exits with error
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "mutation")]
fn e2e_mutate_count_zero_exits_nonzero() {
    let dir = TempDir::new().unwrap();
    let receipt_path = write_receipt(&dir, 3);

    Command::cargo_bin("affi")
        .unwrap()
        .args(["mutate", "receipt", "--count=0", receipt_path.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("count").or(predicate::str::contains("must be")));
}

// ---------------------------------------------------------------------------
// Test: integration — mutate then verify via CLI pipeline
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "mutation")]
fn e2e_cli_pipeline_mutate_then_verify() {
    let dir = TempDir::new().unwrap();
    let receipt_path = write_receipt(&dir, 4);

    // Step 1: mutate with --format=json, extract the first mutated receipt
    let output = Command::cargo_bin("affi")
        .unwrap()
        .args(["mutate", "receipt", "--count=1", "--format=json", receipt_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let arr: serde_json::Value = serde_json::from_str(stdout).unwrap();
    let first_mutation = &arr[0];

    // Step 2: extract the mutated receipt and write it to disk
    let mutated_receipt_json = first_mutation["mutated_receipt"].to_string();
    let mutated_path = dir.path().join("mutated.json");
    std::fs::write(&mutated_path, &mutated_receipt_json).unwrap();

    // Step 3: verify the mutated receipt — must ACCEPT (structural validity)
    Command::cargo_bin("affi")
        .unwrap()
        .args(["verify", mutated_path.to_str().unwrap()])
        .assert()
        .success(); // exit 0 = ACCEPT
}
```

---

## Tera Template: Test Code Generation

### Full Module Template

```
{# src/templates/test_module.tera #}
{# Context:
   fixture_set_name: string
   generated_at: ISO 8601 string
   tests: [TestContext]
   
   TestContext:
     pattern_name: string
     events: [EventContext]
     expected_verdict: "ACCEPT" | "REJECT"
     expected_failure_stage: string | null
   
   EventContext:
     event_type: string
     objects: [ObjectContext]
     payload: string (bytes literal content)
   
   ObjectContext:
     id: string
     obj_type: string
#}
//! Auto-generated receipt test module.
//!
//! Source fixture set: {{ fixture_set_name }}
//! Generated at: {{ generated_at }}
//! Generator: `affi generate test`
//!
//! DO NOT EDIT MANUALLY. Re-run `affi generate test` to regenerate.

#[cfg(test)]
mod generated_{{ fixture_set_name | replace(from="-", to="_") | replace(from=".", to="_") }} {
    use affidavit::chain::ChainAssembler;
    use affidavit::ocel::{build_event, object_ref, SeqCounter};
    use affidavit::verifier::verify;

    {% for test in tests %}
    /// Pattern: `{{ test.pattern_name }}`
    /// Expected verdict: {{ test.expected_verdict }}
    #[test]
    fn test_{{ test.pattern_name | replace(from="-", to="_") | replace(from=".", to="_") | replace(from="/", to="_") }}() {
        let mut asm = ChainAssembler::new();
        let mut counter = SeqCounter::new();

        {% for event in test.events %}
        let event_{{ loop.index0 }} = build_event(
            "{{ event.event_type }}",
            vec![
                {%- for obj in event.objects %}
                object_ref("{{ obj.id }}", "{{ obj.obj_type }}"){% if not loop.last %},{% endif %}
                {%- endfor %}
            ],
            b"{{ event.payload }}",
            &mut counter,
        ).expect("build event {{ loop.index0 }} for pattern {{ test.pattern_name }}");
        asm.append(event_{{ loop.index0 }})
            .expect("append event {{ loop.index0 }} for pattern {{ test.pattern_name }}");
        {% endfor %}

        let receipt = asm.finalize();
        let verdict = verify(&receipt);

        {% if test.expected_verdict == "ACCEPT" %}
        assert!(
            verdict.accepted,
            "pattern '{{ test.pattern_name }}' must ACCEPT; got: {}",
            verdict.reason
        );
        assert_eq!(verdict.outcomes.len(), 6, "verifier must run exactly 6 stages");
        assert!(
            verdict.outcomes.iter().all(|o| o.passed),
            "all stages must pass for ACCEPT verdict"
        );
        {% else %}
        assert!(
            !verdict.accepted,
            "pattern '{{ test.pattern_name }}' must REJECT; got: {}",
            verdict.reason
        );
        {% if test.expected_failure_stage %}
        let failing_stage = verdict.outcomes.iter().find(|o| !o.passed);
        assert!(
            failing_stage.is_some(),
            "at least one stage must have failed"
        );
        assert_eq!(
            failing_stage.unwrap().stage,
            "{{ test.expected_failure_stage }}",
            "expected first failure at stage '{{ test.expected_failure_stage }}'"
        );
        {% endif %}
        {% endif %}
    }
    {% endfor %}
}
```

### Fixture JSON Input Format for `affi generate test`

```json
{
  "fixture_set_name": "core-v1-patterns",
  "fixtures": [
    {
      "pattern_name": "basic-build-test",
      "expected_verdict": "ACCEPT",
      "expected_failure_stage": null,
      "events": [
        {
          "event_type": "build",
          "objects": [{"id": "repo:main", "obj_type": "git"}],
          "payload": "build-payload-0"
        },
        {
          "event_type": "test",
          "objects": [{"id": "suite:unit", "obj_type": "test-suite"}],
          "payload": "test-payload-1"
        }
      ]
    },
    {
      "pattern_name": "wrong-format-version",
      "expected_verdict": "REJECT",
      "expected_failure_stage": "check_format",
      "events": [
        {
          "event_type": "build",
          "objects": [{"id": "repo:main", "obj_type": "git"}],
          "payload": "build-payload-0"
        }
      ],
      "overrides": {
        "format_version": "wrong/v99"
      }
    }
  ]
}
```

---

## Kill Rate Computation Formula

### Definitions

Let:
- `M` = the full set of mutations applied (each `AppliedMutation` instance)
- `|M|` = total number of mutations in the sample
- `K` = the set of mutations where `verifier::verify(&m.mutated_receipt).accepted == false` (killed by verifier)
- `K_admit` = the set of mutations where `admission::admit(m.mutated_receipt.clone()).is_err()` (killed by admission)
- `S` = `M \ K` = surviving mutations (pass the verifier)
- `S_admit` = `M \ K_admit` = surviving the admission gate

### Structural Kill Rate (SKR)

Measures what fraction of mutations the 7-stage certify pipeline rejects.

```
SKR = |K| / |M|
```

**Expected value for affidavit:** `SKR ≈ 0.0` (all four operators produce structurally valid receipts that pass the verifier by design).

**Interpretation:** A non-zero SKR indicates an operator is incorrectly implemented (recomputed chain hash is wrong, or seq numbers are malformed). A zero SKR is the correct outcome — it proves the verifier does NOT over-reject.

### Semantic Survival Rate (SSR)

The complement: what fraction survive the certify pipeline (should be 100% for all operators).

```
SSR = |S| / |M| = 1 - SKR
```

**Expected value:** `SSR = 1.0` (100%).

### Admission Kill Rate (AKR)

Measures what fraction of mutations are rejected by the full admission gate (OCEL structural law + certify pipeline).

```
AKR = |K_admit| / |M|
```

**Expected values by operator:**

| Operator | Expected AKR | Rationale |
|----------|-------------|-----------|
| EventDrop | ≥0.90 | Dropping an event typically leaves dangling object links (OCEL court) or removes the only event (empty events list triggers OCEL `EmptyEventObjectLinks`). Not guaranteed for all drop targets in multi-event receipts where objects are shared across remaining events. |
| EventReorder | ~0.0 | Reordering preserves all events and their objects; no OCEL violation. |
| TypeChange | ~0.0 | Changing event_type does not affect object links or chain structure. |
| PayloadFlip | ~0.0 | Flipping commitment does not affect object links. |

### Kill Rate Test (Code)

```rust
/// Compute and assert the kill rate for a given operator over a sample of receipts and seeds.
///
/// # Arguments
/// - `operator`: the `MutationOperator` to evaluate
/// - `receipts`: sample of receipts (diverse sizes recommended)
/// - `seeds`: seed values to use per receipt
/// - `min_structural_kill_rate`: expected SKR (typically 0.0 for affidavit operators)
/// - `min_admission_kill_rate`: expected AKR (≥0.90 for EventDrop)
#[cfg(test)]
fn assert_kill_rate<Op: MutationOperator>(
    operator: &Op,
    receipts: &[Receipt],
    seeds: &[u64],
    min_structural_kill_rate: f64,
    max_structural_kill_rate: f64,
    min_admission_kill_rate: f64,
) {
    use affidavit::admission::admit;
    use affidavit::verifier::verify;

    let mut total = 0usize;
    let mut structural_kills = 0usize;
    let mut admission_kills = 0usize;

    for receipt in receipts {
        if receipt.events.len() < operator.min_events() {
            continue; // skip under-sized receipts for this operator
        }
        for &seed in seeds {
            let applied = operator.apply(receipt, seed).expect("apply mutation");
            total += 1;

            let verdict = verify(&applied.mutated_receipt);
            if !verdict.accepted {
                structural_kills += 1;
            }

            if admit(applied.mutated_receipt.clone()).is_err() {
                admission_kills += 1;
            }
        }
    }

    assert!(total > 0, "must have at least one sample");

    let skr = structural_kills as f64 / total as f64;
    let akr = admission_kills as f64 / total as f64;

    assert!(
        skr >= min_structural_kill_rate && skr <= max_structural_kill_rate,
        "Structural Kill Rate {:.2}% is outside [{:.2}%, {:.2}%] for operator {}",
        skr * 100.0, min_structural_kill_rate * 100.0, max_structural_kill_rate * 100.0,
        operator.name()
    );

    assert!(
        akr >= min_admission_kill_rate,
        "Admission Kill Rate {:.2}% < {:.2}% threshold for operator {}",
        akr * 100.0, min_admission_kill_rate * 100.0, operator.name()
    );
}
```

### Kill Rate Test Invocations

```rust
#[test]
#[cfg(feature = "mutation")]
fn kill_rate_event_drop() {
    // EventDrop: structural kill rate must be 0% (valid outputs);
    // admission kill rate ≥90% (OCEL court catches empty/dangling links).
    let receipts = build_sample_receipts(100); // 100 receipts of sizes 1–20
    let seeds: Vec<u64> = (0..10).collect();
    assert_kill_rate(&EventDropOperator, &receipts, &seeds, 0.0, 0.0, 0.90);
}

#[test]
#[cfg(feature = "mutation")]
fn kill_rate_event_reorder() {
    let receipts = build_sample_receipts(100);
    let seeds: Vec<u64> = (0..10).collect();
    // Both structural and admission kill rates must be 0% for EventReorder.
    assert_kill_rate(&EventReorderOperator, &receipts, &seeds, 0.0, 0.0, 0.0);
}

#[test]
#[cfg(feature = "mutation")]
fn kill_rate_type_change() {
    let receipts = build_sample_receipts(100);
    let seeds: Vec<u64> = (0..10).collect();
    assert_kill_rate(&TypeChangeOperator, &receipts, &seeds, 0.0, 0.0, 0.0);
}

#[test]
#[cfg(feature = "mutation")]
fn kill_rate_payload_flip() {
    let receipts = build_sample_receipts(100);
    let seeds: Vec<u64> = (0..10).collect();
    assert_kill_rate(&PayloadFlipOperator, &receipts, &seeds, 0.0, 0.0, 0.0);
}
```

---

## Performance Baseline

All performance assertions must pass on a standard CI runner (4-core, 8 GB RAM, no SSD requirement).

| Operation | Requirement | Measurement Method |
|-----------|------------|-------------------|
| Apply 10 mutations (any operator) | <1s wall time | `std::time::Instant` in `tests/perf_mutation.rs` |
| Apply 100 mutations (any operator) | <5s wall time | Same |
| FixtureDatabase search (1000 fixtures) | <10ms | `std::time::Instant` in `tests/fixture_db_perf.rs` |
| FixtureDatabase insert (1000 fixtures) | <500ms total | Same |
| `affi generate test` (50 fixtures) | <2s wall time | E2E timing in `tests/e2e_generate.rs` |
| `affi generate snippet --list` (100 snippets) | <200ms wall time | E2E timing in `tests/e2e_snippet.rs` |

### Performance Test File

```rust
// tests/perf_mutation.rs
#[test]
#[cfg(feature = "mutation")]
fn perf_10_mutations_under_1s() {
    use affidavit::mutation::{EventDropOperator, MutationOperator};
    use std::time::Instant;

    let receipt = build_sample_receipt(10);
    let op = EventDropOperator;
    let start = Instant::now();
    for seed in 0..10u64 {
        let _ = op.apply(&receipt, seed).unwrap();
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs_f64() < 1.0,
        "10 EventDrop mutations took {:.2}s (budget: 1.0s)",
        elapsed.as_secs_f64()
    );
}
```

---

## Phase 3 Exit Gate Checklist

The following checklist must be fully checked before Phase 3 is marked **DONE** on the `claude/zen-cerf-oq87br` branch and before any merge to main.

### Feature Completeness

- [ ] **F1:** `affi mutate receipt --count=N` implemented and all 10 ACs pass
- [ ] **F2:** `affi generate test` implemented and all 10 ACs pass
- [ ] **F3:** `affi generate snippet --pattern=<name>` implemented and all 10 ACs pass
- [ ] **F4:** `tests/property_based.rs` exists with all properties PROP-A through PROP-D passing
- [ ] **F5:** `src/fixture_db.rs` implemented and all 10 ACs pass

### New Modules

- [ ] `src/mutation.rs` exists with `MutationOperator` trait and 4 operator impls
- [ ] `src/fixture_db.rs` exists with `FixtureDatabase`, `Fixture`, `FixtureQuery`
- [ ] `tests/property_based.rs` exists with `ArbitraryReceipt`, `Arbitrary for OperationEvent`, all property fns
- [ ] `tests/e2e_mutation.rs` exists with all test cases from the template above
- [ ] `src/templates/test_fn.tera` and `src/templates/test_module.tera` exist
- [ ] `src/snippets/registry.json` exists with at least 10 snippets

### Build & Test

- [ ] `cargo build` (no features) — zero errors, zero warnings
- [ ] `cargo build --features mutation` — zero errors, zero warnings
- [ ] `cargo build --features fixture-db` — zero errors, zero warnings
- [ ] `cargo build --features mutation,fixture-db` — zero errors, zero warnings
- [ ] `cargo test` — all existing 30 tests still pass (no regression)
- [ ] `cargo test --features mutation` — all mutation tests pass
- [ ] `cargo test --features fixture-db` — all fixture DB tests pass
- [ ] `cargo test --features mutation,fixture-db` — full combined test suite passes
- [ ] `cargo test --test property_based` — all quickcheck properties pass
- [ ] `cargo test --test e2e_mutation --features mutation` — all E2E mutation tests pass
- [ ] `QUICKCHECK_TESTS=500 cargo test --test property_based` — no flakes

### Code Quality

- [ ] `cargo clippy --features mutation,fixture-db -- -D warnings` — clean
- [ ] `cargo fmt --check` — all new files formatted
- [ ] `cargo doc --no-deps --features mutation,fixture-db` — zero warnings
- [ ] No `unwrap()` in non-test paths of `src/mutation.rs` or `src/fixture_db.rs`
- [ ] No unsafe code in any new module

### Kill Rate

- [ ] `kill_rate_event_drop` test passes (SKR = 0%, AKR ≥ 90%)
- [ ] `kill_rate_event_reorder` test passes (SKR = 0%, AKR ≥ 0%)
- [ ] `kill_rate_type_change` test passes (SKR = 0%, AKR ≥ 0%)
- [ ] `kill_rate_payload_flip` test passes (SKR = 0%, AKR ≥ 0%)

### Performance

- [ ] `perf_10_mutations_under_1s` — passes on CI
- [ ] `perf_fixture_db_search_1000_under_10ms` — passes on CI
- [ ] `perf_generate_test_50_fixtures_under_2s` — passes on CI

### Documentation

- [ ] Every public type and function in `src/mutation.rs` has `///` doc
- [ ] Every public type and function in `src/fixture_db.rs` has `///` doc
- [ ] `tests/property_based.rs` module doc comment lists all invariant groups
- [ ] `examples/mutation_demo.rs` exists and runs with `cargo run --example mutation_demo --features mutation`

### Integration Smoke Test

- [ ] The following shell pipeline completes with exit 0:
  ```bash
  cargo build --features mutation,fixture-db
  # Emit → assemble → mutate → verify pipeline
  ./target/debug/affi emit --type build --object repo:main:git
  ./target/debug/affi assemble --out /tmp/test.json
  ./target/debug/affi mutate receipt --count=4 --format=json /tmp/test.json
  ./target/debug/affi verify /tmp/test.json
  ```

---

*End of Definition of Done — Phase 3: Test Generation & Mutation Testing*  
*Branch: `claude/zen-cerf-oq87br` | Version: 26.6.14 | Date: 2026-06-14*
