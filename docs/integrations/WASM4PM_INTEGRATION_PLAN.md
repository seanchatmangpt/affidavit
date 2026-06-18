# wasm4pm Integration Plan v26.6.17

**Date:** 2026-06-14  
**Affidavit Version:** 26.6.17  
**wasm4pm-compat:** 26.6.17 (stable, nightly-required)  
**wasm4pm:** 26.6.12 (workspace with discovery/conformance/predictive engines)

---

## Executive Summary

Affidavit currently follows emit→assemble→verify→show, producing deterministic BLAKE3-sealed receipts that certify receipt format (7-stage pipeline). **wasm4pm integration** adds the process-mining dimension: discover what process *actually happened* from receipt traces, check conformance against declared model, and predict next activities in ongoing traces.

**80/20 split:**
- **80% (off-shelf wasm4pm):** Discovery engines (Heuristic Inductive Miner, Alpha Miner), conformance checkers (alignment-based), predictive monitors (LSTM-ready), OCEL→JSON I/O
- **20% (glue code):** Receipt → OCEL conversion, OTel span hooks, witness tests, CLI integration

**Result:** One unified verifier that both certifies receipt format (structural) AND asserts process lawfulness (behavioral).

---

## Current Architecture

```
┌─── emit ──────┐        ┌─── assemble ──────┐       ┌─── verify ─────────┐
│ Objects       │        │ ChainAssembler    │       │ 7-stage pipeline   │
│ Event type    │──────▶ │ BLAKE3 rolling    │──────▶ │ 1. decode          │
│ Payload       │        │ hash              │        │ 2. check_format    │
│ Commit        │        │ Content-address   │        │ 3. chain_integrity │
│               │        │ Finalize (seal)   │        │ 4. continuity      │
└─── working ───┘        └─── receipt.json ──┘        │ 5. verify_commits  │
                                                       │ 6. evaluate_profile│
                                                       │ 7. emit_verdict    │
                                                       └────────────────────┘
                                                                │
                                                                ▼
                                                            ┌─ show ─┐
                                                            │ Human  │
                                                            │ dump   │
                                                            └────────┘
```

**Key types:**
- `Receipt` — typed struct with private `_seal` field (unconstructable)
- `OperationEvent` — event_type, seq, objects, payload_commitment
- `ObjectRef` — OCEL-shaped (id, obj_type, qualifier)
- `Verdict` — accepted/rejected with reason
- Output: `AdmittedReceipt = Evidence<Receipt, Admitted, AffidavitReceiptChain>`

---

## wasm4pm Capability Surface (Off-Shelf)

### 1. Discovery Module (`wasm4pm::mining::*`)
- **Heuristic Inductive Miner (HIM)** — Default, fast, handles noise
  - Input: OCEL event log
  - Output: Petri net (places, transitions, arcs)
  - Best for: Receipt audit trails with concurrent activities
- **Alpha Miner** — Strict, deterministic
  - Input: Finite event log
  - Output: Petri net (WF-net subset)
  - Best for: Receipts with linear, deterministic flow
- **Inductive Miner (IM)** — Recursive, handles long traces
  - Input: Event log
  - Output: Process tree / Petri net
  - Best for: Large, complex receipt chains

### 2. Conformance Module (`wasm4pm::conformance::*`)
- **Alignment-based conformance** (van Dongen 2008 standard)
  - Input: Petri net (model), OCEL event log (trace)
  - Output: Fitness (0–1), cost matrix, violated transitions
  - Metrics: `fitness`, `precision`, `generalization`, `simplicity`
- **Reply-based variant** (model checker)
  - Input: Same
  - Output: Yes/no (perfect fit only)

### 3. Predictive Monitoring (`wasm4pm::predictive::*`)
- **Next-Activity Prediction**
  - Input: Prefix trace (ongoing receipt)
  - Output: Probability distribution over possible next activities
- **Remaining-Time Prediction** (LSTM-ready, optional)
  - Input: Prefix trace
  - Output: Estimated remaining event count / wall-clock time

---

## 20% Glue: Integration Points

### 1. Receipt → OCEL Conversion (NEW)
**File:** `src/mining.rs` (new module)

```rust
/// Convert a Receipt's operation-events into a wasm4pm OCEL event log.
/// 
/// Each OperationEvent becomes an Event with:
/// - activity = event_type
/// - timestamp = logical seq (derived wall-clock if needed)
/// - objects = qualified refs from the receipt
///
/// Returns EventLog (wasm4pm_compat type) ready for mining.
pub fn receipt_to_ocel(receipt: &Receipt) -> Result<wasm4pm_compat::ocel::EventLog>;
```

**Mapping:**
| Receipt | OCEL |
|---------|------|
| `OperationEvent::id` | Event id (unique) |
| `OperationEvent::event_type` | Activity name |
| `OperationEvent::seq` | Logical timestamp (0, 1, 2, ...) |
| `ObjectRef::id` | Object id |
| `ObjectRef::obj_type` | Object type |
| `ObjectRef::qualifier` | Role/relationship (optional) |
| `OperationEvent::payload_commitment` | Audit trail link (metadata) |

### 2. Discovery Hook in `verify()` (NEW stage)
**File:** `src/verifier.rs` (new stage 8)

```rust
/// Stage 8 (optional): Process Discovery
/// Run Heuristic Inductive Miner on the receipt's OCEL log.
/// 
/// Produces a discovered Petri net model and names the mining witness.
fn stage_discover_process(receipt: &Receipt) -> CheckOutcome;
```

**Logic:**
```
1. Convert receipt → OCEL
2. Run wasm4pm::mining::heuristic_inductive_miner(&log)
3. Return net (or skip if log too small)
4. Outcome: "discovered N places, M transitions" (informational)
```

**Integration:** Optional; gated by feature flag `discovery`.

### 3. Conformance Check in `verify()` (NEW stage)
**File:** `src/verifier.rs` (new stage 9, conditional)

```rust
/// Stage 9 (optional, conditional on declared model): Conformance Check
/// 
/// If receipt carries a reference to an expected Petri net model,
/// check alignment-based conformance.
///
/// Returns fitness metric and list of non-conforming transitions.
fn stage_conformance_check(receipt: &Receipt, model: &PetriNet) -> CheckOutcome;
```

**Logic:**
```
1. Receipt must carry model reference (optional field added)
2. Convert receipt → OCEL
3. Run wasm4pm::conformance::alignment_fitness(&net, &log)
4. Return fitness ∈ [0, 1], detail = "fitness X, cost Y"
5. Outcome: "conformance_check" with fitness detail
```

**Verdict impact:** Fitness < 0.8 → reason includes "low fitness: {fitness}"

### 4. Predictive Monitoring in OTel Spans (NEW)
**File:** `src/tracing.rs` (feature-gated: `otel` + `predictive`)

```rust
/// Emit predictive-monitoring spans during receipt verification.
///
/// For each event in the receipt, predict the next activity from the
/// prefix and emit a span with:
/// - event_id: source event
/// - next_activity: predicted activity name
/// - confidence: probability (0–1)
/// - distance: number of predicted steps
pub fn trace_predictive_next_activity(
    receipt: &Receipt,
    prefix_up_to: usize,
) -> Result<Vec<PredictiveSpan>>;
```

**OTel attributes:**
```
span_name = "affidavit.predictive.next_activity"
event_id = "emit-0"
next_activity = "receipt_assembled"
confidence = 0.87
distance = 1  // immediately next
```

---

## Witness Tests (One Per Surface)

### Test 1: Discovery Witness (`tests/wasm4pm_discovery_witness.rs`)
**Gate:** Feature `discovery`

```rust
#[test]
fn discovery_linearizes_simple_receipt() {
    // Given a 3-event receipt (emit → assemble → verify)
    // When we convert to OCEL and mine with HIM
    // Then we get a Petri net with linear flow (path: t0→t1→t2)
    
    let receipt = build_simple_receipt(vec![
        ("emit", [("order", "Order")]),
        ("assemble", [("order", "Order")]),
        ("verify", [("order", "Order")]),
    ]);
    
    let ocel = receipt_to_ocel(&receipt).unwrap();
    let net = wasm4pm::mining::heuristic_inductive_miner(&ocel).unwrap();
    
    // Assert net structure
    assert_eq!(net.transitions.len(), 3);
    assert_eq!(net.places.len(), 4);  // typical: 1 source + 3 activities + 1 sink
    // Verify path is linear (no concurrency)
    assert_path_is_linear(&net, &["emit", "assemble", "verify"]);
}
```

**Witness claim:** ✅ Discovery mines the true process from receipt traces without oracle.

---

### Test 2: Conformance Witness (`tests/wasm4pm_conformance_witness.rs`)
**Gate:** Feature `conformance` (requires feature `discovery` + model ref)

```rust
#[test]
fn conformance_accepts_lawful_receipt_rejects_violated() {
    // Given a declared Petri net model (simple: emit→assemble→verify)
    // And a lawful receipt that follows the model exactly
    // When we check alignment-based conformance
    // Then fitness = 1.0
    
    let model = build_expected_model(vec![
        Transition::new("emit"),
        Transition::new("assemble"),
        Transition::new("verify"),
    ]);
    
    let lawful_receipt = build_receipt(vec![
        ("emit", [("order", "Order")]),
        ("assemble", [("order", "Order")]),
        ("verify", [("order", "Order")]),
    ]);
    
    let ocel = receipt_to_ocel(&lawful_receipt).unwrap();
    let fitness = wasm4pm::conformance::alignment_fitness(&model, &ocel).unwrap();
    
    assert_eq!(fitness, 1.0);  // Perfect conformance
    
    // Now violate: emit→verify (skip assemble)
    let violated_receipt = build_receipt(vec![
        ("emit", [("order", "Order")]),
        ("verify", [("order", "Order")]),  // Missing assemble!
    ]);
    
    let ocel_violated = receipt_to_ocel(&violated_receipt).unwrap();
    let fitness_violated = wasm4pm::conformance::alignment_fitness(&model, &ocel_violated).unwrap();
    
    assert!(fitness_violated < 0.8);  // Non-conforming
}
```

**Witness claim:** ✅ Conformance checking detects when receipt violates the declared process model.

---

### Test 3: Predictive Monitoring Witness (`tests/wasm4pm_predictive_witness.rs`)
**Gate:** Feature `otel` + feature `predictive`

```rust
#[test]
fn predictive_next_activity_from_prefix_trace() {
    // Given a receipt with events: emit → assemble → verify → show
    // When we predict the next activity after emit
    // Then the model predicts "assemble" with high confidence
    
    let receipt = build_receipt(vec![
        ("emit", [("order", "Order")]),
        ("assemble", [("order", "Order")]),
        ("verify", [("order", "Order")]),
        ("show", [("order", "Order")]),
    ]);
    
    let ocel = receipt_to_ocel(&receipt).unwrap();
    let model = wasm4pm::mining::heuristic_inductive_miner(&ocel).unwrap();
    
    // Predict next activity after the first event (emit)
    let prefix = ocel.events[0..1].to_vec();  // Just "emit"
    let next = wasm4pm::predictive::next_activity(&model, &prefix).unwrap();
    
    assert_eq!(next.activity, "assemble");
    assert!(next.confidence > 0.85);  // High confidence in linear flow
}
```

**Witness claim:** ✅ Predictive monitoring correctly forecasts next activity from partial receipt traces.

---

## Integration Roadmap

### Phase 2.1: Discovery (Stable Rust feasible)
**Timeline:** 1–2 days
**Deliverables:**
1. `src/mining.rs` — Receipt → OCEL conversion
2. `tests/wasm4pm_discovery_witness.rs` — Witness test
3. CLI feature: `affi receipt mine [receipt.json]` (outputs Petri net JSON)
4. `src/verifier.rs` stage 8 — optional discovery outcome

**Dependencies:** wasm4pm (already available, workspace)

**Gate:** Feature flag `discovery` (default: off)

### Phase 2.2: Conformance (Requires model ref in Receipt)
**Timeline:** 1–2 days
**Deliverables:**
1. Receipt struct: add optional `expected_model` field (or separate `.model.json` file)
2. `src/verifier.rs` stage 9 — conformance check (conditional)
3. `tests/wasm4pm_conformance_witness.rs` — Witness test
4. CLI feature: `affi receipt conform [receipt.json] [model.json]`

**Dependencies:** wasm4pm conformance module

**Gate:** Feature flag `conformance`

### Phase 2.3: Predictive Monitoring (OTel-only)
**Timeline:** 1–2 days
**Deliverables:**
1. `src/tracing.rs` extension — `trace_predictive_next_activity()`
2. `tests/wasm4pm_predictive_witness.rs` — Witness test
3. Span emissions during `verify()`
4. CLI: `affi receipt verify --otel --predictive receipt.json` (emits spans)

**Dependencies:** wasm4pm predictive module, opentelemetry (feature: otel)

**Gate:** Feature flags `otel` + `predictive`

---

## Feature Flags (Updated Cargo.toml)

```toml
[features]
default = []
otel = []
discovery = ["wasm4pm/cloud"]        # Heuristic Inductive Miner
conformance = ["discovery"]           # Conformance checking (requires discovery)
predictive = ["otel", "discovery"]    # Next-activity prediction (requires OTel + discovery)

# All-in-one for testing
all-mining = ["discovery", "conformance", "predictive"]
```

---

## CLI Surface (Extensions to Current)

### Current (unchanged)
```bash
affi receipt emit --type <TYPE> <objects>... [payload]
affi receipt assemble [--out FILE]
affi receipt verify <receipt.json>
affi receipt show <receipt.json>
```

### New (Feature-gated)

#### Discovery
```bash
# Requires: feature discovery
affi receipt mine <receipt.json>
  Output: Petri net in JSON (wasm4pm format)
  
# Or: included in verify
affi receipt verify --mine <receipt.json>
  Output: Verdict + discovered process (stage 8)
```

#### Conformance
```bash
# Requires: feature conformance
affi receipt conform <receipt.json> <expected_model.json>
  Output: Fitness score, violated transitions, cost matrix
```

#### Predictive
```bash
# Requires: features otel + predictive
affi receipt verify --otel --predict <receipt.json>
  Output: Verdict + OTel trace with next-activity predictions
```

---

## Risk Mitigation

### Risk: nightly-only (wasm4pm-compat requires nightly)
**Mitigation:** Feature flags default to off; stable Rust users get Phase 1 only (emit/assemble/verify/show). wasm4pm integration is strictly opt-in.

### Risk: Large receipt logs slow discovery
**Mitigation:** HIM includes noise filtering; add configurable sample-size gate (`--sample 1000`). Skip discovery for logs > N events.

### Risk: Model reference encoding (where does expected model come from?)
**Mitigation:** Two approaches:
1. Separate `.model.json` file (user-provided)
2. Receipt field `expected_model: Option<PetriNet>` (future; requires Evidence typestate)

Start with (1); Phase 2.2 can upgrade to (2).

### Risk: Conformance verdict is binary; doesn't explain root cause
**Mitigation:** Return cost matrix and list of violated transitions; emit as OTel attributes so operators can inspect causality.

---

## Testing Strategy

| Test | Phase | Gate | Assertions |
|------|-------|------|-----------|
| discovery_linearizes_simple_receipt | 2.1 | `discovery` | Net has 3 transitions, linear path |
| discovery_handles_concurrent_objects | 2.1 | `discovery` | Net has places; handles ≥2 concurrent objects |
| conformance_accepts_lawful_receipt | 2.2 | `conformance` | Fitness = 1.0 |
| conformance_rejects_violated_flow | 2.2 | `conformance` | Fitness < 0.8, reason names violated transition |
| predictive_next_activity_from_prefix | 2.3 | `predictive` | Confidence > 0.85, activity matches expected |
| predictive_otel_spans_emitted | 2.3 | `predictive` + `otel` | Span count = prefix length, attributes populated |

---

## File Changes (Summary)

### New Files
- `src/mining.rs` — Receipt ↔ OCEL conversion, discovery engine integration
- `tests/wasm4pm_discovery_witness.rs` — Discovery witness
- `tests/wasm4pm_conformance_witness.rs` — Conformance witness
- `tests/wasm4pm_predictive_witness.rs` — Predictive witness

### Modified Files
- `Cargo.toml` — Add wasm4pm dependency, feature flags
- `src/lib.rs` — Export `mining` module (feature-gated)
- `src/verifier.rs` — Add stages 8 (discover) and 9 (conform) (feature-gated)
- `src/tracing.rs` — Add `trace_predictive_next_activity()` (feature-gated)
- `src/types.rs` — Optional: add `expected_model` field to Receipt (Phase 2.2)
- `INTEGRATIONS.md` — Update Phase 2 status

---

## Success Criteria

✅ **Discovery:** Receipt trace → Petri net, lawful process structure certified without oracle  
✅ **Conformance:** Receipt trace vs declared model, fitness metric accurate  
✅ **Predictive:** Prefix trace → next-activity forecast, OTel spans correctly labeled  
✅ **Tests:** 3 witness tests, each proving one surface, all passing  
✅ **Features:** Stable Rust unaffected; Phase 1 still works; Phase 2 opt-in  

---

## References

- **Affidavit Phase 1:** `~/affidavit/STATUS.md` (complete; emit/assemble/verify/show)
- **wasm4pm-compat:** `~/wasm4pm-compat/src/lib.rs` (structure-only, nightly)
- **wasm4pm:** `~/wasm4pm/wasm4pm/src/` (discovery, conformance, predictive engines)
- **Chicago TDD:** `~/.claude/rules/process-mining-chicago-tdd.md` (process law doctrine)
- **Current tests:** `~/affidavit/tests/*.rs` (41 passing; Phase 1 complete)

---

**Phase 1 Status:** ✅ Complete (receipt sealing, deterministic chain, 7-stage verifier)  
**Phase 2 Planning:** ✅ Complete (this document)  
**Phase 2 Start:** Ready on signal
