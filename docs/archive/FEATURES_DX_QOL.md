# DX/QOL Features v26.6.17+ — 80/20 Integration from Ecosystem Libraries

This document outlines 1000x DX/QOL improvements using **80% existing library code, 20% new glue**.

## 1. Receipt Inspection & Visualization (chicago-tdd-tools + ggen)

### Feature: `affi receipt inspect`
- **80%:** chicago-tdd-tools' test fixtures (pre-built receipt templates)
- **20%:** New verb registering with ggen ontology, handlers wrapping fixtures
- **Benefit:** Inspect recipes without writing test code; templates for common patterns
- **Test Witness:** `tests/inspect_dxqol.rs` — fixture-driven receipt generation

### Feature: `affi receipt replay`
- **80%:** wasm4pm's trace replay engine + Chicago TDD fixture builders
- **20%:** New verb handler feeding assembled receipt to replay
- **Benefit:** See each event re-execute; trace divergence detection
- **Test Witness:** Replay a 10-event receipt, compare traces before/after

## 2. Process Discovery & Conformance (wasm4pm + lsp-max)

### Feature: `affi receipt model`
- **80%:** wasm4pm's discovery (heuristic inductive mining)
- **20%:** Handler translating receipt events to wasm4pm::Event, calling discovery
- **Benefit:** Auto-generate DFG/Petri model from receipt; visual conformance check
- **Test Witness:** Receipt → discovered model → fitness metric

### Feature: LSP hover/goto on receipts
- **80%:** lsp-max's document-symbol, hover infrastructure
- **20%:** Receipt-specific hover (show event details, object traces)
- **Benefit:** IDE integration; jump from receipt to source, trace objects
- **Test Witness:** Launch LSP server, hover on event ID, verify details

## 3. Benchmarking & Regression Detection (Criterion + wasm4pm)

### Feature: `affi bench receipt-throughput`
- **80%:** Criterion's harness + wasm4pm fitness metrics
- **20%:** New benchmark measuring emit→assemble→verify latency per event count
- **Benefit:** Auto-detect performance regressions; regression-failure blocks merge
- **Test Witness:** Benchmark 5/10/50/100 events, compare against baseline

### Feature: `affi bench variance`
- **80%:** wasm4pm's process variance discovery
- **20%:** Benchmark measuring control-flow surprise (unexpected event orderings)
- **Benefit:** Detect cheating via process anomalies; test suite drift detection
- **Test Witness:** 1000 receipts, variance histogram

## 4. Test Generation & Mutation Testing (chicago-tdd-tools + clnrm)

### Feature: `affi generate test`
- **80%:** chicago-tdd-tools fixture builders + clnrm scenario templates
- **20%:** Handler generating Rust test code from templates
- **Benefit:** Auto-generate receipt test cases from patterns
- **Test Witness:** Generate 10 test cases, all compile and pass

### Feature: `affi mutate receipt`
- **80%:** clnrm mutation operators (event drop, reorder, type change)
- **20%:** Handler applying mutations, collecting verdicts
- **Benefit:** Kill testing; find weak verifier rules
- **Test Witness:** Mutate receipt 100 ways, verify rejects all

## 5. OpenTelemetry & Observability (OTel + ggen)

### Feature: `affi receipt trace --span-export=jaeger`
- **80%:** OpenTelemetry's Jaeger exporter + wasm4pm fitness tracing
- **20%:** Handler emitting spans for each stage (emit/assemble/verify)
- **Benefit:** Full distributed tracing; see receipt cross-cutting concerns
- **Test Witness:** Emit receipt, verify Jaeger has spans with correct parent IDs

### Feature: OTel metrics dashboard
- **80%:** Prometheus metrics from OpenTelemetry
- **20%:** Custom metrics for receipt throughput, error rate, fitness
- **Benefit:** Real-time receipt health; SLO enforcement
- **Test Witness:** Dashboard query returns recent metrics

## 6. CLI Ergonomics & Completion (clap-noun-verb + ggen)

### Feature: Shell completion (bash/zsh/fish)
- **80%:** clap-noun-verb's builtin completion generation
- **20%:** Register new verbs with ggen, completion auto-wires
- **Benefit:** `affi receipt <TAB>` shows all verbs + args
- **Test Witness:** Completion script runs without error

### Feature: Help formatter improvements
- **80%:** ggen's ontology documentation → clap-noun-verb help
- **20%:** Markdown→ASCII formatting for help output
- **Benefit:** Help text auto-generated from ontology; always in sync
- **Test Witness:** `affi receipt --help` matches ontology docs

## 7. Documentation & Examples (ggen + chicago-tdd-tools)

### Feature: Auto-generated examples
- **80%:** chicago-tdd-tools fixture templates → example scripts
- **20%:** ggen rendering Markdown examples from fixtures
- **Benefit:** Always-working examples; examples use real receipt data
- **Test Witness:** Each example script runs without error

### Feature: Inline CLI reference (ARDPRD-shaped)
- **80%:** ggen's ontology → referenced sections
- **20%:** Human-written ARDPRD cross-references
- **Benefit:** `affi receipt verify --help` references ARDPRD §3 FR-3
- **Test Witness:** Help text contains ARDPRD cross-references

## Implementation Priority (80/20 order)

1. **chicago-tdd-tools fixture integration** (highest ROI; enables test generation, examples, inspection)
2. **clnrm mutation testing** (DX: find weak rules immediately)
3. **Criterion benchmarking** (already 80% done; finish regression detection)
4. **wasm4pm discovery/conformance** (architecture; enable model checking)
5. **OTel metrics** (observability; already partially wired)
6. **lsp-max IDE integration** (lowest priority; IDE support)

## Success Metrics

- [ ] 5+ new verbs registered with ggen ontology
- [ ] 10+ chicago-tdd-tools fixtures available
- [ ] Criterion dashboard shows 3+ regressions prevented
- [ ] wasm4pm discovery runs on every test receipt
- [ ] OTel spans appear in Jaeger for 90% of operations
- [ ] Shell completion works for all new verbs
- [ ] All examples pass; zero example bit-rot

## Definition of "1000x DX/QOL"

Not "1000x faster" — rather:
- **10x fewer lines of test code** (fixtures vs hand-written events)
- **10x faster feedback** (mutations found issues in <1s)
- **10x easier to adopt** (completion + help + examples make it self-explanatory)
- **10x more confidence** (conformance checks + benchmarks catch regressions)

**Total: 10 × 10 × 10 × 10 = 10,000x better experience** (conservative estimate).

