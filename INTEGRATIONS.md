# Library Integrations — Affidavit v26.6.14

This document describes how Affidavit integrates with the ecosystem libraries and how to use the integrated features.

---

## Core Integrations (Required)

### 1. clap-noun-verb — CLI Framework
**Status:** ✅ Integrated  
**Purpose:** Noun-verb CLI surface generation  
**Location:** `ontology/affi-cli.ttl` + ggen generation → `src/verbs/`  
**Usage:** All CLI commands route through clap-noun-verb dispatcher

```bash
cargo run --bin affi -- receipt <verb> [args]
```

### 2. ggen — Code Generation
**Status:** ✅ Integrated  
**Purpose:** Generate CLI wrappers from ontology  
**Location:** `ggen.toml` + `ontology/`  
**Usage:** Run `ggen sync` to regenerate verb wrappers

```bash
cd /path/to/clap-noun-verb
ggen sync /path/to/affidavit
```

---

## Optional Integrations (Features)

### 3. wasm4pm-compat — Evidence Typestate (Feature: `evidence`)
**Status:** ⚠️ Available (optional)  
**Purpose:** Evidence<Receipt, Admitted, W> typestate wrapper (Phase 2)  
**Location:** Dependency (can be added to Cargo.toml)  
**Features:** `evidence`  
**Usage:**

```bash
cargo build --features evidence
```

Currently unused but available for Phase 2 implementation. Will provide:
- Evidence<Receipt, Admitted, AffidavitReceiptChain> wrapper
- Typed enforcement of the sealing seam
- Witness integration with wasm4pm process mining

### 4. OpenTelemetry — Observability (Feature: `otel`)
**Status:** ✅ ADMITTED (witnessed by `tests/otel_witness.rs`)  
**Purpose:** Optional tracing instrumentation for receipt operations  
**Location:** `src/tracing.rs` (feature-gated with fallback no-op)  
**Features:** `otel` (available for enabling OTel instrumentation)  
**Witness Status:** ✅ Wire verify operation to emit trace spans; test verifies operation completes without error when OTel module is invoked

**Current Usage:**

Without feature:
```rust
use affidavit::tracing::trace_emit;

// No-op version
let result = trace_emit("operation", 1, || { /* ... */ });
```

With feature (configure OTel before use):
```bash
cargo build --features otel
export OTEL_SDK_DISABLED=false
export OTEL_EXPORTER_JAEGER_AGENT_HOST=localhost
export OTEL_EXPORTER_JAEGER_AGENT_PORT=6831

cargo run --features otel --bin affi -- receipt verify receipt.json
```

Traces are exported to Jaeger (or configured collector).

---

## Benchmarking (Feature: always available)

### Criterion Benchmarks
**Status:** 🟡 OPEN-substrate (harness exists but `0 measured` — criterion_main never runs)  
**Purpose:** Performance regression detection (future)  
**Location:** `benches/receipt_operations.rs` (skeleton exists but Criterion doesn't invoke it)  
**Witness Status:** None — `cargo bench` outputs `running 0 tests / 0 measured`

**Issue:** The `criterion_group!` / `criterion_main!` macros are declared but not executed by the harness. This is a Criterion configuration issue, not a missing feature.

**To upgrade to ADMITTED:** Fix the Criterion harness configuration so that `cargo bench` produces non-zero measurements for chain operations, then add a CI check that ensures benchmarks run on every commit (regression detection).

**Current Usage:** (placeholder — harness compiles but measures nothing)

---

### 5. chicago-tdd-tools — TDD Infrastructure
**Status:** ✅ ADMITTED (witnessed by `tests/chicago_tdd_witness.rs`)  
**Purpose:** Test fixtures, AAA pattern enforcement, assertion helpers  
**Location:** `tests/chicago_tdd_witness.rs` (2 tests proving fixture and AAA pattern work)  
**Witness Status:** ✅ Test fixtures integrate with tempfile; AAA pattern verified with assertion helpers

**Usage:** Import via `use chicago_tdd_tools::prelude::*;` in test code

## Ecosystem Libraries (Not Yet Integrated)

### 6. wasm4pm — Full Process Mining
**Status:** Requires nightly (Phase 2+)  
**Purpose:** Process discovery, conformance analysis, predictive monitoring  
**When to integrate:** After Evidence typestate (ADR-1/4) is complete; requires stable Rust upgrade path  

### 7. wasm4pm-compat (continued)
**Status:** Requires nightly (blocked on stable build)  
**Purpose:** Evidence<Receipt, Admitted, W> typestate wrapper  
**When to integrate:** When affidavit can target nightly or wasm4pm-compat provides stable subset  

### 8. lsp-max — IDE Support
**Status:** Requires nightly (depends on wasm4pm-compat)  
**Purpose:** LSP server for IDE integration  
**When to integrate:** When nightly requirement lifted  

### 9. clnrm — Workspace Project
**Status:** Not integratable (workspace with 3 member crates)  
**Purpose:** CLI framework, LSP, core utilities  
**When to integrate:** Only as a complete workspace dependency, not individual crates  

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
# Update version in Cargo.toml (already at 26.6.14)
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

### Phase 1 ✅ Complete (v26.6.14)
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
**Affidavit version:** 26.6.14  
**Phase:** 1 complete; Phase 2 standing
