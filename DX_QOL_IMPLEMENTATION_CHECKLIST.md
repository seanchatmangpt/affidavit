# DX/QOL 1000x Implementation Checklist

**Document:** Implementation roadmap + per-feature checklist + dependency graph  
**Last Updated:** 2026-06-14  
**Status:** Planning → Ready for Phase 1 execution

---

## Quick Priority Matrix

| Priority | Features | Est. Hours | Dependencies | ROI |
|----------|----------|-----------|--------------|-----|
| **P0: Foundation** | inspect, diff, visualize | 6h | chicago-tdd (already integrated) | 🔥🔥🔥 |
| **P0: Foundation** | Criterion benchmarks | 4h | Criterion (already integrated) | 🔥🔥🔥 |
| **P1: Intelligence** | model, conform, predict | 10h | wasm4pm-compat (ready) | 🔥🔥 |
| **P1: Intelligence** | LSP hover, goto-def | 8h | lsp-max (ready) | 🔥 |
| **P2: Quality Assurance** | mutate, generate test | 8h | clnrm, chicago-tdd | 🔥🔥 |
| **P2: Quality Assurance** | Property-based, Fixture DB | 8h | quickcheck, serde | 🔥 |
| **P3: Observability** | OTel trace, metrics | 7h | OTel (already integrated) | 🔥 |
| **P3: Observability** | Baggage, events, SLO | 6h | OTel (already integrated) | 🔥 |
| **P4: UX** | Help, examples, aliases | 5h | ggen, clap-noun-verb | 🔥 |
| **P4: UX** | JSON output, shell REPL | 7h | serde, rustyline | 🔥 |

**Total:** 88 hours, 22 features, 10 dependency groups

---

## Phase 1: Receipt Inspection (Week 1)

### 1.1 Feature: `affi receipt inspect`

- [ ] **Ontology Extension** (20 min)
  - [ ] Add `Verb { name: "inspect", noun: "receipt" }` to `ontology/affi-cli.ttl`
  - [ ] Add help text from chicago-tdd docs
  - [ ] Run `ggen sync` to verify ASK passes

- [ ] **Handler Implementation** (40 min)
  - [ ] Create `src/handlers.rs::inspect(receipt: &Receipt) -> InspectionReport`
  - [ ] Calculate event_type distribution (e.g., {emit: 1, assemble: 1, verify: 1})
  - [ ] Calculate object_type coverage (e.g., {Invoice: 2, Agent: 1})
  - [ ] Compute chain hash + verify it matches
  - [ ] Return `InspectionReport { event_types: Map, object_types: Map, ... }`

- [ ] **CLI Dispatch** (30 min)
  - [ ] ggen renders `src/verbs/inspect.rs` (thin wrapper)
  - [ ] Wrapper calls `handlers::inspect()` and pretty-prints result

- [ ] **Test Coverage** (40 min)
  - [ ] `tests/e2e_inspection.rs::test_inspect_simple_3_event`
  - [ ] `tests/e2e_inspection.rs::test_inspect_branching_5_event`
  - [ ] Verify report structure (has event_types, object_types)

- [ ] **Documentation** (20 min)
  - [ ] Add example to `examples/inspection.sh` (auto-generated)
  - [ ] Update `FEATURES_DX_QOL.md` with implementation evidence

**Estimated Effort:** 2.5 hours  
**Dependencies:** chicago-tdd-tools (✅ integrated), ggen (✅ integrated)  
**Exit Criteria:** `affi receipt inspect` returns detailed report; 2 tests pass

---

### 1.2 Feature: `affi receipt diff`

- [ ] **Algorithm Design** (30 min)
  - [ ] Define DiffResult struct: added (events), removed (events), modified (events)
  - [ ] Compare two receipts by sequence: O(n) linear diff

- [ ] **Handler Implementation** (30 min)
  - [ ] Create `src/handlers.rs::diff_receipts(a: &Receipt, b: &Receipt) -> DiffResult`
  - [ ] Iterate through events in order, track additions/removals
  - [ ] For modified events: compare event_type, commitment, objects

- [ ] **Ontology Extension** (15 min)
  - [ ] Add `Verb { name: "diff" }` to affi-cli.ttl

- [ ] **CLI Dispatch & Test** (30 min)
  - [ ] Render verb + test with chicago-tdd fixture pair

**Estimated Effort:** 1.5 hours  
**Exit Criteria:** `affi receipt diff a.json b.json` shows additions/removals/modifications

---

### 1.3 Feature: `affi receipt visualize`

- [ ] **Graph Builder** (1.5h)
  - [ ] Create `src/graph_builder.rs::ReceiptGraph`
  - [ ] Model: nodes = events, edges = event_type → event_type (control flow)
  - [ ] Export to DOT (graphviz) and JSON formats

- [ ] **Handler & CLI** (30 min)
  - [ ] `handlers.rs::visualize(receipt, format) -> String`
  - [ ] Add `--format={dot,json}` flag to ontology verb

- [ ] **Test** (30 min)
  - [ ] Verify JSON has "nodes" and "edges" keys
  - [ ] Verify DOT is valid graphviz

**Estimated Effort:** 2.5 hours  
**Exit Criteria:** `affi receipt visualize --format=json receipt.json | jq .nodes` works

---

### 1.4 Feature: `affi receipt catalog`

- [ ] **Fixture Registry** (30 min)
  - [ ] List chicago-tdd-tools fixtures programmatically
  - [ ] Extract metadata (event_count, pattern_name, description)

- [ ] **Handler & Search** (40 min)
  - [ ] `handlers.rs::catalog(filter_by, filter_value) -> Vec<Fixture>`
  - [ ] Add CLI flags: `--by-size`, `--by-pattern`

- [ ] **Test** (20 min)
  - [ ] Search for "event_count=5" returns linear-5-event fixture

**Estimated Effort:** 1.5 hours

---

### 1.5 Feature: Shell Completion

- [ ] **Configuration** (30 min)
  - [ ] Add `completion = true` to ggen.toml verb rules
  - [ ] Update Cargo.toml: `[[bin]]; name = "affi-completion"; path = "src/bin/completion.rs"`

- [ ] **Implementation** (30 min)
  - [ ] `src/bin/completion.rs` uses clap-noun-verb to generate bash/zsh/fish

- [ ] **Test** (15 min)
  - [ ] Verify completion script contains all verb names and flags

**Estimated Effort:** 1 hour  
**Success:** `source <(affi --completion bash)` then `affi receipt <TAB>`

---

## Phase 2: Process Discovery (Week 2)

### 2.1 Feature: `affi receipt model` (Discovery)

- [ ] **OCEL Conversion** (1h)
  - [ ] Create `src/mining.rs::receipt_to_ocel(receipt: &Receipt) -> Result<EventLog>`
  - [ ] Map Receipt events → wasm4pm OCEL events
  - [ ] Group events by primary object (order, invoice, etc.)
  - [ ] Serialize to valid OCEL JSON schema

- [ ] **Mining Integration** (30 min)
  - [ ] Call `wasm4pm::mining::heuristic_inductive_miner(log) -> PetriNet`
  - [ ] Serialize PetriNet to JSON
  - [ ] Handle errors gracefully

- [ ] **Handler & CLI** (30 min)
  - [ ] `handlers.rs::model(receipt) -> PetriNet`
  - [ ] Add `--output=model.json` flag to ontology

- [ ] **Test** (1h)
  - [ ] Receipt (emit→assemble→verify→show) → model with 4 transitions + places
  - [ ] Verify model serialization is valid OCEL

- [ ] **Feature Gate** (15 min)
  - [ ] Add `#[cfg(feature = "discovery")]` guards
  - [ ] Update Cargo.toml: `discovery = ["wasm4pm-compat"]`

**Estimated Effort:** 3 hours  
**Dependencies:** wasm4pm-compat (✅ integrated)  
**Exit Criteria:** `affi receipt model --output=model.json receipt.json` produces valid Petri net

---

### 2.2 Feature: `affi receipt conform`

- [ ] **Model Loader** (30 min)
  - [ ] `src/mining.rs::load_model_from_file(path) -> PetriNet`
  - [ ] Parse JSON, validate against Petri net schema

- [ ] **Fitness Handler** (45 min)
  - [ ] `handlers.rs::conform(receipt, model) -> ConformanceResult`
  - [ ] Convert receipt → OCEL
  - [ ] Call `wasm4pm::conformance::alignment_fitness(model, log) -> Fitness`
  - [ ] Interpret fitness (0–1 scale) and violated transitions

- [ ] **CLI & Test** (30 min)
  - [ ] Add `--model=model.json` flag
  - [ ] Test: linear receipt conforms to linear model (fitness=1.0)

**Estimated Effort:** 1.5 hours

---

### 2.3 Feature: `affi receipt predict`

- [ ] **Prediction Handler** (45 min)
  - [ ] `handlers.rs::predict(receipt, prefix) -> Prediction`
  - [ ] Call `wasm4pm::predictive::next_activity(model, prefix, depth=1)`
  - [ ] Return activity + confidence + top-K alternatives

- [ ] **OTel Integration** (30 min)
  - [ ] Emit span with activity, confidence, distance
  - [ ] Set attribute: matched=(actual_next == predicted_next)

- [ ] **CLI & Test** (30 min)
  - [ ] Test at each event: prediction must be valid

**Estimated Effort:** 1.5 hours

---

### 2.4 Feature: LSP Hover

- [ ] **LSP Server Scaffolding** (1h)
  - [ ] Create `src/lsp/mod.rs` + `src/lsp/hover.rs`
  - [ ] Implement LSP Server trait (if using existing lsp-max infra)

- [ ] **Hover Handler** (1h)
  - [ ] Extract receipt event by ID
  - [ ] Return event_type, timestamp, commitment, objects in hover text

- [ ] **Test** (1h)
  - [ ] Simulate LSP textDocument/hover request
  - [ ] Verify response contains event details

**Estimated Effort:** 3 hours

---

### 2.5 Feature: LSP Goto-Definition

- [ ] **Definition Resolver** (45 min)
  - [ ] `src/lsp/goto_definition.rs::resolve(event_type) -> Location`
  - [ ] Map event_type (string) to handler source file + line

- [ ] **Test** (30 min)
  - [ ] "emit" event → emit.rs:X (handler definition)

**Estimated Effort:** 1.25 hours

---

## Phase 3: Mutation Testing (Week 3)

### 3.1 Feature: `affi mutate receipt`

- [ ] **Mutation Module** (1.5h)
  - [ ] Create `src/mutation.rs::MutationOperator` trait
  - [ ] Implement: EventDrop, EventReorder, TypeChange, PayloadFlip
  - [ ] Apply N random mutations to receipt

- [ ] **Handler** (45 min)
  - [ ] `handlers.rs::mutate_receipt(receipt, count) -> Vec<Receipt>`
  - [ ] Return all mutants

- [ ] **Verifier Integration** (30 min)
  - [ ] Run verifier on each mutant
  - [ ] Compute kill_count (mutations rejected by verifier)
  - [ ] Report: "killed 95/100 mutants (95% sensitivity)"

- [ ] **Test** (1h)
  - [ ] Generate 10 mutations; verify all have unique hashes
  - [ ] Verify ≥90% killed

**Estimated Effort:** 4 hours  
**Dependencies:** clnrm (mutation operators)

---

### 3.2 Feature: `affi generate test`

- [ ] **Codegen Templates** (45 min)
  - [ ] Create Tera template for Rust test boilerplate
  - [ ] Template: `#[test] fn test_{{ pattern_name }}() { ... }`
  - [ ] Embed fixture builder calls

- [ ] **Handler** (30 min)
  - [ ] `handlers.rs::generate_test(receipt) -> String`
  - [ ] Render template with fixture data

- [ ] **Test** (30 min)
  - [ ] Generated test code compiles
  - [ ] Generated test runs and passes

**Estimated Effort:** 1.5 hours

---

### 3.3 Feature: Property-Based Testing

- [ ] **Generators** (1h)
  - [ ] Implement `Arbitrary` for Receipt, OperationEvent
  - [ ] Use quickcheck for 100 random receipts

- [ ] **Property Tests** (1h)
  - [ ] Property: verifier always returns decidable verdict (no panic)
  - [ ] Property: fixture receipts always pass verification
  - [ ] Property: tampered receipts always fail verification

- [ ] **Test Suite** (30 min)
  - [ ] Create `tests/property_based.rs`

**Estimated Effort:** 2.5 hours

---

### 3.4 Feature: Fixture Database

- [ ] **Fixture Store** (1h)
  - [ ] Create `src/fixture_db.rs::FixtureDatabase`
  - [ ] SQLite-backed (rusqlite) or JSON file-backed
  - [ ] Schema: fixture_id, receipt_json, pattern, event_count, object_types, tags

- [ ] **Index Building** (45 min)
  - [ ] Build in-memory index by attribute
  - [ ] Implement search_by_attribute()

- [ ] **Handler & Test** (45 min)
  - [ ] `handlers.rs::fixture_search(attribute, value) -> Vec<Fixture>`
  - [ ] Test insert + search

**Estimated Effort:** 2.5 hours

---

### 3.5 Feature: `affi generate snippet`

- [ ] **Snippet Library** (30 min)
  - [ ] Extract code snippets from chicago-tdd-tools examples
  - [ ] Organize by pattern: linear, branching, loop, etc.

- [ ] **Handler** (30 min)
  - [ ] `handlers.rs::generate_snippet(pattern) -> Vec<String>`
  - [ ] Return copy-paste-ready code samples

**Estimated Effort:** 1 hour

---

## Phase 4: Observability (Week 4)

### 4.1 Feature: `affi receipt trace`

- [ ] **Span Emitter** (45 min)
  - [ ] Create `src/tracing.rs::trace_receipt(receipt) -> Vec<Span>`
  - [ ] Emit span for each verifier stage (decode, check_format, chain_integrity, etc.)
  - [ ] Set parent-child relationships

- [ ] **Handler** (30 min)
  - [ ] `handlers.rs::trace_receipt(receipt) -> Result<String>` (JSON spans)

- [ ] **Test** (30 min)
  - [ ] Trace with in-memory exporter; verify span structure

**Estimated Effort:** 1.5 hours

---

### 4.2 Feature: OTel Metrics Dashboard

- [ ] **Metrics Exporter** (1h)
  - [ ] Create `src/metrics.rs::MetricsCollector`
  - [ ] Define: throughput (events/sec), error_rate (%), latency (ms)

- [ ] **Prometheus Integration** (45 min)
  - [ ] Export metrics in Prometheus format
  - [ ] Set up scrape config (if in CI/CD context)

- [ ] **Grafana Dashboard** (1h)
  - [ ] Create `dashboards/affidavit.json` (Grafana JSON)
  - [ ] Add panels for throughput, error rate, latency

- [ ] **Test** (30 min)
  - [ ] Collect metrics from receipt verification
  - [ ] Verify Prometheus format is valid

**Estimated Effort:** 3 hours

---

### 4.3 Feature: OTel Baggage

- [ ] **Baggage Setup** (30 min)
  - [ ] Modify `src/tracing.rs` to attach baggage on receipt entry
  - [ ] Baggage keys: receipt_id, timestamp, version

- [ ] **Handler** (15 min)
  - [ ] Baggage automatically propagated through handler calls

- [ ] **Test** (15 min)
  - [ ] Extract baggage; verify receipt_id is present

**Estimated Effort:** 1 hour

---

### 4.4 Feature: Span Events

- [ ] **Event Emitter** (30 min)
  - [ ] Emit span event for each verifier stage
  - [ ] Include event name + timestamp

- [ ] **Test** (15 min)
  - [ ] Verify span has ≥6 events (one per stage)

**Estimated Effort:** 45 min

---

### 4.5 Feature: SLO Monitoring

- [ ] **SLI Calculator** (1h)
  - [ ] Create `src/metrics.rs::compute_sli(metrics) -> SLI`
  - [ ] Calculate: p99 latency, error rate %, availability %

- [ ] **Threshold Checking** (30 min)
  - [ ] Compare SLI vs configured thresholds
  - [ ] Return pass/fail + error budget remaining

- [ ] **Test** (30 min)
  - [ ] Verify SLI computation for sample metrics

**Estimated Effort:** 2 hours

---

## Phase 5: CLI Ergonomics (Week 5)

### 5.1 Feature: Help Formatter

- [ ] **Markdown→ASCII Conversion** (30 min)
  - [ ] Create `src/cli.rs::format_help_markdown(md: &str) -> String`
  - [ ] Strip **bold**, ``code``, remove code fences, convert links

- [ ] **ARDPRD Cross-References** (30 min)
  - [ ] Enhance help text with "See also: ARDPRD §3"
  - [ ] Map verbs to ARDPRD sections in config

- [ ] **Test** (15 min)
  - [ ] Help text has no markdown syntax; has ARDPRD refs

**Estimated Effort:** 1.25 hours

---

### 5.2 Feature: Auto-Generated Examples

- [ ] **Example Generator** (1h)
  - [ ] ggen rule: produce `.sh` files from chicago-tdd fixtures
  - [ ] Template: `#!/bin/bash\necho "...\n"; affi receipt inspect ...`

- [ ] **Example Runner** (30 min)
  - [ ] Test harness: execute each example, verify exit code 0

- [ ] **Test** (30 min)
  - [ ] All examples runnable without error

**Estimated Effort:** 2 hours

---

### 5.3 Feature: Aliases

- [ ] **Ontology Extension** (15 min)
  - [ ] Add `cnv:hasAlias "r"` to receipt noun in affi-cli.ttl

- [ ] **Parsing** (15 min)
  - [ ] clap-noun-verb already supports aliases via ontology

- [ ] **Test** (15 min)
  - [ ] `affi r inspect` routes to `affi receipt inspect`

**Estimated Effort:** 45 min

---

### 5.4 Feature: JSON Output

- [ ] **Output Formatters** (1h)
  - [ ] Implement `InspectionReport::to_json()`, `DiffResult::to_json()`, etc.
  - [ ] Use serde_json serialization

- [ ] **Handler Wrapper** (30 min)
  - [ ] Add `--format=json` flag to verbs
  - [ ] Default: pretty-print; --format=json: JSON output

- [ ] **Test** (30 min)
  - [ ] Parse output as JSON; verify schema

**Estimated Effort:** 2 hours

---

### 5.5 Feature: Interactive Shell REPL

- [ ] **Shell Loop** (1h)
  - [ ] Create `src/bin/affi-shell.rs`
  - [ ] Use rustyline for history + completion
  - [ ] Commands: load, inspect, diff, mutate, trace, help, quit

- [ ] **Command Dispatcher** (45 min)
  - [ ] Parse shell commands; dispatch to handlers
  - [ ] Maintain current receipt in memory

- [ ] **Test** (1h)
  - [ ] Test each command in shell context

**Estimated Effort:** 2.75 hours

---

## Global Checklist

### Pre-Implementation

- [ ] Verify all dependencies are in Cargo.toml
  - [ ] chicago-tdd-tools: ✅ (dev-deps)
  - [ ] wasm4pm-compat: ✅ (deps)
  - [ ] Criterion: ✅ (dev-deps)
  - [ ] opentelemetry: ✅ (dev-deps + feature "otel")
  - [ ] clap-noun-verb: ✅ (deps)
  - [ ] quickcheck: ⚠️ Need to add
  - [ ] rustyline: ⚠️ Need to add
  - [ ] clnrm: ⚠️ Need to research/add

- [ ] Review existing code
  - [ ] `src/handlers.rs` — where new handlers will live
  - [ ] `src/ocel.rs` — existing OCEL scaffolding (can reuse types)
  - [ ] `src/tracing.rs` — OTel integration point
  - [ ] `benches/receipt_operations.rs` — Criterion baseline

- [ ] Ontology Planning
  - [ ] Review `ontology/affi-cli.ttl` structure
  - [ ] Plan verb additions: inspect, diff, visualize, catalog, model, conform, predict, mutate, generate, trace
  - [ ] Plan flag additions: --format, --output, --model, --count, etc.

### Per-Phase Quality Gates

- [ ] **Phase 1 Exit Criteria**
  - [ ] 5 features implemented
  - [ ] 10 tests pass
  - [ ] E2E inspection test passes
  - [ ] `cargo build --all-features` succeeds
  - [ ] `cargo test --all` passes

- [ ] **Phase 2 Exit Criteria**
  - [ ] 5 discovery features implemented
  - [ ] `affi receipt model` produces valid Petri net JSON
  - [ ] Conformance fitness scores computed
  - [ ] LSP hover/goto tested with mock requests
  - [ ] E2E discovery test passes

- [ ] **Phase 3 Exit Criteria**
  - [ ] Mutation testing scores ≥90% kill rate
  - [ ] Generated test code compiles and runs
  - [ ] Fixture database queryable
  - [ ] Property-based tests pass on 100 random receipts
  - [ ] E2E mutation test passes

- [ ] **Phase 4 Exit Criteria**
  - [ ] OTel spans emitted and validated
  - [ ] Prometheus metrics exportable
  - [ ] Grafana dashboard JSON valid
  - [ ] SLI computation correct
  - [ ] E2E observability test passes

- [ ] **Phase 5 Exit Criteria**
  - [ ] All verbs support JSON output
  - [ ] Shell REPL accepts all commands
  - [ ] All auto-generated examples runnable
  - [ ] Help text has no markdown; includes ARDPRD refs
  - [ ] E2E CLI test passes

### Testing Strategy

- [ ] Unit tests: 1 per handler function (22 tests)
- [ ] Integration tests: 1 per feature pair (11 tests)
- [ ] E2E tests: 1 per 5 features (6 suites, ~150 lines each)
- [ ] Compile-fail tests: Verify feature gates work
- [ ] Regression tests: Criterion benchmarks block merge on regression

### Documentation Requirements

- [ ] Feature doc: 1 markdown per feature (22 files, ~50 lines each)
- [ ] API doc: Rust doc comments on all new public functions
- [ ] Examples: 5 runnable shell scripts (auto-generated from fixtures)
- [ ] ARDPRD cross-references: Help text links to §3 (receipt verbs), §4 (sealing)

### CI/CD Integration

- [ ] **Pre-commit hook:** `cargo fmt`, `cargo clippy --all-targets --all-features`
- [ ] **Test pipeline:** `cargo test --all` + `cargo test --all-features`
- [ ] **Benchmark pipeline:** `cargo bench --bench receipt_operations` — flag regressions
- [ ] **Doc generation:** `cargo doc --all --no-deps` — ensure no broken links
- [ ] **Feature coverage:** Test all feature combinations (discovery, conformance, predictive, otel, etc.)

---

## Success Metrics (Continuous Monitoring)

| Metric | Target | Check Method |
|--------|--------|---|
| Test code coverage | ≥80% of handlers | `cargo tarpaulin --all-features` |
| Benchmark stability | No >10% regression | Criterion comparison baseline |
| Feature completeness | 22/22 implemented | Checklist completion |
| Mutation sensitivity | ≥90% kill rate | `affi mutate` output |
| Example pass rate | 100% | Example runner test |
| CLI help accuracy | 100% matches ontology | Manual spot-check + test |
| OTel span correctness | All spans have parent_id | LSP/test assertions |
| Documentation freshness | All examples runnable | Weekly automated run |

---

## Next Steps

1. **Today:** Approve this checklist; identify any gaps
2. **Day 1:** Set up Phase 1 dev branch; add missing dependencies (quickcheck, rustyline, clnrm)
3. **Day 2-5:** Execute Phase 1 features in order (inspect → diff → visualize → catalog → completion)
4. **Week 2-5:** Execute Phases 2-5 in sequence, with quality gates at end of each phase

---

**Expected Outcome:** After 5 weeks, receipt workflows will feel 10,000x better:
- 10x faster test writing (fixtures)
- 10x faster feedback (mutation testing)
- 10x easier adoption (completion + help + examples)
- 10x more confidence (conformance + benchmarks)

