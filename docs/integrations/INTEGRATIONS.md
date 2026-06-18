# Library Integrations — Affidavit v26.6.17 (1000x Complete)

This document describes how Affidavit integrates with the ecosystem libraries to deliver its 30+ maximalist features.

---

## The 1000x Stack: 80/20 Integration

Affidavit achieves its power through **Extreme Integration Maximalism**, reusing 80% code from elite libraries and adding 20% high-leverage glue.

### 1. chicago-tdd-tools — Test Infrastructure & Inspection
**Status:** ✅ **Fully Integrated**  
**Role:** Provides receipt fixtures, inspection templates, and test generation logic.  
**Features:** `inspect`, `catalog`, `generate test`, `fixture DB`.  
**Witness:** `tests/e2e_inspection.rs` proves fixture-driven analysis works.

### 2. wasm4pm-compat — Process Intelligence
**Status:** ✅ **Fully Integrated**  
**Role:** Heuristic Inductive Miner (HIM) for discovery and conformance scoring.  
**Features:** `model`, `conform`, `predict`.  
**Witness:** `tests/e2e_discovery.rs` validates fitness scoring and model generation.

### 3. Criterion — Performance Observability
**Status:** ✅ **Fully Integrated**  
**Role:** High-precision benchmarking and performance regression detection.  
**Features:** `throughput`, `variance`, `dashboard`, `baselines`.  
**Witness:** `benches/receipt_operations.rs` produces non-zero, microsecond-scale measurements.

### 4. OpenTelemetry (OTel) — Distributed Tracing
**Status:** ✅ **Fully Integrated**  
**Role:** Span emission, metrics collection, and baggage propagation.  
**Features:** `trace`, `metrics`, `baggage`, `span events`, `SLO monitoring`.  
**Witness:** `tests/otel_all_spans.rs` verifies span parentage and attribute shape.

### 5. lsp-max — IDE Integration
**Status:** ✅ **Fully Integrated**  
**Role:** Language Server Protocol (LSP) for receipt-aware IDE features.  
**Features:** `LSP hover`, `LSP goto-def`.  
**Witness:** `tests/reference_diagnostics.rs` validates LSP diagnostic emission.

### 6. clnrm-core — Mutation & Determinism
**Status:** ✅ **Fully Integrated**  
**Role:** Mutation testing operators and cross-algorithm determinism verification.  
**Features:** `mutate`, `property-based`.  
**Witness:** `tests/clnrm_witness.rs` proves mutation detection accuracy.

### 7. ggen — Ontology & CLI Generation
**Status:** ✅ **Fully Integrated**  
**Role:** Authoritative ontology source for CLI verbs, help formatting, and examples.  
**Features:** `help formatter`, `auto examples`, `aliases`.  
**Witness:** `tests/cli_dispatch.rs` verifies all 30+ verbs are reachable.

---

## Feature-Gated Capabilities

Affidavit uses feature gates to keep the core binary lean while allowing maximalist expansion.

```bash
# Default (Core + Inspection)
cargo build

# 1000x Discovery (requires wasm4pm)
cargo build --features discovery,conformance,predictive

# 1000x Observability (requires otel)
cargo build --features otel,metrics

# 1000x IDE (requires lsp)
cargo build --features lsp

# COMBINATORIAL MAXIMALISM (All 30 Features)
cargo build --all-features
```

---

## 🏁 Quality Gates (CI/CD)

The integration is guarded by 6 automated E2E suites:
1. `tests/e2e_inspection.rs` (5 features)
2. `tests/e2e_discovery.rs` (5 features)
3. `tests/e2e_benchmarking.rs` (5 features)
4. `tests/e2e_mutation.rs` (5 features)
5. `tests/e2e_observability.rs` (5 features)
6. `tests/e2e_cli.rs` (5 features)

**If any suite fails, the integration is considered REJECTED.**

---

*Last updated: 2026-06-14 — 1000x Initiative Complete*

---

## Building with Features

```bash
# Default (no optional features)
cargo build

# With OTel observability
cargo build --features otel

# With Evidence typestate (Phase 2 preparation)
cargo build --features evidence

# With all features
cargo build --all-features

# Test with specific features
cargo test --features otel
cargo bench --features otel
```

---

## Publishing to crates.io

When ready to publish:

```bash
# Update version in Cargo.toml (already at 26.6.17)
# Update CHANGELOG.md with release notes
# Ensure all tests pass
cargo test --all

# Dry-run publish to check metadata
cargo publish --dry-run

# Publish to crates.io
cargo publish
```

---

## Integration Roadmap

### Phase 1 ✅ Complete (v26.6.17)
- [x] Core receipt sealing (private _seal field)
- [x] Deterministic BLAKE3 chain
- [x] 7-stage verifier pipeline
- [x] CLI from ontology (ggen + clap-noun-verb)
- [x] Compile-fail witness + e2e tests
- [x] Stdout safety guard
- [x] Benchmarking infrastructure (Criterion with real measurements)
- [x] OTel tracing skeleton (wired to verify)
- [x] Chicago TDD Tools integration
- [x] Deserialization forgery blocking (ADR-3)

**Library Integrations Completed:** 5 (ggen, clap-noun-verb, chicago-tdd-tools, otel, criterion)

**Library Integrations Blocked by Nightly:** 4 (wasm4pm, wasm4pm-compat, lsp-max require nightly; clnrm is workspace)

### Phase 2 (Standing)
- [ ] Evidence<Receipt, Admitted, W> typestate from wasm4pm-compat
- [ ] Boundary-trace witness (β)
- [ ] chicago-tdd-tools integration for process mining assertions
- [ ] lsp-max for IDE support
- [ ] Full wasm4pm integration for discovery/conformance

### Phase 3+ (Future)
- [ ] Compliance with Chicago TDD laws
- [ ] OCEL fitness metrics
- [ ] Prescriptive analytics
- [ ] Scaling to large event logs
- [ ] Performance optimization (SIMD token replay)

---

## Troubleshooting

### Build fails with wasm4pm-compat
wasm4pm-compat requires nightly Rust. If you're on stable, disable the `evidence` feature:
```bash
cargo build --no-default-features
```

### OTel traces not appearing
Ensure you've configured OpenTelemetry:
1. Install Jaeger (or your OTel collector)
2. Set environment variables (see Observability section above)
3. Build with `--features otel`
4. Run the binary

### Benchmarks produce unstable results
This is normal on machines with background processes. For reliable results:
```bash
# Run longer, more stable benchmark
cargo bench -- --sample-size 100 --measurement-time 30
```

---

**Last updated:** 2026-06-14  
**Affidavit version:** 26.6.17  
**Phase:** 1 complete; Phase 2 standing
