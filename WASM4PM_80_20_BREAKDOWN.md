# wasm4pm Integration: The 80/20 Split Detailed

**Goal:** Identify exactly what wasm4pm provides off-shelf and what Affidavit must glue.

---

## Part 1: The 80% (What wasm4pm Does For Free)

### Module 1: Discovery Engines

#### 1.1 Heuristic Inductive Miner (HIM)
**Location:** `wasm4pm::mining::heuristic_inductive_miner()`
**Input:** `wasm4pm_compat::ocel::EventLog`
**Output:** `wasm4pm_compat::petri::PetriNet`
**What it does:**
- Reads OCEL events (activity names, objects, timestamps)
- Discovers concurrency and loops via heuristics
- Produces Petri net: places, transitions, input/output arcs
- Noise-tolerant (configurable threshold)

**Why affidavit needs it:**
- Prove that the receipt's actual trace matches some lawful process
- Not just structurally sound (7-stage pipeline) but behaviorally sound
- Serve as oracle for conformance checks

**Example:**
```
Input receipt events:
  emit: [order:Order]
  assemble: [order:Order]
  verify: [order:Order]
  show: [order:Order]

Output Petri net:
  Transitions: {emit, assemble, verify, show}
  Places: {p_start, p1, p2, p3, p_end}
  Flow: start → emit → p1 → assemble → p2 → verify → p3 → show → end
```

---

#### 1.2 Alpha Miner
**Location:** `wasm4pm::mining::alpha_miner()`
**Input:** `EventLog`
**Output:** `PetriNet`
**What it does:**
- Deterministic discovery (no noise)
- Produces WF-net (workflow net) subset
- Strict: only handles strictly sequential logs

**Why affidavit might use it:**
- For receipts with guaranteed linear order (assemble always after emit)
- As a sanity check: "is this receipt strictly sequential?"

---

#### 1.3 Inductive Miner
**Location:** `wasm4pm::mining::inductive_miner()`
**Input:** `EventLog`
**Output:** `ProcessTree` or `PetriNet`
**What it does:**
- Recursive decomposition
- Handles complex models with loops and concurrency
- Produces most expressive (and readable) models

---

### Module 2: Conformance Checking

#### 2.1 Alignment-Based Fitness
**Location:** `wasm4pm::conformance::alignment_fitness()`
**Input:** `PetriNet` (model), `EventLog` (trace)
**Output:** 
```rust
struct AlignmentFitness {
    fitness: f64,           // 0.0 to 1.0
    cost: u32,              // alignment cost
    violated_transitions: Vec<String>,
    cost_matrix: HashMap<String, u32>,
}
```
**What it does:**
- Computes optimal sequence alignment between model and log
- Finds minimum "cost" to replay log on model
- Cost = missing/wrong transitions, substitutions
- Fitness = 1 - (cost / max_cost)

**Why affidavit needs it:**
- Verify that receipt activities match the declared process model
- Provide a metric (0–1) for "how lawful" the receipt is
- List which transitions were violated (for audit)

**Example:**
```
Model: emit → assemble → verify → show

Receipt trace: emit → verify → show
(missing assemble)

Alignment cost: 1 substitution (assemble skipped)
Fitness: 0.75 (one out of four steps violated)
Violated: [assemble]
```

---

#### 2.2 Replay-Based Checking
**Location:** `wasm4pm::conformance::replay_fitness()`
**Input:** `PetriNet`, `EventLog`
**Output:** `bool` (trace can be perfectly replayed on net)
**What it does:**
- Strict boolean: does the log follow the model exactly?
- No partial credit; used for binary accept/reject

**Why affidavit might use it:**
- As a hard gate: "is this receipt 100% lawful?"
- In conformance_check stage, after alignment fitness

---

### Module 3: Predictive Monitoring

#### 3.1 Next-Activity Prediction
**Location:** `wasm4pm::predictive::next_activity()`
**Input:** `PetriNet` (model), `Trace` (prefix of events), `depth: usize`
**Output:**
```rust
struct NextActivityPrediction {
    activity: String,           // predicted next activity
    confidence: f64,            // 0.0 to 1.0
    distance: usize,            // steps to reach (1 = immediate)
    top_k: Vec<(String, f64)>,  // alternatives
}
```
**What it does:**
- Runs the model forward from current state
- Determines which transitions are enabled
- Returns probability distribution over enabled transitions
- Ranks by likelihood (enables, loop probability)

**Why affidavit needs it:**
- Forecast next event before receipt is finalized
- OTel tracing: emit span "next_activity = assemble" with confidence
- Anomaly detection: "was the actual next activity the predicted one?"

**Example:**
```
Model: emit → assemble → verify → show

Receipt prefix: emit, assemble

Next activity prediction:
  activity: "verify"
  confidence: 0.98
  distance: 1
  top_k: [("verify", 0.98), ("assemble", 0.02)]
```

---

#### 3.2 Remaining-Time / Event-Count Prediction
**Location:** `wasm4pm::predictive::remaining_time()` (optional, LSTM-ready)
**Input:** Trace prefix, model
**Output:** Estimated remaining events / wall-clock duration
**What it does:**
- Estimates how many more steps until completion
- Regression model (can be LSTM or Markov)

**Why affidavit might use it:**
- Predict "receipt assembly will take ~3 more events"
- SLO monitoring: "this receipt is taking longer than expected"

---

### Module 4: Format Converters (OCEL I/O)

#### 4.1 OCEL Serialization
**Location:** `wasm4pm_compat::ocel::*`
**What it does:**
- `EventLog::to_json()` → JSON
- `EventLog::to_xml()` → XES (XML Event Stream)
- `EventLog::from_json()` → parse back

**Why affidavit needs it:**
- Round-trip: Receipt → OCEL → JSON (for debugging)
- Exchange discovered models with other systems
- Web/REST APIs: OCEL is standard format

---

## Part 2: The 20% (What Affidavit Must Glue)

### Glue 1: Receipt ↔ OCEL Conversion

**File:** `src/mining.rs` (NEW)

**What affidavit must do:**

```rust
use wasm4pm_compat::ocel::{Event, EventLog, Trace};
use wasm4pm_compat::prelude::*;

/// Convert a Receipt's operation-events into an OCEL event log.
pub fn receipt_to_ocel(receipt: &Receipt) -> Result<EventLog> {
    let mut trace_map: HashMap<String, Vec<Event>> = HashMap::new();
    
    // Group events by primary object (order, invoice, etc.)
    for op_event in &receipt.events {
        let primary_object = op_event.objects.get(0)
            .ok_or_else(|| anyhow!("event has no objects"))?;
        
        let trace_key = &primary_object.id;
        
        let event = Event::new(op_event.event_type.clone())
            .with_timestamp(op_event.seq as u64)  // logical timestamp
            .with_attribute("payload_commitment", op_event.payload_commitment.as_hex().to_string())
            .with_attribute("event_id", op_event.id.clone());
        
        // Add objects as OCEL object references
        for obj_ref in &op_event.objects {
            event.add_object(
                obj_ref.id.clone(),
                obj_ref.obj_type.clone(),
                obj_ref.qualifier.clone(),
            );
        }
        
        trace_map.entry(trace_key.clone())
            .or_insert_with(Vec::new)
            .push(event);
    }
    
    // Convert map to traces, then traces to log
    let traces: Vec<Trace> = trace_map
        .into_values()
        .map(|events| Trace::from_events(events))
        .collect();
    
    Ok(EventLog::from_traces(traces))
}

/// Inverse: OCEL log back to receipt structure (for round-trip testing)
pub fn ocel_to_receipt(log: &EventLog) -> Result<Receipt> {
    // ...
}
```

**Why this is 20%:**
- Simple mapping: OperationEvent → Event, ObjectRef → objects
- No algorithm; just type wrapping
- ~60 lines of glue code

---

### Glue 2: Verifier Stage 8 & 9 Integration

**File:** `src/verifier.rs` (MODIFY)

**Current pipeline (7 stages):**
```
1. decode
2. check_format
3. chain_integrity
4. continuity
5. verify_commitments
6. evaluate_profile
7. emit_verdict
```

**Add (feature-gated):**
```rust
#[cfg(feature = "discovery")]
fn stage_discover_process(receipt: &Receipt) -> CheckOutcome {
    use crate::mining::receipt_to_ocel;
    use wasm4pm::mining;
    
    match receipt_to_ocel(receipt) {
        Ok(log) => {
            match mining::heuristic_inductive_miner(&log) {
                Ok(net) => {
                    let detail = format!(
                        "discovered {} transitions, {} places",
                        net.transitions.len(),
                        net.places.len()
                    );
                    CheckOutcome {
                        stage: "discover_process".to_string(),
                        passed: true,
                        detail,
                    }
                }
                Err(e) => CheckOutcome {
                    stage: "discover_process".to_string(),
                    passed: false,
                    detail: format!("mining failed: {e}"),
                }
            }
        }
        Err(e) => CheckOutcome {
            stage: "discover_process".to_string(),
            passed: false,
            detail: format!("ocel conversion failed: {e}"),
        }
    }
}

#[cfg(feature = "conformance")]
fn stage_conformance_check(receipt: &Receipt, expected_model: &PetriNet) -> CheckOutcome {
    use crate::mining::receipt_to_ocel;
    use wasm4pm::conformance;
    
    match receipt_to_ocel(receipt) {
        Ok(log) => {
            match conformance::alignment_fitness(expected_model, &log) {
                Ok(fitness_result) => {
                    let passed = fitness_result.fitness >= 0.8;
                    let detail = format!(
                        "fitness {:.2}, cost {}",
                        fitness_result.fitness, fitness_result.cost
                    );
                    CheckOutcome {
                        stage: "conformance_check".to_string(),
                        passed,
                        detail,
                    }
                }
                Err(e) => CheckOutcome {
                    stage: "conformance_check".to_string(),
                    passed: false,
                    detail: format!("conformance check failed: {e}"),
                }
            }
        }
        Err(e) => CheckOutcome {
            stage: "conformance_check".to_string(),
            passed: false,
            detail: format!("ocel conversion failed: {e}"),
        }
    }
}
```

**Why this is 20%:**
- Call wasm4pm APIs; handle errors
- ~40 lines per stage
- No discovery/conformance logic; it's in wasm4pm

---

### Glue 3: OTel Tracing Integration

**File:** `src/tracing.rs` (MODIFY)

**Current:**
```rust
#[cfg(feature = "otel")]
pub fn trace_emit(op: &str, seq: u64, f: impl FnOnce() -> R) -> R {
    // Emit a span with operation name and sequence
}

#[cfg(not(feature = "otel"))]
pub fn trace_emit(op: &str, seq: u64, f: impl FnOnce() -> R) -> R {
    // No-op
}
```

**Add (feature-gated: `otel` + `predictive`):**
```rust
#[cfg(all(feature = "otel", feature = "predictive"))]
pub fn trace_predictive_next_activity(receipt: &Receipt) -> Result<()> {
    use crate::mining::receipt_to_ocel;
    use wasm4pm::mining;
    use wasm4pm::predictive;
    
    let log = receipt_to_ocel(receipt)?;
    let net = mining::heuristic_inductive_miner(&log)?;
    
    // For each event, predict the next one
    for (i, event) in receipt.events.iter().enumerate() {
        let prefix_trace = &log.events[0..=i];
        let next = predictive::next_activity(&net, prefix_trace, 1)?;
        
        // Emit OTel span
        let tracer = opentelemetry::global::tracer("affidavit.predictive");
        let mut span = tracer.start(format!(
            "affidavit.predictive.next_activity[{}]",
            event.id
        ));
        
        span.set_attribute("event_id", event.id.clone());
        span.set_attribute("activity", next.activity.clone());
        span.set_attribute("confidence", next.confidence);
        span.set_attribute("distance", next.distance as i64);
        
        // Optional: set tag for actual vs predicted
        if i + 1 < receipt.events.len() {
            let actual_next = &receipt.events[i + 1];
            span.set_attribute("actual_next", actual_next.event_type.clone());
            span.set_attribute("matched", next.activity == actual_next.event_type);
        }
        
        drop(span);  // End span
    }
    
    Ok(())
}
```

**Why this is 20%:**
- Hook wasm4pm predictive output into OTel
- ~50 lines
- No prediction algorithm; all in wasm4pm

---

### Glue 4: CLI Dispatch (Feature-Gated)

**File:** `src/verbs/` (NEW optional verbs, generated by ggen if we extend ontology)

**Current:**
- `emit.rs` — calls `cli::emit()`
- `assemble.rs` — calls `cli::assemble()`
- `verify.rs` — calls `cli::verify()`
- `show.rs` — calls `cli::show()`

**Add (feature-gated):**
```
mine.rs      (feature: discovery)
conform.rs   (feature: conformance)
predict.rs   (feature: predictive)
```

**Example (`mine.rs`):**
```rust
use affidavit::cli;

pub fn mine(receipt_path: &str, output: Option<&str>) -> anyhow::Result<()> {
    let receipt = affidavit::chain::deserialize_receipt(receipt_path)?;
    let ocel = affidavit::mining::receipt_to_ocel(&receipt)?;
    
    #[cfg(feature = "discovery")]
    {
        use wasm4pm::mining;
        let net = mining::heuristic_inductive_miner(&ocel)?;
        let json = serde_json::to_string_pretty(&net)?;
        
        if let Some(out) = output {
            std::fs::write(out, json)?;
        } else {
            println!("{}", json);
        }
    }
    
    #[cfg(not(feature = "discovery"))]
    {
        return Err(anyhow!("discovery feature not enabled"));
    }
    
    Ok(())
}
```

**Why this is 20%:**
- Thin CLI wrapper
- All logic in verifier stages or mining module
- ~30 lines per verb

---

## Summary: 80/20 Distribution

| Component | Responsibility | Lines of Code | Source |
|-----------|-----------------|---------------|--------|
| **80% (wasm4pm)** | | | |
| Heuristic Inductive Miner | Discover model from trace | 2000+ | wasm4pm |
| Alpha Miner | Strict discovery | 1500+ | wasm4pm |
| Inductive Miner | Recursive discovery | 2000+ | wasm4pm |
| Alignment Fitness | Conformance scoring | 1500+ | wasm4pm |
| Replay Checking | Binary conformance | 800+ | wasm4pm |
| Next-Activity Prediction | Forecastng | 1000+ | wasm4pm |
| OCEL Serialization | Format I/O | 500+ | wasm4pm-compat |
| **Total 80%** | **~11,300 lines** | | |
| | | | |
| **20% (affidavit)** | | | |
| Receipt → OCEL glue | Type mapping | 60 | mining.rs |
| Stage 8 (discover) | Verifier hook | 40 | verifier.rs |
| Stage 9 (conform) | Verifier hook | 40 | verifier.rs |
| OTel predictive | Tracing hook | 50 | tracing.rs |
| CLI verbs (mine/conform/predict) | Dispatch | 90 | verbs/*.rs |
| Cargo.toml features | Gating logic | 10 | Cargo.toml |
| **Total 20%** | **~290 lines** | | |

---

## What Affidavit Does NOT Inherit

- ❌ Replay engine (too complex; wasm4pm handles it)
- ❌ Advanced conformance (we use alignment, not custom cost functions)
- ❌ Visualization (JSON only; consumers visualize)
- ❌ Multi-process mining (single trace per receipt)
- ❌ Time prediction (remains optional)

---

## Data Flow: Receipt → Discovered Model → Conformance Check

```
┌──────────────┐
│   Receipt    │
│  JSON        │
│ (sealed)     │
└──────┬───────┘
       │
       │ receipt_to_ocel()
       ▼
┌──────────────────────┐
│   OCEL EventLog      │  (20% glue)
│ {events, objects}    │
└──────┬───────────────┘
       │
       │ heuristic_inductive_miner()
       ▼
┌──────────────────┐                 (80% wasm4pm)
│   Petri Net      │
│ {places,         │
│  transitions,    │
│  flow arcs}      │
└──────┬───────────┘
       │
       ├─────────────┬──────────────────┐
       │             │                  │
       │ alignment   │ next_activity    │ stage 8
       │ fitness()   │ predict()        │ (discover)
       │             │                  │
       ▼             ▼                  ▼
   Fitness      Prediction         CheckOutcome
   (0–1)        (activity,          (discovered)
                confidence)
       │             │                  │
       └─────────────┴──────────────────┘
                     │
                     ▼
         ┌──────────────────────┐
         │  Verdict.outcomes[]  │  (stage 9:
         │  [{discover,         │   conform)
         │    fitness, ...}]    │
         └──────────────────────┘
```

---

## Concrete Example: 3-Event Receipt

**Receipt (emit → assemble → verify):**
```json
{
  "format_version": "v1",
  "events": [
    {
      "id": "emit-0",
      "seq": 0,
      "event_type": "emit",
      "objects": [{"id": "order-1", "obj_type": "Order"}],
      "payload_commitment": "abcd1234..."
    },
    {
      "id": "assemble-1",
      "seq": 1,
      "event_type": "assemble",
      "objects": [{"id": "order-1", "obj_type": "Order"}],
      "payload_commitment": "efgh5678..."
    },
    {
      "id": "verify-2",
      "seq": 2,
      "event_type": "verify",
      "objects": [{"id": "order-1", "obj_type": "Order"}],
      "payload_commitment": "ijkl9012..."
    }
  ],
  "chain_hash": "..."
}
```

**Stage 8 (discover):**
```
1. receipt_to_ocel() → EventLog {
     events: [
       Event(emit, ts=0, objects=[order-1]),
       Event(assemble, ts=1, objects=[order-1]),
       Event(verify, ts=2, objects=[order-1])
     ]
   }

2. heuristic_inductive_miner() → PetriNet {
     transitions: [emit, assemble, verify],
     places: [p_start, p1, p2, p_end],
     flow: [
       (p_start, emit, p1),
       (p1, assemble, p2),
       (p2, verify, p_end)
     ]
   }

3. CheckOutcome {
     stage: "discover_process",
     passed: true,
     detail: "discovered 3 transitions, 4 places"
   }
```

**Stage 9 (conform, if model provided):**
```
1. expected_model = PetriNet {
     transitions: [emit, assemble, verify],
     places: [p_start, p1, p2, p_end],
     flow: [...]
   }

2. alignment_fitness(expected_model, ocel_log) → {
     fitness: 1.0,
     cost: 0,
     violated_transitions: [],
     cost_matrix: {}
   }

3. CheckOutcome {
     stage: "conformance_check",
     passed: true,
     detail: "fitness 1.00, cost 0"
   }
```

**Final Verdict:**
```
Verdict {
  accepted: true,
  profile: CoreV1,
  outcomes: [
    {stage: "decode", passed: true, detail: "3 event(s), format_version present"},
    {stage: "check_format", passed: true, detail: "format_version == v1"},
    {stage: "chain_integrity", passed: true, detail: "recomputed chain hash matches"},
    {stage: "continuity", passed: true, detail: "seq 0,1,2; no gaps; ids unique"},
    {stage: "verify_commitments", passed: true, detail: "all 3 commitments well-formed"},
    {stage: "evaluate_profile", passed: true, detail: "all activities present"},
    {stage: "discover_process", passed: true, detail: "discovered 3 transitions, 4 places"},
    {stage: "conformance_check", passed: true, detail: "fitness 1.00, cost 0"}
  ],
  reason: "all stages passed"
}
```

---

## Maintenance Burden (20% Side)

- **Update wasm4pm → Affidavit API changes:** O(1) — just call sites in mining.rs, tracing.rs
- **Add new discovery algorithm:** Change stage_discover_process() one-liner (swap `alpha_miner` for `inductive_miner`)
- **Extend OCEL mapping:** Add field to receipt_to_ocel() (~5 lines per new field)
- **OTel attribute changes:** Edit trace_predictive_next_activity() (~2 lines per attribute)

**Total maintenance:** ~2–3 hours per major wasm4pm API upgrade (rare; ~1/quarter)

---

**Conclusion:** Affidavit contributes ~290 lines of glue to wire wasm4pm (80%) into the verifier pipeline. The 80% is algorithmic depth (discovery, conformance, prediction); the 20% is orchestration and type bridging.
