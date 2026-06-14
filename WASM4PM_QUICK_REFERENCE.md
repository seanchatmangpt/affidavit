# wasm4pm Integration — Quick Reference Card

**TL;DR:** Wire wasm4pm (discovery, conformance, predictive) into affidavit verifier. 80% off-shelf engines, 20% glue code. 3 witness tests. Feature-gated; stable Rust unaffected.

---

## The Three Surfaces at a Glance

```
┌─ DISCOVERY ────────────────────────────┐
│ Input:  Receipt (operation-events)     │
│ Engine: Heuristic Inductive Miner      │
│ Output: Petri Net (places, arcs)       │
│ Proves: Process structure correct      │
│ Feature: discovery                     │
└────────────────────────────────────────┘

┌─ CONFORMANCE ──────────────────────────┐
│ Input:  Receipt + Expected Model       │
│ Engine: Alignment-based fitness        │
│ Output: Fitness ∈ [0, 1], violations   │
│ Proves: Receipt follows declared flow  │
│ Feature: conformance                   │
└────────────────────────────────────────┘

┌─ PREDICTIVE ───────────────────────────┐
│ Input:  Receipt prefix (first N events)│
│ Engine: Next-activity probability      │
│ Output: Next activity + confidence     │
│ Proves: Forecast matches actual        │
│ Features: otel + predictive            │
└────────────────────────────────────────┘
```

---

## Files to Create/Modify

### Create (NEW)
```
src/mining.rs                         60 lines (Receipt ↔ OCEL)
tests/wasm4pm_discovery_witness.rs    140 lines
tests/wasm4pm_conformance_witness.rs  140 lines
tests/wasm4pm_predictive_witness.rs   140 lines
```

### Modify
```
Cargo.toml                            + wasm4pm dep, features
src/lib.rs                            + pub mod mining
src/verifier.rs                       + stages 8, 9
src/tracing.rs                        + trace_predictive_next_activity()
src/types.rs                          + optional expected_model field
INTEGRATIONS.md                       + Phase 2 status
```

---

## Feature Flags (Cargo.toml)

```toml
[features]
default = []
discovery = ["wasm4pm/cloud"]
conformance = ["discovery"]
predictive = ["otel", "discovery"]
all-mining = ["discovery", "conformance", "predictive"]
```

**Stability:**
- Default (no flags) → Stable Rust, Phase 1 only
- `discovery` → Nightly (via wasm4pm-compat)
- `conformance` → Nightly (via discovery)
- `predictive` → Nightly (via discovery)

---

## Verifier Pipeline (Updated)

```
Stage 1: decode
Stage 2: check_format
Stage 3: chain_integrity
Stage 4: continuity
Stage 5: verify_commitments
Stage 6: evaluate_profile
Stage 7: emit_verdict
─────────────────────────────── (Phase 1 complete)
Stage 8: discover_process       (Feature: discovery)
Stage 9: conformance_check      (Feature: conformance, conditional)
─────────────────────────────── (Phase 2)
```

All stages feature-gated; default (no features) runs stages 1–7 only.

---

## CLI Surface (New Verbs)

```bash
# Current (unchanged)
affi receipt emit --type TYPE <objects>... [payload]
affi receipt assemble [--out FILE]
affi receipt verify <receipt.json>
affi receipt show <receipt.json>

# New (feature-gated)
affi receipt mine <receipt.json>                  # discovery
affi receipt conform <receipt.json> <model.json>  # conformance
affi receipt verify --otel --predict <receipt>    # predictive (option flag)
```

---

## The 20% Glue (Affidavit Code)

### 1. Receipt → OCEL (src/mining.rs)
```rust
pub fn receipt_to_ocel(receipt: &Receipt) -> Result<EventLog> {
    // Map OperationEvent → Event
    // Map ObjectRef → objects
    // Return EventLog (wasm4pm type)
}
```
**Why:** Type bridging; wasm4pm expects OCEL format.

### 2. Verifier Stages (src/verifier.rs)
```rust
#[cfg(feature = "discovery")]
fn stage_discover_process(receipt: &Receipt) -> CheckOutcome {
    let ocel = receipt_to_ocel(receipt)?;
    let net = wasm4pm::mining::heuristic_inductive_miner(&ocel)?;
    // Return CheckOutcome with net details
}

#[cfg(feature = "conformance")]
fn stage_conformance_check(receipt: &Receipt, model: &PetriNet) -> CheckOutcome {
    let ocel = receipt_to_ocel(receipt)?;
    let fitness = wasm4pm::conformance::alignment_fitness(model, &ocel)?;
    // Return CheckOutcome with fitness metric
}
```
**Why:** Hooks into verifier pipeline; call wasm4pm APIs.

### 3. OTel Tracing (src/tracing.rs)
```rust
#[cfg(all(feature = "otel", feature = "predictive"))]
pub fn trace_predictive_next_activity(receipt: &Receipt) -> Result<()> {
    let ocel = receipt_to_ocel(receipt)?;
    let net = wasm4pm::mining::heuristic_inductive_miner(&ocel)?;
    for event in &receipt.events {
        let next = wasm4pm::predictive::next_activity(&net, ...)?;
        // Emit OTel span with next activity + confidence
    }
}
```
**Why:** Carry wasm4pm predictions to OTel collector.

### 4. CLI Verbs (src/verbs/mine.rs, conform.rs, predict.rs)
```rust
pub fn mine(receipt_path: &str, output: Option<&str>) -> Result<()> {
    let receipt = deserialize_receipt(receipt_path)?;
    let ocel = receipt_to_ocel(&receipt)?;
    let net = wasm4pm::mining::heuristic_inductive_miner(&ocel)?;
    println!("{}", serde_json::to_string_pretty(&net)?);
}
```
**Why:** CLI entry points; thin wrappers.

---

## Witness Tests (One Per Surface)

### Discovery Witness
**File:** `tests/wasm4pm_discovery_witness.rs` (Feature: `discovery`)

```rust
#[test]
fn discovery_linearizes_simple_receipt() {
    // Given: 3-event receipt (emit→assemble→verify)
    // When:  Mine with HIM
    // Then:  Discover net with 3 transitions, linear path verified
    
    let receipt = build_simple_receipt(...);
    let ocel = receipt_to_ocel(&receipt)?;
    let net = heuristic_inductive_miner(&ocel)?;
    assert_eq!(net.transitions.len(), 3);
    assert_path_is_linear(&net, &["emit", "assemble", "verify"]);
}
```

**Claim:** ✅ Discovery mines true process from receipt without oracle.

### Conformance Witness
**File:** `tests/wasm4pm_conformance_witness.rs` (Feature: `conformance`)

```rust
#[test]
fn conformance_accepts_lawful_receipt() {
    // Given: Expected model (emit→assemble→verify) and lawful receipt
    // When:  Check fitness
    // Then:  Fitness = 1.0, cost = 0, no violations
    
    let model = build_expected_model(...);
    let receipt = build_lawful_receipt(...);
    let ocel = receipt_to_ocel(&receipt)?;
    let fitness = alignment_fitness(&model, &ocel)?;
    assert_eq!(fitness.fitness, 1.0);
}

#[test]
fn conformance_rejects_violated_flow() {
    // Given: Expected model and receipt that skips assemble
    // When:  Check fitness
    // Then:  Fitness < 0.8, assemble flagged as violated
    
    let model = build_expected_model(...);
    let receipt = build_violated_receipt(...);  // emit → verify (skip assemble)
    let ocel = receipt_to_ocel(&receipt)?;
    let fitness = alignment_fitness(&model, &ocel)?;
    assert!(fitness.fitness < 0.8);
}
```

**Claim:** ✅ Conformance detects lawful vs violated traces.

### Predictive Witness
**File:** `tests/wasm4pm_predictive_witness.rs` (Features: `predictive` + `otel`)

```rust
#[test]
fn predictive_next_activity_from_prefix() {
    // Given: Receipt with linear flow (emit→assemble→verify→show)
    // When:  Predict next activity after [emit]
    // Then:  Forecast is "assemble" with confidence > 0.85
    
    let receipt = build_receipt(...);
    let ocel = receipt_to_ocel(&receipt)?;
    let net = heuristic_inductive_miner(&ocel)?;
    let prefix = ocel.events[0..1].to_vec();
    let next = next_activity(&net, &prefix, 1)?;
    assert_eq!(next.activity, "assemble");
    assert!(next.confidence > 0.85);
}
```

**Claim:** ✅ Predictive forecasts align with actual next activity.

---

## Test Execution

```bash
# All mining tests (feature-gated)
cargo test --all-features --test "*witness*"

# By surface
cargo test --features discovery --test wasm4pm_discovery_witness
cargo test --features conformance --test wasm4pm_conformance_witness
cargo test --features predictive,otel --test wasm4pm_predictive_witness

# Verify stable Rust unaffected
cargo test  # No features; runs Phase 1 only (stages 1–7)

# Full suite
cargo test --all-features
```

---

## Chicago TDD Doctrine

> _If the code says it worked but the event log cannot prove a lawful process happened, then it did not work._

**Application:**
1. Code returns `Ok(net)` ← success path
2. Inspect event log (OCEL): can you replay net → trace?
3. If net does NOT match trace, test fails (witness violated)
4. Verdict: Code success + log proof = acceptance

---

## Decision Tree: Which Feature?

```
Do you want to mine a process from receipt?
├─ Yes → discovery
└─ No  → skip

Do you have a declared model to check against?
├─ Yes → conformance (requires discovery)
└─ No  → skip

Do you want next-activity forecasts in OTel?
├─ Yes → predictive (requires otel + discovery)
└─ No  → skip
```

---

## Quick Integration Checklist

- [ ] Create `src/mining.rs` with `receipt_to_ocel()`
- [ ] Add stage 8 & 9 to `src/verifier.rs` (feature-gated)
- [ ] Update `src/tracing.rs` with predictive hook
- [ ] Create 3 witness test files
- [ ] Add feature flags to `Cargo.toml`
- [ ] Update `INTEGRATIONS.md`
- [ ] Run `cargo test --all-features` (all pass)
- [ ] Run `cargo test` (stable Rust, Phase 1 only, all pass)
- [ ] Verify CLI verbs (`affi receipt mine`, `conform`, `predict`)

---

## Effort Estimate

| Phase | Component | Days | Blocker |
|-------|-----------|------|---------|
| 2.1 | Discovery | 1–2 | None (stable Rust feasible) |
| 2.2 | Conformance | 1–2 | Needs model ref mechanism |
| 2.3 | Predictive | 1–2 | OTel env setup |
| **Total** | | **3–6** | Feature flag gating |

---

## Risk Flags 🚩

| Risk | Mitigation |
|------|-----------|
| nightly-only | Feature flags off-by-default |
| Large receipts slow mining | Sample-size gate (--sample 1000) |
| Model reference undefined | Separate .model.json file (v1); Receipt field (future) |
| OTel overhead | Gate behind explicit feature |

---

## Success Metrics ✅

- All 3 witness tests pass
- Stable Rust unaffected (default build)
- Feature flags properly gate nightly code
- CLI verbs reachable and work
- INTEGRATIONS.md updated to Phase 2

---

**Status:** Planning ✅ → Implementation ⏭️  
**Documents:** 4 planning docs (this + 3 detailed)  
**Code ready:** Test templates provided  
**Next:** Signal to start Phase 2.1 (Discovery)
