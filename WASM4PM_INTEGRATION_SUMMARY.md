# wasm4pm Integration Plan — Executive Summary

**Project:** Affidavit v26.6.14 + wasm4pm Phase 2  
**Date:** 2026-06-14  
**Status:** Planning complete; ready for implementation  
**Effort:** ~3–4 days of development work  

---

## The Mission

Affidavit currently certifies receipts structurally (7-stage format verifier) but does not prove they follow a lawful process. **wasm4pm integration** adds process-mining dimension:

1. **Discover** what process actually happened (Heuristic Inductive Miner)
2. **Check conformance** against a declared model (alignment-based fitness)
3. **Predict next activity** during ongoing receipt assembly (LSTM-ready forecasting)

**Result:** One unified verifier that proves both *structural* integrity (BLAKE3 sealed) and *behavioral* conformance (process lawfulness).

---

## The Numbers

| Metric | Value |
|--------|-------|
| **wasm4pm LOC contributed** | ~11,300 lines (discovery, conformance, prediction engines) |
| **Affidavit glue LOC** | ~290 lines (type bridging, CLI dispatch, OTel hooks) |
| **Ratio** | 80% wasm4pm / 20% affidavit |
| **New test files** | 3 (discovery, conformance, predictive witness tests) |
| **Feature flags** | 4 (`discovery`, `conformance`, `predictive`, `all-mining`) |
| **New CLI verbs** | 3 (`mine`, `conform`, `predict`) |
| **Development timeline** | Phase 2.1 (discovery): 1–2 days; Phase 2.2 (conformance): 1–2 days; Phase 2.3 (predictive): 1–2 days |

---

## Key Deliverables

### Phase 2.1: Discovery (Stable Rust feasible)
```
src/mining.rs                              (60 lines) Receipt ↔ OCEL conversion
src/verifier.rs stage 8                    (40 lines) Discovery hook
tests/wasm4pm_discovery_witness.rs         (140 lines) Witness test
Cargo.toml feature flag: discovery
```
**Proves:** Receipt trace → Petri net mined correctly.

### Phase 2.2: Conformance (Requires model reference)
```
src/verifier.rs stage 9                    (40 lines) Conformance hook
tests/wasm4pm_conformance_witness.rs       (140 lines) Witness test
Cargo.toml feature flag: conformance
Receipt struct: optional expected_model field
```
**Proves:** Receipt fitness vs declared model accurate; violations detected.

### Phase 2.3: Predictive Monitoring (OTel-only)
```
src/tracing.rs extension                   (50 lines) Predictive spans
tests/wasm4pm_predictive_witness.rs        (140 lines) Witness test
Cargo.toml feature flags: predictive
```
**Proves:** Next-activity forecast correct; OTel spans emitted with high confidence.

---

## The 20% Glue (Affidavit's Contribution)

### 1. Receipt → OCEL Conversion
```rust
// Simple type mapping: OperationEvent → Event, ObjectRef → objects
pub fn receipt_to_ocel(receipt: &Receipt) -> Result<wasm4pm_compat::ocel::EventLog>
```
**Why:** wasm4pm accepts OCEL event logs; receipts use a different type structure.

### 2. Verifier Stage Integration
```rust
// Call wasm4pm APIs from within the certify pipeline
fn stage_discover_process(receipt: &Receipt) -> CheckOutcome { ... }
fn stage_conformance_check(receipt: &Receipt, model: &PetriNet) -> CheckOutcome { ... }
```
**Why:** Stages 8 & 9 are optional (feature-gated) and conditional (conformance needs a model).

### 3. OTel Tracing Hooks
```rust
// Emit predictive-monitoring spans during verification
pub fn trace_predictive_next_activity(receipt: &Receipt) -> Result<()> { ... }
```
**Why:** wasm4pm produces prediction results; affidavit carries them to OTel.

### 4. CLI Dispatch
```bash
affi receipt mine <receipt.json>              # Feature: discovery
affi receipt conform <receipt.json> <model>   # Feature: conformance
affi receipt predict <receipt.json>           # Features: predictive + otel
```
**Why:** User entry points; thin wrappers around verifier stages.

---

## Process Mining Doctrine (Chicago TDD)

> **_If the code says it worked but the event log cannot prove a lawful process happened, then it did not work._**

Application to Affidavit:
1. **Code path:** wasm4pm returns `Ok(net)` (success)
2. **Event log:** Receipt is converted to OCEL; mined process is introspected
3. **Proof:** Path tracing through net confirms it encodes receipt's activity sequence
4. **Verdict:** If net does NOT match receipt, test fails (even if code says success)

**Three witness tests prove one surface each:**
- Discovery witness: Mined net matches receipt activities
- Conformance witness: Fitness metric correctly reflects trace-model alignment
- Predictive witness: Forecast aligns with actual next activity

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| **nightly-only requirement** | Feature flags default to off; stable Rust users keep Phase 1 only |
| **Large receipts slow discovery** | Configurable sample-size gate; skip for logs > N events |
| **Model reference missing** | Two approaches: (1) separate `.model.json` (current); (2) Receipt field (Phase 2.2) |
| **Conformance verdict too binary** | Return cost matrix and violated transitions; emit as OTel attributes |
| **OTel span overload** | Gate predictive spans behind explicit feature flag + flag in CLI |

---

## Success Criteria

- ✅ **Discovery:** Receipt trace → Petri net, lawful process certified without oracle
- ✅ **Conformance:** Receipt vs model, fitness metric accurate, violations named
- ✅ **Predictive:** Prefix trace → next-activity forecast, confidence > 0.85
- ✅ **Tests:** 3 witness tests, each proving one surface, all passing
- ✅ **Features:** Stable Rust unaffected; Phase 1 intact; Phase 2 opt-in
- ✅ **Documentation:** Integration guide updated; witness claims auditable

---

## Integration Points (4 Files to Create/Modify)

### New Files
| File | Lines | Purpose |
|------|-------|---------|
| `src/mining.rs` | ~60 | Receipt ↔ OCEL glue |
| `tests/wasm4pm_discovery_witness.rs` | ~140 | Discovery witness |
| `tests/wasm4pm_conformance_witness.rs` | ~140 | Conformance witness |
| `tests/wasm4pm_predictive_witness.rs` | ~140 | Predictive witness |

### Modified Files
| File | Changes |
|------|---------|
| `Cargo.toml` | Add wasm4pm dep, feature flags (discovery, conformance, predictive) |
| `src/lib.rs` | Export `pub mod mining` (feature-gated) |
| `src/verifier.rs` | Add stages 8 & 9 (feature-gated) |
| `src/tracing.rs` | Add `trace_predictive_next_activity()` (feature-gated) |
| `src/types.rs` | Optional: add `expected_model: Option<PetriNet>` to Receipt |
| `INTEGRATIONS.md` | Update Phase 2 status |

---

## Documentation Created for Planning

1. **WASM4PM_INTEGRATION_PLAN.md** (this folder)
   - Full roadmap with feature flags, CLI surface, risk mitigation
   - 300+ lines; comprehensive

2. **WASM4PM_80_20_BREAKDOWN.md** (this folder)
   - Detailed breakdown of off-shelf wasm4pm vs affidavit glue
   - Data flow diagrams, concrete examples
   - 500+ lines; tactical

3. **WASM4PM_WITNESS_TEST_TEMPLATES.md** (this folder)
   - Runnable test stubs for all three surfaces
   - Chicago TDD doctrine application
   - 450+ lines; code-ready

4. **WASM4PM_INTEGRATION_SUMMARY.md** (this document)
   - Executive summary and quick reference
   - 200+ lines; strategic

---

## Next Steps

### Immediate (Planning → Handoff)
1. ✅ Planning complete (you are here)
2. ⏭️ Code review of integration plan (optional; can start implementation)
3. ⏭️ Begin Phase 2.1 (Discovery) development

### Phase 2.1 (Discovery, Days 1–2)
```bash
# Create new module
touch src/mining.rs

# Implement receipt_to_ocel()
# Add stage_discover_process() to verifier.rs
# Create discovery witness test
# Feature flag: discovery

# Validate
cargo test --features discovery
```

### Phase 2.2 (Conformance, Days 3–4)
```bash
# Add Receipt field (optional) or separate model reference
# Add stage_conformance_check() to verifier.rs
# Create conformance witness test
# Feature flag: conformance

# Validate
cargo test --features conformance
```

### Phase 2.3 (Predictive, Days 5–6)
```bash
# Add trace_predictive_next_activity() to tracing.rs
# Create predictive witness test
# Feature flags: predictive + otel

# Validate
cargo test --features predictive,otel
```

### Phase 2 Completion
```bash
# All tests pass
cargo test --all-features

# Feature isolation verified
cargo test (default, no discovery/conformance/predictive)
cargo test --features discovery
cargo test --features conformance
cargo test --features predictive

# Documentation updated
# Bumps: v26.6.14 → v26.7.0 (Phase 2 complete)
```

---

## How to Use This Plan

### For Implementation
1. Start with **WASM4PM_80_20_BREAKDOWN.md** to understand the split
2. Use **WASM4PM_WITNESS_TEST_TEMPLATES.md** as code stubs
3. Refer to **WASM4PM_INTEGRATION_PLAN.md** for feature gates and CLI details

### For Code Review
1. Check each glue function against the breakdown (~290 LOC expected)
2. Verify witness tests are independent and run in isolation
3. Confirm feature flags gate nightly code properly

### For Acceptance
- ✅ All 3 witness tests pass
- ✅ Stable Rust unaffected (default features only)
- ✅ New features opt-in only
- ✅ Documentation updated (INTEGRATIONS.md)

---

## FAQ

**Q: Can I use this without wasm4pm?**  
A: Yes. Default features have no wasm4pm dependency. Phase 1 (emit/assemble/verify/show) is fully functional on stable Rust.

**Q: Do I need to declare a model to use conformance?**  
A: Yes, conformance checks against a declared Petri net. Phase 2.2 will support two forms: separate `.model.json` file or Receipt field.

**Q: What if my receipt is too large?**  
A: HIM has noise filtering. Discovery can be skipped for logs > 10K events; add a CLI flag (`--sample 1000`) to cap.

**Q: Is OTel required for predictive?**  
A: For monitoring yes (spans need a collector). Prediction itself is algorithms-only and works standalone. Gated behind `otel` + `predictive` features.

**Q: When is Phase 3?**  
A: Deferred. Phase 2 completes the integration. Phase 3 would be scaling, optimization, and compliance with Chicago TDD law suite.

---

## References

| Document | Purpose |
|----------|---------|
| STATUS.md | Phase 1 completion (current state) |
| WASM4PM_INTEGRATION_PLAN.md | Full roadmap (300+ lines) |
| WASM4PM_80_20_BREAKDOWN.md | Glue breakdown (500+ lines) |
| WASM4PM_WITNESS_TEST_TEMPLATES.md | Test stubs (450+ lines) |
| INTEGRATIONS.md | Library integration status (will be updated) |
| Cargo.toml | Current dependencies |

---

**Status:** ✅ Planning complete.  
**Confidence:** High (80/20 split well-defined; no algorithmic work needed from affidavit).  
**Next:** Ready for implementation on signal.

Questions? Refer to the detailed planning documents above.
