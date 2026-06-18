# Definition of Done — Phase 2: Process Discovery & IDE Integration

**Project:** affidavit — Provenance Layer  
**Version:** 26.6.17  
**Branch:** `claude/zen-cerf-oq87br`  
**Phase:** 2 of the DX/QOL 1000x initiative  
**Author:** Sean Chatman (xpointsh@gmail.com)  
**Date:** 2026-06-14  

---

## Overview

Phase 2 extends `affidavit` from a pure certification tool into a provenance-aware IDE citizen. It adds five new capabilities:

| # | Feature | CLI | Feature Gate | New Module(s) |
|---|---------|-----|-------------|---------------|
| 1 | Process model discovery | `affi receipt model` | `discovery` | `src/mining.rs` |
| 2 | Conformance fitness scoring | `affi receipt conform` | `conformance` | (extends `src/discovery.rs`) |
| 3 | Next-activity prediction | `affi receipt predict` | `predictive` | (extends `src/mining.rs`) |
| 4 | LSP Hover | n/a (editor) | `lsp` | `src/lsp/hover.rs` |
| 5 | LSP Goto-Definition | n/a (editor) | `lsp` | `src/lsp/goto_definition.rs` |

All five share a common constraint inherited from Phase 1: **certify, don't decide.** The mining and conformance surfaces reveal structure in a receipt; they do not adjudicate honesty. The LSP surfaces report; they do not fix.

The existing `src/lsp.rs` (verdict → diagnostics) is **not replaced**; it becomes `src/lsp/diagnostics.rs` and the new hover and goto-definition modules join it under `src/lsp/mod.rs`.

---

## Preconditions (must be true before Phase 2 work begins)

- [ ] All 30 Phase 1 tests pass on `main` (`cargo test` exits 0).
- [ ] `cargo build` produces `affi` binary with no warnings under `RUSTFLAGS="-D warnings"`.
- [ ] Dev branch `claude/zen-cerf-oq87br` is cut from `main` with a clean `git status`.
- [ ] `wasm4pm` dependency path `../wasm4pm/wasm4pm` resolves and exposes: `heuristic_inductive_miner`, `alignment_fitness`, `next_activity` (verify with `cargo doc --open`).
- [ ] `lsp-max` dependency path `../lsp-max` resolves and exposes `HoverResponse`, `GotoDefinitionResponse`, `Location`, `Range`, `Position` (verify with `cargo doc --open`).

---

## Shared Conventions for All Phase 2 Code

### Module Hierarchy After Phase 2

```
src/
├── mining.rs                   # receipt_to_ocel(), discover_him(), predict_next()
├── lsp/
│   ├── mod.rs                  # re-exports; pub use diagnostics::*, hover::*, goto_definition::*
│   ├── diagnostics.rs          # moved from src/lsp.rs — verdict_to_diagnostics()
│   ├── hover.rs                # hover_for_event_id()
│   └── goto_definition.rs      # goto_definition_for_event_type()
└── verbs/
    ├── conform.rs              # affi receipt conform --model=expected.json
    └── predict.rs              # affi receipt predict
```

### Feature Gate Discipline

Feature gates are **additive**: the binary must compile and all Phase 1 tests must still pass with `cargo test` (no features). Phase 2 functionality activates only under its gate.

```toml
# Cargo.toml additions
[features]
default = []
otel       = []                  # Phase 1 (unchanged)
discovery  = ["wasm4pm/him"]     # Feature 1: HIM model discovery
conformance = ["wasm4pm/align"]  # Feature 2: alignment fitness
predictive  = ["wasm4pm/predict"]# Feature 3: next-activity prediction
lsp         = ["lsp-max/hover", "lsp-max/goto"] # Features 4+5
```

### Error Type Conventions

All new `Result` types use `thiserror`-derived enums with named variants. No `anyhow::anyhow!("string")` in library code (only in `src/cli.rs` and `src/handlers.rs`). No `.unwrap()` outside of tests.

### No `println!` Policy

The `#![deny(clippy::print_stdout)]` lint in `src/lib.rs` applies to all new library modules. CLI output routes through `eprintln!` (handler layer) or structured return values.

### Determinism

New mining and prediction surfaces must be **deterministic**: same receipt → same model, same scores, same prediction ranking. No wall-clock, no random seeds, no thread-local state.

---

## Feature 1: `affi receipt model` — HIM Process Discovery

### Description

`affi receipt model <RECEIPT>` reads a sealed receipt, converts it to an OCEL event log, runs wasm4pm's Heuristic Inductive Miner (HIM), and emits a Petri net (places, transitions, arcs) as JSON to stdout. The receipt is type-gated on admission: only an `AdmittedReceipt` may be mined (the same discipline as the existing `discover_from_admitted`).

Feature gate: `--features discovery`

---

### 1.1 Acceptance Criteria

**AC-1.1** Given a 3-event receipt with activities `["create", "transform", "release"]`, when `affi receipt model receipt.json` is run, then stdout contains valid JSON with keys `places`, `transitions`, `arcs` and stderr reports the activity count.

**AC-1.2** Given the same receipt, when the model is run twice in separate processes, then both runs produce byte-identical JSON (determinism guarantee: no timestamps, no random seeds).

**AC-1.3** Given a receipt with 100 events (mixed activities), when `affi receipt model receipt.json` is run, then it completes in under 5 seconds on developer hardware (MacBook M2 or equivalent x86-64 workstation with 8 GB RAM) and exit code is 0.

**AC-1.4** Given a receipt that fails the OCEL court (objectless event), when `affi receipt model` is run, then exit code is non-zero and stderr contains `admission refused` with the named refusal variant (`EmptyEventObjectLinks`).

**AC-1.5** Given a receipt that fails chain integrity (tampered event), when `affi receipt model` is run, then exit code is non-zero and stderr contains `chain hash mismatch`.

**AC-1.6** Given a receipt with a single-activity trace (10 events, all `event_type = "deploy"`), when `affi receipt model` is run, then the output Petri net contains exactly one transition whose label is `"deploy"`.

**AC-1.7** Given the feature is compiled **without** `--features discovery`, when `affi receipt model receipt.json` is run, then the command exits non-zero and stderr contains `feature 'discovery' required` (not a panic, not a silent no-op).

**AC-1.8** Given a receipt file that does not exist, when `affi receipt model missing.json` is run, then exit code is non-zero and stderr contains the file path and a human-readable I/O error message.

**AC-1.9** Given a receipt with N events where each event has a unique `event_type`, when `affi receipt model` is run, then the discovered Petri net has at least N distinct transitions (one per unique activity).

**AC-1.10** Given `--format json` flag, when `affi receipt model receipt.json --format json` is run, then stdout is valid JSON parseable by `serde_json::from_str::<serde_json::Value>`.

---

### 1.2 Implementation Checklist

**`src/mining.rs` — new module**

- [ ] Define `OcelLog` conversion: `pub fn receipt_to_ocel(receipt: &Receipt) -> OcelEventLog`  
  (distinct from `discovery::project_to_event_log` which returns `wasm4pm::EventLog`; this returns the OCEL2-standard struct for HIM input)
- [ ] Implement `#[cfg(feature = "discovery")] pub fn discover_him(receipt: &AdmittedReceipt) -> Result<PetriNet, MiningError>`
- [ ] Define `pub struct PetriNet` with fields: `places: Vec<Place>`, `transitions: Vec<Transition>`, `arcs: Vec<Arc>`
- [ ] Define `pub struct Place { pub id: String, pub label: Option<String> }`
- [ ] Define `pub struct Transition { pub id: String, pub label: String, pub is_silent: bool }`
- [ ] Define `pub struct Arc { pub from: String, pub to: String, pub weight: u32 }`
- [ ] Derive `Serialize, Deserialize, Debug, Clone, PartialEq` on all structs
- [ ] Define `#[derive(Debug, thiserror::Error)] pub enum MiningError` with variants:
  - `#[error("wasm4pm HIM failed: {0}")] Him(String)`
  - `#[error("admission refused: {0}")] Admission(String)`
  - `#[error("OCEL conversion failed: {0}")] OcelConversion(String)`
- [ ] Implement `#[cfg(not(feature = "discovery"))] pub fn discover_him(...) -> Result<PetriNet, MiningError>` that returns `Err(MiningError::Him("feature 'discovery' required".into()))`

**`src/verbs/model.rs` — update existing**

- [ ] Change handler call from `crate::handlers::model(receipt)` to route through `mining::discover_him` when feature enabled
- [ ] Serialize `PetriNet` to stdout as JSON
- [ ] Print summary to stderr: `"discovered Petri net: {P} places, {T} transitions, {A} arcs"`
- [ ] Preserve existing admission type-gate: call `admission::admit()` before mining

**`src/handlers.rs` — update `model` handler**

- [ ] Update `pub fn model(receipt: String) -> Result<()>` to call `mining::discover_him` under feature gate
- [ ] Add `#[cfg(feature = "discovery")]` path producing JSON stdout
- [ ] Add `#[cfg(not(feature = "discovery"))]` path producing structured error

**`src/lib.rs` — module registration**

- [ ] Add `pub mod mining;`
- [ ] Re-export `mining::PetriNet` from the crate root

**`Cargo.toml` — feature gates**

- [ ] Add `[features]` entries: `discovery = ["wasm4pm/him"]`
- [ ] Verify `wasm4pm/him` feature name matches actual dependency feature

---

### 1.3 Feature Gate Testing

```bash
# With feature: model runs and produces Petri net JSON
cargo test --features discovery -- model
cargo run --features discovery --bin affi -- receipt model tests/fixtures/honest.json

# Without feature: compile still succeeds, model returns structured error
cargo test -- model_feature_disabled
cargo build  # no features — must succeed
```

- [ ] Test `model_with_discovery_feature_produces_petri_net_json` (feature enabled)
- [ ] Test `model_without_discovery_feature_returns_structured_error` (feature disabled)

---

### 1.4 API Contracts

```rust
// src/mining.rs

/// Convert a sealed Receipt into an OCEL2-standard event log for HIM input.
/// Each OperationEvent maps to one OCEL event; event_type → activity label.
/// Objects map to OCEL objects; qualifiers map to OCEL relationship qualifiers.
pub fn receipt_to_ocel(receipt: &Receipt) -> OcelEventLog;

/// Discover a Heuristic Inductive Miner Petri net from an admitted receipt.
/// Gate: only callable with `--features discovery`.
/// The receipt must have passed both admission courts (OCEL + chain).
#[cfg(feature = "discovery")]
pub fn discover_him(receipt: &AdmittedReceipt) -> Result<PetriNet, MiningError>;

/// Deterministic check: same receipt → identical PetriNet byte output.
/// Guaranteed: no wall-clock, no random seed in wasm4pm HIM path.
#[cfg(feature = "discovery")]
pub fn discover_him_deterministic(receipt: &AdmittedReceipt) -> Result<PetriNet, MiningError> {
    discover_him(receipt)  // determinism is a property of the function, not a wrapper
}

pub struct PetriNet {
    pub places: Vec<Place>,
    pub transitions: Vec<Transition>,
    pub arcs: Vec<Arc>,
}

pub struct Place {
    pub id: String,
    pub label: Option<String>,
}

pub struct Transition {
    pub id: String,
    pub label: String,
    pub is_silent: bool,
}

pub struct Arc {
    pub from: String,  // place.id or transition.id
    pub to: String,    // place.id or transition.id
    pub weight: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum MiningError {
    #[error("wasm4pm HIM failed: {0}")]
    Him(String),
    #[error("admission refused: {0}")]
    Admission(String),
    #[error("OCEL conversion failed: {0}")]
    OcelConversion(String),
}
```

---

### 1.5 Petri Net JSON Schema

The canonical JSON output of `affi receipt model`:

```json
{
  "$schema": "https://affidavit.rs/schemas/petri-net/v1.json",
  "schema_version": "petri-net/v1",
  "source_receipt_hash": "<blake3-hex-64-chars>",
  "miner": "heuristic-inductive-miner",
  "places": [
    { "id": "p0", "label": null },
    { "id": "p1", "label": "after_create" },
    { "id": "p_sink", "label": "end" }
  ],
  "transitions": [
    { "id": "t_create",    "label": "create",    "is_silent": false },
    { "id": "t_transform", "label": "transform", "is_silent": false },
    { "id": "t_release",   "label": "release",   "is_silent": false },
    { "id": "t_tau0",      "label": "τ",         "is_silent": true  }
  ],
  "arcs": [
    { "from": "p0",         "to": "t_create",    "weight": 1 },
    { "from": "t_create",   "to": "p1",          "weight": 1 },
    { "from": "p1",         "to": "t_transform", "weight": 1 },
    { "from": "t_transform","to": "p_sink",       "weight": 1 },
    { "from": "p_sink",     "to": "t_release",   "weight": 1 },
    { "from": "t_release",  "to": "p0",          "weight": 1 }
  ]
}
```

**Invariants the verifier will check:**
- Every `arc.from` and `arc.to` references an existing `place.id` or `transition.id`.
- Places and transitions are disjoint ID spaces (no `id` appears in both).
- The net has exactly one source place (no incoming arcs) and one sink place (no outgoing arcs) for a sound workflow net.
- Silent transitions (`is_silent: true`) use label `"τ"`.

---

### 1.6 OCEL Conversion Schema

Receipt event → OCEL event mapping:

| Receipt field | OCEL field | Notes |
|--------------|-----------|-------|
| `event.id` | `ocel:eid` | Direct copy; must be globally unique within the log |
| `event.event_type` | `ocel:activity` | The activity label used by the miner |
| `event.seq` | `ocel:timestamp` (synthetic) | Encoded as `seq * 1_000_000_000` nanoseconds from epoch 0; deterministic, never wall-clock |
| `event.objects[*].id` | `ocel:oid` | Each unique object becomes one OCEL object |
| `event.objects[*].obj_type` | `ocel:type` | OCEL object type |
| `event.objects[*].qualifier` | `ocel:qualifier` | Optional; omitted when `None` |
| `event.payload_commitment` | custom attr `"affidavit:commitment"` | Carried as opaque string attribute for auditability |

Full OCEL2 JSON example for a 2-event receipt:

```json
{
  "ocel:version": "2.0",
  "ocel:ordering": "timestamp",
  "ocel:attribute-names": ["affidavit:commitment"],
  "ocel:object-types": [
    { "name": "artifact", "attributes": [] },
    { "name": "agent",    "attributes": [] }
  ],
  "ocel:events": [
    {
      "ocel:eid":       "evt-0",
      "ocel:activity":  "create",
      "ocel:timestamp": "1970-01-01T00:00:00.000000000Z",
      "ocel:vmap": {
        "affidavit:commitment": "6ef47c82a1d3..."
      },
      "ocel:typedOmap": [
        { "ocel:oid": "file-1", "ocel:type": "artifact", "ocel:qualifier": "output" }
      ]
    },
    {
      "ocel:eid":       "evt-1",
      "ocel:activity":  "transform",
      "ocel:timestamp": "1970-01-01T00:00:01.000000000Z",
      "ocel:vmap": {
        "affidavit:commitment": "a2d95f11b4e2..."
      },
      "ocel:typedOmap": [
        { "ocel:oid": "file-1",  "ocel:type": "artifact", "ocel:qualifier": null },
        { "ocel:oid": "agent-1", "ocel:type": "agent",    "ocel:qualifier": "actor" }
      ]
    }
  ],
  "ocel:objects": [
    { "ocel:oid": "file-1",  "ocel:type": "artifact", "ocel:ovmap": {} },
    { "ocel:oid": "agent-1", "ocel:type": "agent",    "ocel:ovmap": {} }
  ]
}
```

---

### 1.7 Test Evidence

**Required test names and file locations:**

| Test name | File | Assertion |
|-----------|------|-----------|
| `receipt_to_ocel_maps_all_events` | `src/mining.rs` | `ocel.events.len() == receipt.events.len()` |
| `receipt_to_ocel_maps_event_type_to_activity` | `src/mining.rs` | `ocel.events[0].activity == receipt.events[0].event_type` |
| `receipt_to_ocel_seq_becomes_synthetic_timestamp` | `src/mining.rs` | timestamp of event N equals `N * 1_000_000_000` ns |
| `receipt_to_ocel_deduplicates_objects` | `src/mining.rs` | one OCEL object per unique `object.id` |
| `receipt_to_ocel_carries_commitment_as_attribute` | `src/mining.rs` | `ocel.events[0].vmap["affidavit:commitment"] == event.payload_commitment.as_hex()` |
| `discover_him_names_receipt_activities` | `src/mining.rs` (feature gate) | model contains every unique `event_type` as transition label |
| `discover_him_is_deterministic` | `src/mining.rs` (feature gate) | `discover_him(&r) == discover_him(&r)` |
| `discover_him_single_activity_yields_one_transition` | `src/mining.rs` (feature gate) | `net.transitions.len() == 1` for single-activity receipt |
| `model_feature_disabled_returns_structured_error` | `src/mining.rs` | `MiningError::Him` variant when feature not compiled |
| `e2e_model_produces_valid_petri_net_json` | `tests/e2e_discovery.rs` | stdout parses as `PetriNet`, contains `"places"` key |
| `e2e_model_missing_receipt_exits_nonzero` | `tests/e2e_discovery.rs` | exit code non-zero, stderr contains path |
| `e2e_model_objectless_receipt_refused` | `tests/e2e_discovery.rs` | exit non-zero, stderr contains `EmptyEventObjectLinks` |
| `e2e_model_100_events_under_5s` | `tests/e2e_discovery.rs` | `Duration < 5s` |

---

### 1.8 Error Handling

| Scenario | Behavior |
|----------|----------|
| `wasm4pm` HIM returns internal error | Map to `MiningError::Him(e.to_string())`; exit non-zero; stderr: `"model discovery failed: {detail}"` |
| Receipt fails OCEL court | Map to `MiningError::Admission(refusal.to_string())`; exit non-zero; stderr names the refusal variant |
| Receipt fails chain integrity | Deserialization fails before mining; stderr: `"chain hash mismatch"`; exit non-zero |
| Receipt file not found | `ChainError::Io` surfaced before mining; stderr: `"io error at {path}: {os_error}"` |
| `feature = "discovery"` not compiled | Returns `Err(MiningError::Him("feature 'discovery' required"))` from stub; CLI exits non-zero with that message |
| Receipt with 0 events | `receipt_to_ocel` produces empty OCEL log; HIM returns trivially empty net or error — both are acceptable; must not panic |

---

### 1.9 Performance Budget

| Operation | Budget | Measurement |
|-----------|--------|-------------|
| `receipt_to_ocel` conversion (100 events) | < 1 ms | `std::time::Instant` in benchmark |
| `discover_him` (100 events, 10 distinct activities) | < 5 s | wall-clock in `e2e_model_100_events_under_5s` |
| `discover_him` (10 events) | < 500 ms | wall-clock in unit test timing assertion |
| Petri net JSON serialization | < 5 ms | negligible; include in end-to-end budget |

The 5-second budget for 100 events is a **hard gate**: the E2E test must assert `elapsed < Duration::from_secs(5)`.

---

### 1.10 Documentation

- [ ] Module-level doc comment in `src/mining.rs` explaining the receipt-to-OCEL projection and the admission type-gate
- [ ] `//! # Example` pointing to `examples/ocel_events.rs` updated with HIM usage
- [ ] New example `examples/model_discovery.rs` demonstrating: build receipt → admit → discover_him → print net
- [ ] `CLAUDE.md` section "Integration Points" updated: add "Process Discovery (HIM)" subsection
- [ ] `README.md` CLI Surface table updated with `affi receipt model` entry
- [ ] `Cargo.toml` `[features]` table documented inline with a comment per feature

---

## Feature 2: `affi receipt conform --model=expected.json` — Fitness Scoring

### Description

`affi receipt conform <RECEIPT> --model <expected.json>` computes an alignment fitness score (0.0–1.0) between the receipt's event trace and a reference Petri net model. Uses `wasm4pm::alignment_fitness()`. The receipt is type-gated on admission. Output is a structured JSON blob containing `fitness`, `precision`, `generalization`, `simplicity`, and an `interpretation` field.

Feature gate: `--features conformance`

---

### 2.1 Acceptance Criteria

**AC-2.1** Given a 3-event receipt with trace `[create, transform, release]` and a reference model that exactly describes this trace, when `affi receipt conform receipt.json --model expected.json` is run, then the fitness score is ≥ 0.95 and the `interpretation` field is `"perfect-fit"`.

**AC-2.2** Given the same receipt and a completely mismatched model (e.g., only `[deploy, rollback]` transitions), when `conform` is run, then fitness is ≤ 0.2 and `interpretation` is `"poor-fit"`.

**AC-2.3** Given a valid receipt and an existing reference model, when `conform` is run twice in separate processes, then both runs produce the same JSON output (determinism).

**AC-2.4** Given `--model` points to a nonexistent file, when `conform` is run, then exit code is non-zero and stderr contains the missing file path.

**AC-2.5** Given a reference model JSON with invalid structure (missing `transitions` field), when `conform` is run, then exit code is non-zero and stderr contains `"model parse error"`.

**AC-2.6** Given a receipt that fails admission (tampered chain), when `conform` is run, then exit code is non-zero and stderr contains `"admission refused"`.

**AC-2.7** Given `--features conformance` not compiled, when `affi receipt conform` is run, then exit code is non-zero and stderr contains `"feature 'conformance' required"`.

**AC-2.8** Given a receipt with a 100-event trace, when `conform` is run, then it completes in under 10 seconds.

**AC-2.9** Given a fitness score of 0.85, when stdout is examined, then `interpretation` is `"acceptable-fit"`.

**AC-2.10** Given the `--format json` flag, when `conform` is run, then stdout is valid JSON parseable with `serde_json::from_str::<ConformanceReport>`.

---

### 2.2 Implementation Checklist

**`src/mining.rs` — extend with conformance**

- [ ] Define `pub struct ConformanceReport` (see API Contracts below)
- [ ] Implement `#[cfg(feature = "conformance")] pub fn alignment_fitness_score(receipt: &AdmittedReceipt, model: &PetriNet) -> Result<ConformanceReport, ConformanceError>`
- [ ] Implement `pub fn interpret_score(fitness: f64) -> FitnessInterpretation`
- [ ] Define `pub enum FitnessInterpretation` with variants: `PerfectFit`, `GoodFit`, `AcceptableFit`, `PoorFit`, `NoFit`
- [ ] Implement `Display` on `FitnessInterpretation` returning the string used in JSON
- [ ] Define `#[derive(Debug, thiserror::Error)] pub enum ConformanceError` with variants: `Wasm4pm(String)`, `ModelParse(String)`, `Admission(String)`
- [ ] Implement stub `#[cfg(not(feature = "conformance"))]` returning `Err(ConformanceError::Wasm4pm("feature 'conformance' required".into()))`
- [ ] Implement `pub fn load_petri_net(path: &Path) -> Result<PetriNet, ConformanceError>` (parses JSON from file)

**`src/verbs/conform.rs` — new file**

- [ ] Create `src/verbs/conform.rs` with `#[verb("conform", "receipt")]` attribute
- [ ] Add `--model <MODEL>` argument (required `String`)
- [ ] Delegate to `crate::handlers::conform(receipt, model_path)`

**`src/verbs/mod.rs` — register new verb**

- [ ] Add `pub mod conform;`

**`src/handlers.rs` — add `conform` handler**

- [ ] Implement `pub fn conform(receipt: String, model: String) -> Result<()>`
- [ ] Under `#[cfg(feature = "conformance")]`: load receipt → admit → load PetriNet → alignment_fitness_score → serialize report → print to stdout
- [ ] Under `#[cfg(not(feature = "conformance"))]`: return structured error

**`src/cli.rs` — not changed** (handlers layer absorbs new functionality)

---

### 2.3 Feature Gate Testing

```bash
# With feature
cargo test --features conformance -- conform
cargo run --features conformance --bin affi -- receipt conform tests/fixtures/honest.json \
  --model tests/fixtures/expected_model.json

# Without feature
cargo test -- conform_feature_disabled
cargo build  # must succeed
```

- [ ] Test `conform_with_feature_computes_score_in_range` (feature enabled)
- [ ] Test `conform_feature_disabled_returns_structured_error` (feature disabled)
- [ ] Test `interpret_score_thresholds_are_correct` (always compiled, tests `interpret_score`)

---

### 2.4 API Contracts

```rust
// src/mining.rs

/// The structured report returned by alignment fitness scoring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConformanceReport {
    /// Alignment-based fitness: fraction of trace moves that are synchronous.
    /// Range: [0.0, 1.0]. Source: wasm4pm::alignment_fitness().
    pub fitness: f64,
    /// Activity coverage ratio (NOT van der Aalst escaping-edges precision).
    /// |log_activities ∩ model_activities| / |model_activities|.
    pub activity_coverage: f64,
    /// Simplicity score from Occam dimension: 1 / (1 + arcs/transitions).
    pub simplicity: f64,
    /// Human-readable interpretation of the fitness score.
    pub interpretation: String,
    /// The number of trace events that could not be replayed (move-on-log).
    pub unfit_events: u32,
    /// The number of model moves needed to complete replay (move-on-model).
    pub model_moves: u32,
}

/// Compute alignment fitness between an admitted receipt's trace and a reference model.
#[cfg(feature = "conformance")]
pub fn alignment_fitness_score(
    receipt: &AdmittedReceipt,
    model: &PetriNet,
) -> Result<ConformanceReport, ConformanceError>;

/// Interpret a raw fitness score as a named category.
pub fn interpret_score(fitness: f64) -> FitnessInterpretation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitnessInterpretation {
    PerfectFit,     // 0.95 – 1.00
    GoodFit,        // 0.80 – <0.95
    AcceptableFit,  // 0.60 – <0.80
    PoorFit,        // 0.30 – <0.60
    NoFit,          // 0.00 – <0.30
}

/// Load a PetriNet from a JSON file at path.
pub fn load_petri_net(path: &std::path::Path) -> Result<PetriNet, ConformanceError>;
```

---

### 2.5 Conformance Score Interpretation Guide

| Score Range | `FitnessInterpretation` | JSON value | Meaning |
|-------------|------------------------|-----------|---------|
| 0.95 – 1.00 | `PerfectFit` | `"perfect-fit"` | Every trace move is synchronous with the model; model exactly describes the observed process |
| 0.80 – <0.95 | `GoodFit` | `"good-fit"` | Minor deviations; the model largely describes the process; acceptable for most provenance audits |
| 0.60 – <0.80 | `AcceptableFit` | `"acceptable-fit"` | Moderate deviations; the model partially describes the process; worth investigating outlier events |
| 0.30 – <0.60 | `PoorFit` | `"poor-fit"` | Significant deviations; the model describes a substantially different process; likely wrong model or wrong receipt |
| 0.00 – <0.30 | `NoFit` | `"no-fit"` | Near-total mismatch; the model does not describe the observed process; do not use for certification |

**Thresholds are inclusive at the lower bound, exclusive at the upper bound** (except `PerfectFit` which is inclusive at both ends). The exact threshold values (0.95, 0.80, 0.60, 0.30) are part of the public API contract and must not be changed without a CHANGELOG entry and a version bump.

**Example JSON output:**

```json
{
  "fitness": 0.923,
  "activity_coverage": 0.875,
  "simplicity": 0.667,
  "interpretation": "good-fit",
  "unfit_events": 1,
  "model_moves": 2
}
```

---

### 2.6 Test Evidence

| Test name | File | Assertion |
|-----------|------|-----------|
| `interpret_score_perfect_fit_at_095` | `src/mining.rs` | `interpret_score(0.95) == PerfectFit` |
| `interpret_score_perfect_fit_at_100` | `src/mining.rs` | `interpret_score(1.0) == PerfectFit` |
| `interpret_score_good_fit_at_080` | `src/mining.rs` | `interpret_score(0.80) == GoodFit` |
| `interpret_score_acceptable_at_060` | `src/mining.rs` | `interpret_score(0.60) == AcceptableFit` |
| `interpret_score_poor_at_030` | `src/mining.rs` | `interpret_score(0.30) == PoorFit` |
| `interpret_score_no_fit_at_zero` | `src/mining.rs` | `interpret_score(0.0) == NoFit` |
| `conform_score_in_range` | `src/mining.rs` (feature gate) | `0.0 <= report.fitness <= 1.0` |
| `conform_self_model_high_fitness` | `src/mining.rs` (feature gate) | receipt conformed against its own discovered model has fitness ≥ 0.8 |
| `conform_feature_disabled_returns_error` | `src/mining.rs` | `Err(ConformanceError::Wasm4pm("feature 'conformance' required"))` |
| `load_petri_net_valid_file` | `src/mining.rs` | parses fixture JSON into `PetriNet` |
| `load_petri_net_missing_file_errors` | `src/mining.rs` | `Err(ConformanceError::ModelParse(...))` |
| `e2e_conform_self_model_passes` | `tests/e2e_discovery.rs` | exit 0, stdout contains `"fitness"` |
| `e2e_conform_missing_model_exits_nonzero` | `tests/e2e_discovery.rs` | exit non-zero, stderr contains path |
| `e2e_conform_100_events_under_10s` | `tests/e2e_discovery.rs` | `elapsed < Duration::from_secs(10)` |

---

### 2.7 Error Handling

| Scenario | Behavior |
|----------|----------|
| `wasm4pm::alignment_fitness()` fails | `ConformanceError::Wasm4pm(detail)`; exit non-zero; stderr: `"alignment fitness failed: {detail}"` |
| `--model` file not found | `ConformanceError::ModelParse("no such file: {path}")`; exit non-zero |
| `--model` JSON malformed | `ConformanceError::ModelParse("model parse error: {serde_err}")`; exit non-zero |
| Receipt admission fails | `ConformanceError::Admission(refusal.to_string())`; exit non-zero |
| Feature not compiled | Stub returns structured error; exit non-zero; stderr: `"feature 'conformance' required"` |
| Fitness outside [0,1] | Assert in debug; clamp to [0,1] in release; log a warning |

---

### 2.8 Performance Budget

| Operation | Budget |
|-----------|--------|
| `alignment_fitness_score` (10-event trace) | < 1 s |
| `alignment_fitness_score` (100-event trace) | < 10 s |
| `load_petri_net` (net with 50 transitions) | < 10 ms |

---

### 2.9 Documentation

- [ ] Inline doc on `ConformanceReport` explaining which values are genuine van der Aalst quality dimensions and which are not (match honest labelling in `src/discovery.rs`)
- [ ] Inline doc on `FitnessInterpretation` listing threshold values
- [ ] Update `CLAUDE.md` section "Integration Points" with "Conformance Scoring" subsection
- [ ] Update `README.md` CLI Surface with `affi receipt conform` entry and `--model` flag
- [ ] Add entry to `CHANGELOG.md` (or create it) under `[Unreleased]`

---

## Feature 3: `affi receipt predict` — Next-Activity Forecast

### Description

`affi receipt predict <RECEIPT>` uses `wasm4pm::next_activity()` to predict the next activity in the receipt's process trace, returning a ranked list of (activity, confidence) pairs. The top-K results (default K=5, configurable via `--top-k`) are printed as JSON. The receipt is type-gated on admission.

Feature gate: `--features predictive`

---

### 3.1 Acceptance Criteria

**AC-3.1** Given a receipt with 5 events `[create, transform, transform, review, release]`, when `affi receipt predict receipt.json` is run, then stdout is valid JSON with a `predictions` array of at most 5 items, each with `activity` (string) and `confidence` (float in [0,1]).

**AC-3.2** Given the same receipt run twice, the output JSON is byte-identical (determinism).

**AC-3.3** Given `--top-k 3`, when `predict` is run, then the `predictions` array has at most 3 items.

**AC-3.4** Given `--top-k 0`, when `predict` is run, then exit code is non-zero and stderr contains `"--top-k must be at least 1"`.

**AC-3.5** Given a receipt that fails admission, when `predict` is run, then exit code is non-zero and stderr contains `"admission refused"`.

**AC-3.6** Given `--features predictive` not compiled, when `predict` is run, then exit code is non-zero and stderr contains `"feature 'predictive' required"`.

**AC-3.7** Given a receipt with a single event and an activity that has never appeared as a "next" activity in the training log, when `predict` is run, then the `predictions` array may be empty — not a panic, not an error.

**AC-3.8** Given `--format json`, when `predict` is run, then stdout is parseable as `PredictionReport`.

**AC-3.9** Given a receipt with 50 events, when `predict` is run, then it completes in under 3 seconds.

**AC-3.10** The `confidence` values in `predictions` sum to ≤ 1.0 (they represent a probability distribution over next activities, not independent probabilities).

---

### 3.2 Implementation Checklist

**`src/mining.rs` — extend with prediction**

- [ ] Define `pub struct PredictionReport { pub predictions: Vec<ActivityPrediction>, pub context_length: usize, pub model_type: String }`
- [ ] Define `pub struct ActivityPrediction { pub activity: String, pub confidence: f64 }`
- [ ] Derive `Serialize, Deserialize, Debug, Clone, PartialEq` on both structs
- [ ] Implement `#[cfg(feature = "predictive")] pub fn predict_next(receipt: &AdmittedReceipt, top_k: usize) -> Result<PredictionReport, PredictionError>`
- [ ] Validate `top_k >= 1`, returning `PredictionError::InvalidTopK(k)` otherwise
- [ ] Define `#[derive(Debug, thiserror::Error)] pub enum PredictionError` with variants: `Wasm4pm(String)`, `Admission(String)`, `InvalidTopK(usize)`
- [ ] Implement stub for `#[cfg(not(feature = "predictive"))]`

**`src/verbs/predict.rs` — new file**

- [ ] Create with `#[verb("predict", "receipt")]`
- [ ] Add `--top-k <N>` argument with default 5
- [ ] Delegate to `crate::handlers::predict(receipt, top_k)`

**`src/verbs/mod.rs`**

- [ ] Add `pub mod predict;`

**`src/handlers.rs`**

- [ ] Implement `pub fn predict(receipt: String, top_k: usize) -> Result<()>`

---

### 3.3 Feature Gate Testing

- [ ] Test `predict_with_feature_returns_predictions` (feature enabled)
- [ ] Test `predict_feature_disabled_returns_structured_error` (feature disabled)
- [ ] Test `predict_top_k_zero_returns_error` (always compiled validation)
- [ ] Test `predict_confidence_values_sum_to_one_or_less`

---

### 3.4 API Contracts

```rust
// src/mining.rs

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictionReport {
    /// Top-K ranked next-activity predictions in descending confidence order.
    pub predictions: Vec<ActivityPrediction>,
    /// Number of events in the receipt used as context for prediction.
    pub context_length: usize,
    /// Identifier of the underlying model used for prediction.
    pub model_type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityPrediction {
    /// The predicted next activity label (matches an `event_type` from the receipt).
    pub activity: String,
    /// Confidence in [0.0, 1.0]; sum over all predictions is ≤ 1.0.
    pub confidence: f64,
}

/// Predict the top-K most likely next activities for the receipt's trace.
/// top_k must be >= 1, otherwise returns PredictionError::InvalidTopK.
#[cfg(feature = "predictive")]
pub fn predict_next(
    receipt: &AdmittedReceipt,
    top_k: usize,
) -> Result<PredictionReport, PredictionError>;

#[derive(Debug, thiserror::Error)]
pub enum PredictionError {
    #[error("wasm4pm next_activity failed: {0}")]
    Wasm4pm(String),
    #[error("admission refused: {0}")]
    Admission(String),
    #[error("--top-k must be at least 1, got {0}")]
    InvalidTopK(usize),
}
```

**Example JSON output:**

```json
{
  "predictions": [
    { "activity": "review",  "confidence": 0.62 },
    { "activity": "release", "confidence": 0.28 },
    { "activity": "reject",  "confidence": 0.10 }
  ],
  "context_length": 5,
  "model_type": "next-activity/wasm4pm-v1"
}
```

---

### 3.5 Test Evidence

| Test name | File | Assertion |
|-----------|------|-----------|
| `predict_returns_predictions_within_top_k` | `src/mining.rs` | `predictions.len() <= top_k` |
| `predict_confidence_in_range` | `src/mining.rs` | `0.0 <= c <= 1.0` for all predictions |
| `predict_confidence_sum_le_one` | `src/mining.rs` | `sum(confidences) <= 1.0 + f64::EPSILON` |
| `predict_is_deterministic` | `src/mining.rs` | two calls on same receipt yield equal output |
| `predict_top_k_zero_is_error` | `src/mining.rs` | `Err(PredictionError::InvalidTopK(0))` |
| `predict_feature_disabled_returns_error` | `src/mining.rs` | structured error without panic |
| `e2e_predict_stdout_is_valid_json` | `tests/e2e_discovery.rs` | `serde_json::from_str` succeeds |
| `e2e_predict_top_k_3_at_most_3_predictions` | `tests/e2e_discovery.rs` | `predictions.len() <= 3` |
| `e2e_predict_50_events_under_3s` | `tests/e2e_discovery.rs` | `elapsed < Duration::from_secs(3)` |

---

### 3.6 Error Handling

| Scenario | Behavior |
|----------|----------|
| `wasm4pm::next_activity()` fails | `PredictionError::Wasm4pm(detail)`; exit non-zero |
| `--top-k 0` | `PredictionError::InvalidTopK(0)`; exit non-zero; stderr: `"--top-k must be at least 1, got 0"` |
| Receipt admission fails | `PredictionError::Admission(refusal.to_string())`; exit non-zero |
| Feature not compiled | Stub returns structured error; exit non-zero |
| Empty predictions list | Return `PredictionReport { predictions: vec![], ... }`; exit 0; this is valid (not an error) |

---

### 3.7 Performance Budget

| Operation | Budget |
|-----------|--------|
| `predict_next` (10-event trace, top-k 5) | < 500 ms |
| `predict_next` (50-event trace, top-k 5) | < 3 s |

---

### 3.8 Documentation

- [ ] Module doc on `predict_next` explaining confidence semantics and that it is a probability distribution
- [ ] Update `CLAUDE.md` CLI Surface table with `affi receipt predict`
- [ ] Update `README.md` CLI Surface with `affi receipt predict` and `--top-k` flag

---

## Feature 4: LSP Hover — Event ID Hover Cards

### Description

When a user hovers over a receipt `event_id` string in an lsp-max-driven editor, the hover response shows the event's `event_type`, `payload_commitment` (truncated to 12 chars), `seq`, and a list of `objects`. The implementation lives in `src/lsp/hover.rs` under `#[cfg(feature = "lsp")]`.

The existing `src/lsp.rs` is split into `src/lsp/mod.rs` + `src/lsp/diagnostics.rs` (moved) + `src/lsp/hover.rs` (new). All existing tests pass unchanged after the move.

Feature gate: `--features lsp`

---

### 4.1 Acceptance Criteria

**AC-4.1** Given a receipt with event `{ "id": "evt-1", "event_type": "transform", "payload_commitment": "a2d95f11...", "seq": 1, "objects": [{"id": "file-1", "obj_type": "artifact"}] }`, when `hover_for_event_id("evt-1", &receipt)` is called, then the returned `HoverResponse` contains the string `"transform"` in its markdown content.

**AC-4.2** Given the same event, the hover content contains `"evt-1"`, `"seq: 1"`, and at least the first 12 characters of `payload_commitment`.

**AC-4.3** Given the same event with one object, the hover content contains `"file-1:artifact"`.

**AC-4.4** Given an event with a qualified object `{ "id": "dataset", "obj_type": "artifact", "qualifier": "input" }`, the hover content contains `"dataset:artifact:input"` or `"input"`.

**AC-4.5** Given an event ID that does not exist in the receipt, when `hover_for_event_id` is called, then it returns `None` (not an error, not a panic).

**AC-4.6** Given an empty receipt with no events, when `hover_for_event_id("evt-0", &receipt)` is called, then it returns `None`.

**AC-4.7** Given `--features lsp` not compiled, when `hover_for_event_id` is called, then the function is not defined (the module is `#[cfg(feature = "lsp")]` gated) and a compile error results — failing-when-fake on the compilation axis.

**AC-4.8** Given 100 successive hover calls on a 100-event receipt (simulating editor rapid hover), when all 100 calls complete, then each individual call takes < 100 ms.

**AC-4.9** Given a hover for an event with 10 objects, when the response content is examined, all 10 object IDs appear in the markdown.

**AC-4.10** The `HoverResponse` returned is a genuine `lsp_max::lsp_types::HoverResponse` — removing `lsp-max` must cause a compile error (not a stub returning a `String`).

---

### 4.2 Implementation Checklist

**`src/lsp/mod.rs` — new file (replaces `src/lsp.rs`)**

- [ ] Move all content from `src/lsp.rs` to `src/lsp/diagnostics.rs`
- [ ] `src/lsp/mod.rs` re-exports: `pub use diagnostics::*;` and (under feature gate) `pub use hover::*; pub use goto_definition::*;`
- [ ] Update `src/lib.rs`: `pub mod lsp;` stays unchanged (now resolves to `src/lsp/mod.rs`)
- [ ] Verify all existing tests in `src/lsp/diagnostics.rs` pass without modification

**`src/lsp/hover.rs` — new file**

- [ ] `#[cfg(feature = "lsp")]` wraps the entire module
- [ ] Implement `pub fn hover_for_event_id(event_id: &str, receipt: &Receipt) -> Option<HoverResponse>`
- [ ] Use `lsp_max::lsp_types::HoverResponse` as the return type (not a newtype)
- [ ] Build markdown content string: heading with event_id, table with type/seq/commitment, bullet list of objects
- [ ] Truncate `payload_commitment` to first 12 hex chars followed by `"..."`
- [ ] Format qualified objects as `"id:type:qualifier"`, unqualified as `"id:type"`
- [ ] Return `None` when `event_id` not found in `receipt.events`
- [ ] No I/O, no allocation beyond the response struct

**`src/lib.rs`**

- [ ] No change needed (module path resolves automatically)

**`Cargo.toml`**

- [ ] Add `lsp = ["lsp-max/hover", "lsp-max/goto"]` feature

---

### 4.3 Feature Gate Testing

```bash
# With feature
cargo test --features lsp -- hover
cargo build --features lsp

# Without feature: module does not exist; callers that don't use cfg(feature) get compile error
cargo build  # no features — must succeed
```

- [ ] Test `hover_for_existing_event_contains_event_type` (feature enabled)
- [ ] Test `hover_for_missing_event_returns_none` (feature enabled)
- [ ] Test `hover_content_contains_truncated_commitment` (feature enabled)
- [ ] Test `hover_content_contains_all_objects` (feature enabled)
- [ ] Verify `cargo build` (no features) does not fail due to missing `hover_for_event_id`

---

### 4.4 API Contracts

```rust
// src/lsp/hover.rs  (only compiled with --features lsp)
#[cfg(feature = "lsp")]

use crate::types::Receipt;
use lsp_max::lsp_types::HoverResponse;

/// Produce a hover card for the event with the given ID.
/// Returns None if no event with that ID exists in the receipt.
///
/// Content is Markdown. The commitment is truncated to 12 hex chars + "...".
/// Objects are listed as "id:type" or "id:type:qualifier".
pub fn hover_for_event_id(event_id: &str, receipt: &Receipt) -> Option<HoverResponse>;
```

---

### 4.5 LSP Hover Response JSON Schema

The wire format of an `lsp_max::lsp_types::HoverResponse` for a receipt event:

```json
{
  "contents": {
    "kind": "markdown",
    "value": "## evt-1\n\n| Field | Value |\n|-------|-------|\n| event_type | `transform` |\n| seq | `1` |\n| commitment | `a2d95f11b4e2...` |\n\n**Objects:**\n- `file-1:artifact`\n- `dataset:artifact:input`\n"
  },
  "range": null
}
```

- `contents.kind` must be `"markdown"` (not `"plaintext"`).
- `range` is `null` (no source span for receipt events; they are referenced by ID, not file position).
- The commitment is always truncated to first 12 hex chars + `"..."` regardless of full length.

---

### 4.6 Test Evidence

| Test name | File | Assertion |
|-----------|------|-----------|
| `hover_returns_some_for_valid_event_id` | `src/lsp/hover.rs` | `result.is_some()` |
| `hover_content_contains_event_type` | `src/lsp/hover.rs` | markdown contains `event_type` value |
| `hover_content_contains_seq` | `src/lsp/hover.rs` | markdown contains `"seq"` and seq value as string |
| `hover_content_contains_truncated_commitment` | `src/lsp/hover.rs` | markdown contains first 12 chars of commitment |
| `hover_content_does_not_contain_full_commitment` | `src/lsp/hover.rs` | full 64-char hex is not present (only truncated form) |
| `hover_content_lists_all_objects` | `src/lsp/hover.rs` | markdown contains `"id:type"` for every object |
| `hover_qualified_object_includes_qualifier` | `src/lsp/hover.rs` | markdown contains qualifier string |
| `hover_missing_event_returns_none` | `src/lsp/hover.rs` | `result.is_none()` |
| `hover_empty_receipt_returns_none` | `src/lsp/hover.rs` | empty receipt → `None` |
| `hover_response_is_lsp_max_type` | `src/lsp/hover.rs` | `HoverResponse` is `lsp_max::lsp_types::HoverResponse` (compile-time) |
| `hover_100_calls_under_100ms_each` | `src/lsp/hover.rs` | timing assertion in a loop |
| `e2e_diagnostics_still_pass_after_lsp_refactor` | `tests/e2e_discovery.rs` | all existing lsp tests pass after split |

---

### 4.7 Error Handling

| Scenario | Behavior |
|----------|----------|
| Event ID not in receipt | Return `None`; never panic |
| Receipt with 0 events | Return `None` for any query |
| `event_type` is unexpectedly empty | Include empty string in markdown; do not panic (admission gate prevents this in practice) |
| `payload_commitment` shorter than 12 chars | Display full string without truncation marker |
| lsp-max unavailable (feature not compiled) | Module is `#[cfg(feature = "lsp")]` and not compiled; callers must guard with `#[cfg(feature = "lsp")]` |

---

### 4.8 Performance Budget

| Operation | Budget |
|-----------|--------|
| Single `hover_for_event_id` call (any receipt size) | < 100 ms |
| 100 successive hover calls on 100-event receipt | < 10 s total |

Hover must be allocation-efficient: it traverses `receipt.events` linearly and builds a single `String`. No database, no index, no caching required.

---

### 4.9 Documentation

- [ ] Module-level doc in `src/lsp/hover.rs` explaining that `range: None` is intentional and why
- [ ] Add `//! # Example` pointing to a new example or the existing `examples/verdict_diagnostics.rs`
- [ ] Update `CLAUDE.md` section "Integration Points → LSP" with hover subsection
- [ ] Update `README.md` under "Integration Points → LSP"
- [ ] Note in docs that the `src/lsp.rs` → `src/lsp/` refactor is non-breaking (same public API)

---

## Feature 5: LSP Goto-Definition — Event Type → Handler Source

### Description

When a user triggers goto-definition on a receipt `event_type` value in an lsp-max-driven editor, the server returns a `GotoDefinitionResponse` pointing to the source file and line number of the Rust handler function for that event type. For example, `event_type = "emit"` resolves to `src/verbs/emit.rs` at line 1. The mapping is static (a `BTreeMap` compiled into the binary), not a runtime file search.

Feature gate: `--features lsp` (same gate as hover)

---

### 5.1 Acceptance Criteria

**AC-5.1** Given a receipt event with `event_type = "emit"`, when `goto_definition_for_event_type("emit")` is called, then the returned `GotoDefinitionResponse` has `uri` pointing to a path ending in `src/verbs/emit.rs` and `range.start.line` is 0 (line 1 in 0-indexed LSP convention).

**AC-5.2** Given `event_type = "assemble"`, when goto-definition is called, then `uri` ends in `src/verbs/assemble.rs`.

**AC-5.3** Given `event_type = "verify"`, then `uri` ends in `src/verbs/verify.rs`.

**AC-5.4** Given `event_type = "conform"` (Phase 2 new verb), then `uri` ends in `src/verbs/conform.rs`.

**AC-5.5** Given an `event_type` with no registered handler (e.g., `"unknown-op"`), when goto-definition is called, then it returns `None` — not an error, not a panic.

**AC-5.6** Given an empty `event_type` string, then goto-definition returns `None`.

**AC-5.7** Given `--features lsp` not compiled, then the module is not compiled and callers that reference it unconditionally get a compile error — failing-when-fake.

**AC-5.8** Given the full set of known event types (`emit`, `assemble`, `verify`, `show`, `inspect`, `model`, `diagnose`, `conform`, `replay`, `graph`, `stats`, `predict`), when goto-definition is called for each, then all return `Some(GotoDefinitionResponse)` with a valid file path.

**AC-5.9** The `GotoDefinitionResponse` is a genuine `lsp_max::lsp_types::GotoDefinitionResponse` — removing `lsp-max` must cause a compile error.

**AC-5.10** The static handler map must be a `BTreeMap<&'static str, (&'static str, u32)>` initialized at startup (not regenerated per call) for predictable performance.

---

### 5.2 Implementation Checklist

**`src/lsp/goto_definition.rs` — new file**

- [ ] `#[cfg(feature = "lsp")]` wraps the entire module
- [ ] Define `static HANDLER_MAP: std::sync::OnceLock<std::collections::BTreeMap<&'static str, (&'static str, u32)>>` initialized with all known event types
- [ ] Populate `HANDLER_MAP` entries for: `emit`, `assemble`, `verify`, `show`, `inspect`, `model`, `diagnose`, `conform`, `replay`, `graph`, `stats`, `predict`
- [ ] Each entry: `("event_type", ("src/verbs/event_type.rs", 0u32))` (0-indexed line number pointing to the `pub fn` or `#[verb]` attribute)
- [ ] Implement `pub fn goto_definition_for_event_type(event_type: &str) -> Option<GotoDefinitionResponse>`
- [ ] Use `lsp_max::lsp_types::{GotoDefinitionResponse, Location, Range, Position, Uri}` (or equivalent from `lsp-max`)
- [ ] Return `None` for empty string or unknown `event_type`
- [ ] Build `Uri` from the relative path using `Uri::from_file_path` or the lsp-max equivalent
- [ ] The `range` is a zero-width range at `(line, 0)` to `(line, 0)` — pointing to the beginning of the handler definition line

**`src/lsp/mod.rs`**

- [ ] Add `#[cfg(feature = "lsp")] pub mod goto_definition;`
- [ ] Add `#[cfg(feature = "lsp")] pub use goto_definition::*;`

---

### 5.3 Feature Gate Testing

- [ ] Test `goto_definition_emit_resolves_to_emit_rs` (feature enabled)
- [ ] Test `goto_definition_unknown_returns_none` (feature enabled)
- [ ] Test `goto_definition_empty_string_returns_none` (feature enabled)
- [ ] Test `goto_definition_all_known_types_resolve` (feature enabled)
- [ ] Verify `cargo build` (no features) succeeds

---

### 5.4 API Contracts

```rust
// src/lsp/goto_definition.rs  (only compiled with --features lsp)
#[cfg(feature = "lsp")]

use lsp_max::lsp_types::{GotoDefinitionResponse, Location, Position, Range};

/// Static mapping: event_type → (handler file path, 0-indexed line number).
/// Initialized once via OnceLock; lookup is O(log n) via BTreeMap.
static HANDLER_MAP: std::sync::OnceLock<
    std::collections::BTreeMap<&'static str, (&'static str, u32)>
> = std::sync::OnceLock::new();

fn handler_map() -> &'static std::collections::BTreeMap<&'static str, (&'static str, u32)> {
    HANDLER_MAP.get_or_init(|| {
        let mut m = std::collections::BTreeMap::new();
        m.insert("emit",      ("src/verbs/emit.rs",      0));
        m.insert("assemble",  ("src/verbs/assemble.rs",  0));
        m.insert("verify",    ("src/verbs/verify.rs",    0));
        m.insert("show",      ("src/verbs/show.rs",      0));
        m.insert("inspect",   ("src/verbs/inspect.rs",   0));
        m.insert("model",     ("src/verbs/model.rs",     0));
        m.insert("diagnose",  ("src/verbs/diagnose.rs",  0));
        m.insert("conform",   ("src/verbs/conform.rs",   0));
        m.insert("replay",    ("src/verbs/replay.rs",    0));
        m.insert("graph",     ("src/verbs/graph.rs",     0));
        m.insert("stats",     ("src/verbs/stats.rs",     0));
        m.insert("predict",   ("src/verbs/predict.rs",   0));
        m
    })
}

/// Return the LSP goto-definition location for a receipt event_type.
/// Returns None if the event_type has no registered handler.
pub fn goto_definition_for_event_type(event_type: &str) -> Option<GotoDefinitionResponse>;
```

---

### 5.5 LSP Goto-Definition Response JSON Schema

```json
{
  "uri": "file:///home/user/affidavit/src/verbs/emit.rs",
  "range": {
    "start": { "line": 0, "character": 0 },
    "end":   { "line": 0, "character": 0 }
  }
}
```

Notes:
- `uri` is an absolute `file://` URI. The base path is resolved at call time from the process working directory or a provided workspace root.
- `range` is a zero-width point range at the beginning of the handler file. This is intentional: the handler occupies the whole file, so the definition is at its start.
- Line numbers are 0-indexed (LSP convention). Line 0 = the first line of the file.

---

### 5.6 Handler Map: Complete Mapping Table

| `event_type` | Handler file | 0-indexed line | Notes |
|-------------|-------------|---------------|-------|
| `emit` | `src/verbs/emit.rs` | 0 | `#[verb("emit", "receipt")]` at line 9 → point to top for now |
| `assemble` | `src/verbs/assemble.rs` | 0 | |
| `verify` | `src/verbs/verify.rs` | 0 | |
| `show` | `src/verbs/show.rs` | 0 | |
| `inspect` | `src/verbs/inspect.rs` | 0 | |
| `model` | `src/verbs/model.rs` | 0 | |
| `diagnose` | `src/verbs/diagnose.rs` | 0 | |
| `conform` | `src/verbs/conform.rs` | 0 | Phase 2 new verb |
| `replay` | `src/verbs/replay.rs` | 0 | |
| `graph` | `src/verbs/graph.rs` | 0 | |
| `stats` | `src/verbs/stats.rs` | 0 | |
| `predict` | `src/verbs/predict.rs` | 0 | Phase 2 new verb |

When a new verb is added, the handler map **must** be updated in the same PR. This is enforced by the `goto_definition_all_known_types_resolve` test, which will fail if a verb exists in `src/verbs/` but not in the map.

---

### 5.7 Test Evidence

| Test name | File | Assertion |
|-----------|------|-----------|
| `goto_emit_resolves_to_emit_rs` | `src/lsp/goto_definition.rs` | `uri` contains `"emit.rs"`, `range.start.line == 0` |
| `goto_assemble_resolves_to_assemble_rs` | `src/lsp/goto_definition.rs` | `uri` contains `"assemble.rs"` |
| `goto_verify_resolves_to_verify_rs` | `src/lsp/goto_definition.rs` | `uri` contains `"verify.rs"` |
| `goto_conform_resolves_to_conform_rs` | `src/lsp/goto_definition.rs` | `uri` contains `"conform.rs"` (Phase 2) |
| `goto_predict_resolves_to_predict_rs` | `src/lsp/goto_definition.rs` | `uri` contains `"predict.rs"` (Phase 2) |
| `goto_unknown_event_type_returns_none` | `src/lsp/goto_definition.rs` | `result.is_none()` |
| `goto_empty_event_type_returns_none` | `src/lsp/goto_definition.rs` | `result.is_none()` |
| `goto_all_known_types_resolve` | `src/lsp/goto_definition.rs` | all 12 known types return `Some` |
| `goto_response_is_lsp_max_type` | `src/lsp/goto_definition.rs` | type check at compile time |
| `goto_handler_map_initialized_once` | `src/lsp/goto_definition.rs` | two calls return `ptr::eq` to same map |

---

### 5.8 Error Handling

| Scenario | Behavior |
|----------|----------|
| `event_type` not in `HANDLER_MAP` | Return `None`; no error logged (caller decides whether to show UI hint) |
| Empty string `event_type` | Return `None` |
| Feature not compiled | Module not compiled; callers must use `#[cfg(feature = "lsp")]` guard |
| `OnceLock` initialization panic | Not possible: initializer is infallible (pure `BTreeMap` construction) |

---

### 5.9 Performance Budget

| Operation | Budget |
|-----------|--------|
| Single `goto_definition_for_event_type` call | < 1 ms |
| `HANDLER_MAP` initialization (first call) | < 1 ms |
| 1000 successive goto calls | < 100 ms total |

---

### 5.10 Documentation

- [ ] Module-level doc in `src/lsp/goto_definition.rs` explaining static map rationale and the LSP range convention
- [ ] Comment in `HANDLER_MAP` initializer: "Update when adding a new verb to src/verbs/"
- [ ] Update `CLAUDE.md` section "Integration Points → LSP" with goto-definition subsection
- [ ] Update `README.md` under "Integration Points → LSP"

---

## E2E Test Template: `tests/e2e_discovery.rs`

This file is the integration witness for all Phase 2 features. Copy and expand; do not delete existing tests.

```rust
//! End-to-end tests for Phase 2: Process Discovery & IDE Integration.
//!
//! Each test exercises the real `affi` binary (built via `cargo build`) against
//! real receipt files produced by emit → assemble in a TempDir. Tests are
//! failing-when-fake: removing wasm4pm, lsp-max, or the feature gate causes
//! compilation failure or assertion failure — not a silent pass.
//!
//! Run with:
//!   cargo test --features "discovery conformance predictive lsp" --test e2e_discovery
//!
//! Individual feature subsets:
//!   cargo test --features discovery    --test e2e_discovery model
//!   cargo test --features conformance  --test e2e_discovery conform
//!   cargo test --features predictive   --test e2e_discovery predict
//!   cargo test --features lsp          --test e2e_discovery lsp

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::TempDir;

// ─── Helpers ────────────────────────────────────────────────────────────────

fn affi(dir: &TempDir) -> Command {
    let mut c = Command::cargo_bin("affi").expect("affi binary builds");
    c.current_dir(dir.path());
    c
}

/// Build a receipt with the given event types, assemble to `receipt_name`.
/// Returns the path to the assembled receipt file.
fn build_receipt(dir: &TempDir, activities: &[&str], receipt_name: &str) -> PathBuf {
    for (i, activity) in activities.iter().enumerate() {
        let obj = format!("obj-{i}:artifact");
        affi(dir)
            .args([
                "receipt", "emit",
                "--type", activity,
                "--object", &obj,
                "--payload", "-",
            ])
            .write_stdin(*activity)
            .assert()
            .success();
    }
    affi(dir)
        .args(["receipt", "assemble", "--out", receipt_name])
        .assert()
        .success();
    dir.path().join(receipt_name)
}

/// Build a 100-event receipt with 5 cycling activities.
fn build_large_receipt(dir: &TempDir, receipt_name: &str) -> PathBuf {
    let activities = ["create", "transform", "review", "approve", "release"];
    let all: Vec<&str> = (0..100).map(|i| activities[i % 5]).collect();
    build_receipt(dir, &all, receipt_name)
}

// ─── Feature 1: model ───────────────────────────────────────────────────────

#[test]
#[cfg(feature = "discovery")]
fn e2e_model_produces_valid_petri_net_json() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, &["create", "transform", "release"], "r.json");

    let output = affi(&dir)
        .args(["receipt", "model", "r.json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).expect("utf8 stdout");
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("model output must be valid JSON");

    assert!(parsed.get("places").is_some(),    "JSON must have 'places' key");
    assert!(parsed.get("transitions").is_some(), "JSON must have 'transitions' key");
    assert!(parsed.get("arcs").is_some(),      "JSON must have 'arcs' key");
}

#[test]
#[cfg(feature = "discovery")]
fn e2e_model_stderr_reports_activity_count() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, &["create", "transform", "release"], "r.json");

    affi(&dir)
        .args(["receipt", "model", "r.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("transitions"));
}

#[test]
#[cfg(feature = "discovery")]
fn e2e_model_activities_appear_as_transitions() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, &["create", "transform", "release"], "r.json");

    let output = affi(&dir)
        .args(["receipt", "model", "r.json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).expect("utf8");
    assert!(stdout.contains("create"),    "model must mention 'create' activity");
    assert!(stdout.contains("transform"), "model must mention 'transform' activity");
    assert!(stdout.contains("release"),   "model must mention 'release' activity");
}

#[test]
#[cfg(feature = "discovery")]
fn e2e_model_is_deterministic() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, &["create", "transform", "release"], "r.json");

    let run1 = affi(&dir)
        .args(["receipt", "model", "r.json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let run2 = affi(&dir)
        .args(["receipt", "model", "r.json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(run1, run2, "model output must be deterministic across runs");
}

#[test]
#[cfg(feature = "discovery")]
fn e2e_model_missing_receipt_exits_nonzero() {
    let dir = TempDir::new().expect("tempdir");

    affi(&dir)
        .args(["receipt", "model", "does_not_exist.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("does_not_exist.json"));
}

#[test]
#[cfg(feature = "discovery")]
fn e2e_model_100_events_under_5_seconds() {
    let dir = TempDir::new().expect("tempdir");
    build_large_receipt(&dir, "large.json");

    let start = Instant::now();
    affi(&dir)
        .args(["receipt", "model", "large.json"])
        .assert()
        .success();
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(5),
        "model on 100-event receipt must complete in < 5s; took {elapsed:?}"
    );
}

#[test]
#[cfg(feature = "discovery")]
fn e2e_model_objectless_receipt_refused_at_admission() {
    use affidavit::chain::{recompute_chain, FORMAT_VERSION};
    use affidavit::types::{Blake3Hash, OperationEvent};

    let dir = TempDir::new().expect("tempdir");

    // Build an objectless receipt that passes chain integrity but fails OCEL court
    let event = OperationEvent {
        id: "evt-0".to_string(),
        seq: 0,
        event_type: "create".to_string(),
        objects: vec![], // empty room — OCEL court refuses this
        payload_commitment: Blake3Hash::from_bytes(b"payload"),
    };
    let chain_hash = recompute_chain(std::slice::from_ref(&event)).expect("chain");
    let json = serde_json::json!({
        "format_version": FORMAT_VERSION,
        "events": [event],
        "chain_hash": chain_hash,
    });
    fs::write(
        dir.path().join("objectless.json"),
        serde_json::to_string(&json).expect("serialize"),
    ).expect("write");

    affi(&dir)
        .args(["receipt", "model", "objectless.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("EmptyEventObjectLinks").or(
            predicate::str::contains("admission refused")
        ));
}

// ─── Feature 2: conform ─────────────────────────────────────────────────────

#[test]
#[cfg(feature = "conformance")]
fn e2e_conform_self_model_high_fitness() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, &["create", "transform", "release"], "r.json");

    // First discover the model
    let model_output = affi(&dir)
        .args(["receipt", "model", "r.json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    fs::write(dir.path().join("model.json"), &model_output).expect("write model");

    // Then conform against it
    let conform_output = affi(&dir)
        .args(["receipt", "conform", "r.json", "--model", "model.json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: serde_json::Value = serde_json::from_slice(&conform_output)
        .expect("conform output must be valid JSON");

    let fitness = report["fitness"].as_f64().expect("fitness must be a number");
    assert!(
        fitness >= 0.0 && fitness <= 1.0,
        "fitness must be in [0.0, 1.0]; got {fitness}"
    );
    // Self-model must fit well
    assert!(
        fitness >= 0.6,
        "receipt conformed against its own model must score >= 0.6; got {fitness}"
    );
}

#[test]
#[cfg(feature = "conformance")]
fn e2e_conform_output_contains_interpretation() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, &["create", "transform", "release"], "r.json");

    let model_output = affi(&dir)
        .args(["receipt", "model", "r.json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    fs::write(dir.path().join("model.json"), &model_output).expect("write model");

    affi(&dir)
        .args(["receipt", "conform", "r.json", "--model", "model.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("interpretation"));
}

#[test]
#[cfg(feature = "conformance")]
fn e2e_conform_missing_model_exits_nonzero() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, &["create"], "r.json");

    affi(&dir)
        .args(["receipt", "conform", "r.json", "--model", "missing_model.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing_model.json"));
}

#[test]
#[cfg(all(feature = "discovery", feature = "conformance"))]
fn e2e_conform_100_events_under_10_seconds() {
    let dir = TempDir::new().expect("tempdir");
    build_large_receipt(&dir, "large.json");

    let model_output = affi(&dir)
        .args(["receipt", "model", "large.json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    fs::write(dir.path().join("large_model.json"), &model_output).expect("write model");

    let start = Instant::now();
    affi(&dir)
        .args(["receipt", "conform", "large.json", "--model", "large_model.json"])
        .assert()
        .success();
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(10),
        "conform on 100-event receipt must complete in < 10s; took {elapsed:?}"
    );
}

// ─── Feature 3: predict ─────────────────────────────────────────────────────

#[test]
#[cfg(feature = "predictive")]
fn e2e_predict_stdout_is_valid_json() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, &["create", "transform", "review", "approve", "release"], "r.json");

    let output = affi(&dir)
        .args(["receipt", "predict", "r.json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: serde_json::Value = serde_json::from_slice(&output)
        .expect("predict output must be valid JSON");

    assert!(parsed.get("predictions").is_some(), "JSON must have 'predictions' key");
    assert!(parsed.get("context_length").is_some(), "JSON must have 'context_length' key");
}

#[test]
#[cfg(feature = "predictive")]
fn e2e_predict_top_k_3_at_most_3_predictions() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, &["create", "transform", "review", "approve", "release"], "r.json");

    let output = affi(&dir)
        .args(["receipt", "predict", "r.json", "--top-k", "3"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: serde_json::Value = serde_json::from_slice(&output).expect("json");
    let predictions = parsed["predictions"].as_array().expect("predictions array");

    assert!(
        predictions.len() <= 3,
        "with --top-k 3, at most 3 predictions expected; got {}",
        predictions.len()
    );
}

#[test]
#[cfg(feature = "predictive")]
fn e2e_predict_confidence_values_in_range() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, &["create", "transform", "review", "approve", "release"], "r.json");

    let output = affi(&dir)
        .args(["receipt", "predict", "r.json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: serde_json::Value = serde_json::from_slice(&output).expect("json");
    let predictions = parsed["predictions"].as_array().expect("predictions array");

    for (i, pred) in predictions.iter().enumerate() {
        let confidence = pred["confidence"].as_f64()
            .unwrap_or_else(|| panic!("prediction {i} must have numeric confidence"));
        assert!(
            (0.0..=1.0).contains(&confidence),
            "prediction {i} confidence must be in [0,1]; got {confidence}"
        );
    }
}

#[test]
#[cfg(feature = "predictive")]
fn e2e_predict_is_deterministic() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, &["create", "transform", "release"], "r.json");

    let run1 = affi(&dir).args(["receipt", "predict", "r.json"])
        .assert().success().get_output().stdout.clone();
    let run2 = affi(&dir).args(["receipt", "predict", "r.json"])
        .assert().success().get_output().stdout.clone();

    assert_eq!(run1, run2, "predict output must be deterministic");
}

#[test]
#[cfg(feature = "predictive")]
fn e2e_predict_top_k_zero_exits_nonzero() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, &["create"], "r.json");

    affi(&dir)
        .args(["receipt", "predict", "r.json", "--top-k", "0"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("top-k").or(predicate::str::contains("at least 1")));
}

#[test]
#[cfg(feature = "predictive")]
fn e2e_predict_50_events_under_3_seconds() {
    let dir = TempDir::new().expect("tempdir");
    let activities: Vec<&str> = (0..50)
        .map(|i| ["create", "transform", "review", "approve", "release"][i % 5])
        .collect();
    build_receipt(&dir, &activities, "medium.json");

    let start = Instant::now();
    affi(&dir)
        .args(["receipt", "predict", "medium.json"])
        .assert()
        .success();
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(3),
        "predict on 50-event receipt must complete in < 3s; took {elapsed:?}"
    );
}

// ─── LSP: existing diagnostics still pass after src/lsp.rs → src/lsp/ refactor ──

#[test]
#[cfg(feature = "lsp")]
fn e2e_diagnostics_clean_receipt_produces_no_squiggles() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, &["create", "transform", "release"], "r.json");

    affi(&dir)
        .args(["receipt", "diagnose", "r.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("no diagnostics — receipt is clean"));
}

#[test]
fn e2e_phase1_full_pipeline_still_passes_after_phase2() {
    // Regression guard: the full Phase 1 pipeline must still pass.
    // This is a copy of dx_full_pipeline_e2e.rs::full_dx_pipeline_through_the_binary.
    let dir = TempDir::new().expect("tempdir");

    for (ty, obj) in [
        ("create",    "f:artifact"),
        ("transform", "d:artifact"),
        ("release",   "f:artifact"),
    ] {
        affi(&dir)
            .args(["receipt", "emit", "--type", ty, "--object", obj, "--payload", "-"])
            .write_stdin(ty)
            .assert()
            .success();
    }

    affi(&dir).args(["receipt", "assemble", "--out", "r.json"]).assert().success();
    affi(&dir).args(["receipt", "verify",   "r.json"]).assert().success()
        .stderr(predicate::str::contains("verdict: ACCEPT"));
    affi(&dir).args(["receipt", "inspect",  "r.json"]).assert().success()
        .stderr(predicate::str::contains("RECEIPT INSPECTION REPORT"));
    affi(&dir).args(["receipt", "model",    "r.json"]).assert().success()
        .stderr(predicate::str::contains("create"));
    affi(&dir).args(["receipt", "conformance", "r.json"]).assert().success()
        .stderr(predicate::str::contains("fitness (token replay):"));
    affi(&dir).args(["receipt", "diagnose", "r.json"]).assert().success()
        .stderr(predicate::str::contains("no diagnostics"));
}
```

---

## Cross-Cutting DoD Requirements

### Compile Gates (must all pass on branch `claude/zen-cerf-oq87br`)

```bash
# 1. No features: existing Phase 1 tests pass
cargo test

# 2. All Phase 2 features enabled: no warnings under deny(warnings)
RUSTFLAGS="-D warnings" cargo test --features "discovery conformance predictive lsp"

# 3. Feature combinations: partial combos must compile
cargo build --features discovery
cargo build --features "discovery conformance"
cargo build --features lsp

# 4. Each feature independently: no cross-contamination
cargo build --features discovery
cargo build --features conformance
cargo build --features predictive
cargo build --features lsp

# 5. docs compile
cargo doc --features "discovery conformance predictive lsp" --no-deps
```

All five commands must exit 0 before Phase 2 is considered Done.

---

### Test Count Targets

| Category | Phase 1 baseline | Phase 2 additions | Phase 2 total |
|----------|-----------------|------------------|---------------|
| Unit (lib) | 19 | ≥ 30 new tests across `src/mining.rs`, `src/lsp/hover.rs`, `src/lsp/goto_definition.rs` | ≥ 49 |
| Dispatch (handlers) | 6 | ≥ 4 new tests for `conform`, `predict` handlers | ≥ 10 |
| E2E | 4 (e2e.rs) + 1 (dx_full_pipeline) | ≥ 18 new tests in `tests/e2e_discovery.rs` | ≥ 23 |
| UI | 1 | ≥ 2 new (conform + predict verb registration) | ≥ 3 |
| **Total** | **30** | **≥ 54** | **≥ 84** |

---

### No Regression Policy

All 30 Phase 1 tests must still pass after every Phase 2 commit. This is enforced by `e2e_phase1_full_pipeline_still_passes_after_phase2` and by running `cargo test` (no features) in CI.

---

### CI Requirements

Add the following matrix to `.github/workflows/ci.yml` (or equivalent):

```yaml
strategy:
  matrix:
    features:
      - ""
      - "discovery"
      - "conformance"
      - "predictive"
      - "lsp"
      - "discovery conformance predictive lsp"
steps:
  - run: cargo test --features "${{ matrix.features }}"
```

---

### Changelog Entry

Before merging Phase 2, add to `CHANGELOG.md` under `[Unreleased]`:

```markdown
### Added
- `affi receipt model`: Heuristic Inductive Miner via wasm4pm produces Petri net JSON (`--features discovery`)
- `affi receipt conform --model`: alignment fitness scoring 0–1 with interpretation labels (`--features conformance`)
- `affi receipt predict`: next-activity forecast with confidence and --top-k (`--features predictive`)
- LSP hover: hover on receipt event ID returns markdown card with event_type, commitment, objects (`--features lsp`)
- LSP goto-definition: event_type → handler source file mapping for all 12 known verbs (`--features lsp`)
- `src/mining.rs`: `receipt_to_ocel`, `discover_him`, `alignment_fitness_score`, `predict_next`
- `src/lsp/`: refactored from flat `src/lsp.rs` to module tree with `diagnostics`, `hover`, `goto_definition`

### Changed
- `src/lsp.rs` moved to `src/lsp/diagnostics.rs`; public API unchanged (all re-exported from `src/lsp/mod.rs`)
- `Cargo.toml`: added features `discovery`, `conformance`, `predictive`, `lsp`
```

---

### Review Checklist (PR author fills before requesting review)

- [ ] `cargo test` (no features) passes — 30 Phase 1 tests green
- [ ] `cargo test --features "discovery conformance predictive lsp"` passes — all Phase 2 tests green
- [ ] `RUSTFLAGS="-D warnings" cargo build --features "discovery conformance predictive lsp"` exits 0
- [ ] `cargo doc --features "discovery conformance predictive lsp" --no-deps` exits 0
- [ ] `src/mining.rs` has module-level doc comment
- [ ] `src/lsp/hover.rs` has module-level doc comment
- [ ] `src/lsp/goto_definition.rs` has module-level doc comment
- [ ] `CLAUDE.md` updated: Integration Points section
- [ ] `README.md` updated: CLI Surface table
- [ ] `CHANGELOG.md` updated: `[Unreleased]` section
- [ ] No `.unwrap()` in library code (only in test functions)
- [ ] No `println!` in library modules (only `eprintln!` in handlers; stdout used only for structured output)
- [ ] Performance budgets verified with timing assertions in tests
- [ ] Determinism verified: `model` and `predict` tested for byte-identical output across two runs
- [ ] Honest labelling: `ConformanceReport` doc states which dimensions are genuine van der Aalst and which are not
- [ ] Handler map in `goto_definition.rs` includes all 12 verbs including Phase 2 additions

---

*End of Definition of Done — Phase 2: Process Discovery & IDE Integration*
