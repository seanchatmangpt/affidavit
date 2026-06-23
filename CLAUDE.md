# affidavit — Provenance Layer Documentation

**Version:** 26.6.22  
**Project:** Receipt Assembly & Certification  
**Language:** Rust (2021 edition)  
**License:** MIT OR Apache-2.0

---

## Overview

`affidavit` implements the **Provenance Layer**: an append-only, content-addressed chain of operation-events that certify what a process did. The `affi` CLI lets you:

1. **Emit** operation-events (record what happened)
2. **Assemble** into immutable receipts (finalize the chain)
3. **Verify** receipts against a formal standard (certify without deciding)

The project's doctrine: **certify, don't decide.** The verifier checks a receipt against a format standard and never decides whether work is honest.

---

## ⚠️ Operational ground truth for coding agents (read first)

The root build situation changed — earlier docs that say "the root crate cannot compile" are now **stale**:

- The root `affidavit` crate **builds and tests**. The previously-broken upstream crates (`wasm4pm`, `wasm4pm-compat`, `clnrm-core`) are replaced by local stubs via `[patch.crates-io]` in `Cargo.toml` (`stubs/`). `cargo build --all-targets` and `cargo test` pass (789 tests, incl. doctests); `cargo fmt --all -- --check` passes.
- The one red gate is `cargo clippy --all-targets -- -D warnings`: `src/lib.rs` sets `#![deny(clippy::print_stdout)]` but the library still has ~236 raw `println!` calls (pre-existing lint debt, ~270 findings total). Not a build break; CI keeps the `clippy` job non-blocking until it's paid down. Don't delete `stubs/` or the `[patch]` block to "fix" deps.
- Buildable, tested subprojects also live elsewhere: **`affidavit-core/`** (zero-dep `no_std` verifier + process mining — `cargo test` green), **`web/`** (Next.js — `npx tsc --noEmit`), **`tools/confevo/`** (Python — `python3 -m unittest`).
- Full operational map, per-area validate commands, and conventions: **[`AGENTS.md`](AGENTS.md)** (and **[`affidavit-core/AGENTS.md`](affidavit-core/AGENTS.md)** for that crate's strict invariants).

Everything below describes the *intended* `affidavit` design — treat it as the spec, not the current build state.

---

## Architecture

### High-Level Structure

```
affidavit/
├── src/
│   ├── bin/affi.rs           # CLI entrypoint
│   ├── bin/affi-shell.rs     # Interactive REPL (feature: shell)
│   ├── lib.rs                # Public API
│   ├── cli.rs                # Clap configuration (noun-verb pattern)
│   ├── chain.rs              # Receipt construction & sealing
│   ├── verifier.rs           # 7-stage certify pipeline
│   ├── types.rs              # Domain types (Event, Receipt, Verdict)
│   ├── admission.rs          # Validation gates
│   ├── registry.rs           # Compile-time verb registry (67 verbs, 10 groups)
│   ├── diag.rs               # Stable exit codes & structured diagnostics
│   ├── output.rs             # Unified Out handle (human/JSON/YAML)
│   ├── discovery.rs          # Type discovery & schema registry
│   ├── ocel.rs               # Object-Centric Event Logs integration
│   ├── handlers.rs           # Event dispatch & routing
│   ├── lsp/                  # Language server integration
│   ├── tracing.rs            # Observable spans & telemetry
│   ├── quality.rs            # Western Electric SPC monitoring
│   ├── sbom.rs               # SBOM generation & parsing
│   ├── sbom_compliance.rs    # NTIA compliance checking
│   ├── sbom_vulnerability.rs # Vulnerability aggregation & risk
│   ├── verbs/                # 67 command implementations
│   │   ├── emit.rs           # emit — record an event
│   │   ├── assemble.rs       # assemble — finalize receipt
│   │   ├── verify.rs         # verify — certify pipeline
│   │   ├── show.rs           # show — human-readable dump
│   │   ├── inspect.rs        # inspect — detailed analysis
│   │   ├── diagnose.rs       # diagnose — troubleshoot failures
│   │   ├── doctor.rs         # doctor — env & receipt-store health check
│   │   ├── stats.rs          # stats — chain metrics
│   │   ├── graph.rs          # graph — DAG visualization
│   │   ├── replay.rs         # replay — re-execute chain
│   │   ├── model.rs          # model — type schema extraction
│   │   ├── conformance.rs    # conformance — profile checking
│   │   └── [55 more verbs across 10 groups]
│   └── [quality, sbom, 1000x, and other modules]
├── docs/
│   ├── INDEX.md              # Documentation index (this session's primary nav)
│   ├── innovation/           # 5 DX/QoL design proposals (v26.6.19 fan-out)
│   ├── roadmap/              # 10-workstream 2030 program plan
│   └── integrations/         # LSP, WASM4PM, CLNRM integration guides
├── examples/
│   ├── golden_run.sh         # Full lifecycle (emit → assemble → verify)
│   └── [other runnable examples]
├── tests/
│   └── [integration tests]
├── benches/
│   └── [Criterion benchmarks]
├── rust-boilerplate/         # Cross-repo Rust scaffolding template
├── SECURITY.md               # Security policy & disclosure
├── deny.toml                 # Supply-chain advisory checks
├── typos.toml                # Automated typo detection
├── Cargo.toml
├── README.md
└── CLAUDE.md [this file]
```

### Dependency Ecosystem

**Local dependencies** (monorepo):
- `clap-noun-verb` — CLI argument parsing (noun/verb pattern)
- `clnrm-core` — Canonicalization & normalization primitives
- `wasm4pm` — Package manager & module system
- `lsp-max` — Language server extensions

**External crates**:
- `blake3` — Content addressing & chain hashing
- `serde` / `serde_json` — Serialization
- `anyhow` / `thiserror` — Error handling
- `linkme` — Plugin discovery
- `opentelemetry` — Observability (via feature gate)

---

## Key Concepts

### 1. The Receipt Chain

A receipt is a **BLAKE3 chain of operation-events**:

```json
{
  "format_version": "core/v1",
  "events": [
    {
      "seq": 0,
      "event_id": "evt-0",
      "event_type": "build",
      "objects": [{"id": "repo:main", "type": "git"}],
      "commitment": "6ef47c82..."
    },
    {
      "seq": 1,
      "event_id": "evt-1",
      "event_type": "test",
      "objects": [{"id": "suite:unit", "type": "test-suite"}],
      "commitment": "a2d95f11..."
    }
  ],
  "chain_hash": "203d3bbf...",
  "profile": "core/v1"
}
```

Each event's content-address (`blake3(payload)`) is stored as `commitment`. The rolling chain hash folds each event's bytes into the next, so **any edit propagates through all later links**.

### 2. Sealing & Immutability

Receipts are **sealed** — their `_seal` field is private and only constructible via the canonical `ChainAssembler::finalize` method. This makes struct-literal construction impossible at compile time (Rust: `E0451`).

```rust
// This fails at compile-time:
let receipt = Receipt { _seal: (), ... };  // ERROR: private field

// This works (only canonical path):
let receipt = assembler.finalize()?;       // ✓
```

### 3. The Verifier: 7-Stage Pipeline

The verifier maps 1:1 to a C4 Level-3 component view:

| # | Stage | Component | Check |
|---|-------|-----------|-------|
| 1 | `decode` | decode | Receipt is present & version field parses |
| 2 | `check_format` | check_format | `format_version == "core/v1"` |
| 3 | `chain_integrity` | check_chain_integrity | Recompute rolling BLAKE3; must match stored `chain_hash` |
| 4 | `continuity` | resolve_continuity | `seq` contiguous from 0; event IDs unique |
| 5 | `verify_commitments` | verify_commitments | Each commitment is a well-formed BLAKE3 digest |
| 6 | `evaluate_profile` | evaluate_profile | Profile `core/v1`: each event has `event_type` & commitment |
| 7 | `emit_verdict` | emit Verdict | ACCEPT iff all stages pass; else REJECT with first failure reason |

**Exit codes:**
- `0` → ACCEPT
- `2` → REJECT

### 4. Determinism Guarantees

- **No wall-clock:** Events ordered by monotonic `seq`, not timestamps
- **Canonical JSON:** Events serialized with deterministic field ordering
- **Same inputs → same receipt:** Receipt hash is reproducible

---

## CLI Surface

The CLI exposes **67 canonical verbs** across 10 groups, defined in `src/registry.rs`. Run `affi --help` for the full list or use `affi guide search <keyword>` for fuzzy lookup.

**Verb groups:** Core · Diagnostics · Analysis · Ingestion · Compliance · Attestation · SBOM · Insights · Engineering · Tooling

### Core Commands (Noun-Verb Pattern)

Affidavit v26.6.22 provides a comprehensive CLI organized into 9 primary verb families:

**Core Provenance (11 verbs):**
`emit`, `assemble`, `verify`, `show`, `inspect`, `diagnose`, `stats`, `graph`, `replay`, `model`, `conformance`

**Emit Variants (8 verbs):**
`emit-batch`, `emit-from-cicd`, `emit-from-cloud`, `emit-from-github`, `emit-from-gitlab`, `emit-from-monitoring`, `emit-from-sbom`, `emit-from-security`

**Assemble Variants (2 verbs):**
`assemble-and-notarize`, `assemble-with-signature`

**Verify Variants (3 verbs):**
`verify-compliance`, `verify-family`, `verify-sla`

**SBOM & Supply Chain (5 verbs):**
`sbom-scan`, `sbom-attest`, `sbom-blast-radius`, `sbom-compliance`, `sbom-ntia`

**Quality & Monitoring (6 verbs):**
`monitor`, `portfolio-health`, `trend-analysis`, `variance`, `anomaly-detect`, `predict`

**Audit & Compliance (10 verbs):**
`audit`, `attest`, `notarize`, `sign`, `gdpr-proof`, `hipaa`, `pci-dss`, `soc2-audit`, `license-compliance`, `policy-enforce`

**Analysis & Troubleshooting (14 verbs):**
`causality-chain`, `dependency-matrix`, `security-debt`, `tech-debt`, `root-cause`, `explain-incident`, `find-blast-radius`, `bus-factor`, `orphaned-code`, `coverage-analysis`, `dora-metrics`, `team-velocity`, `find-slow-test`, `regression-analysis`

**Developer Tools (6 verbs):**
`doctor`, `diff`, `visualize`, `catalog`, `search`, `query`, `timeline`, `profile`, `receipt-throughput`, `install-git-hook`, `test`

See `src/verbs/` for implementation details or run `affi --help` for full usage.

```bash
affi doctor [OPTIONS]
  [--receipts <path>]
  [--format {json,human}]
```
Run environment and receipt-store health checks. Reports installation status, feature availability, and chain integrity across the store. Exit codes use the stable catalog in `src/diag.rs`.

---

## Development Workflow

### Build & Test

```bash
# Build debug binary
cargo build
cargo run --bin affi -- emit --help

# Run all tests (211+ tests across unit, integration, property, and compliance suites)
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test verify_chain_integrity

# Benches (requires nightly or --release)
cargo bench --bench receipt_operations
```

### Run the Golden Example

```bash
# Full lifecycle: emit → assemble → verify (honest) → verify (tampered)
bash examples/golden_run.sh
```

Output shows:
- Event emission and sequencing
- Receipt assembly with content address
- 7-stage verify pipeline on clean receipt (ACCEPT)
- Tampered receipt detection (REJECT with reason)

### Key Test Patterns

**Unit tests** (`src/lib.rs`):
- Chain construction & sealing
- BLAKE3 commitment verification
- Event admission gates

**Dispatch tests** (`src/handlers.rs`):
- Event routing & handler dispatch
- Type discovery & schema resolution

**E2E tests** (`tests/`):
- Full CLI pipeline
- Receipt round-trip (emit → assemble → verify)
- Tampering detection

**UI tests** (`src/cli.rs`):
- CLI argument parsing
- Error messages & output formatting

---

## Integration Points

### OCEL (Object-Centric Event Logs)

`src/ocel.rs` provides integration with OCEL standards:

```rust
use affidavit::ocel::{OcelEvent, OcelAdapter};

// Convert affidavit event to OCEL
let ocel_event = OcelAdapter::from_receipt_event(&event)?;
```

See `examples/ocel_events.rs`.

### LSP (Language Server)

`src/lsp.rs` exposes receipt verification as LSP diagnostics:

- Hover over a receipt path → shows receipt summary
- Diagnostics → shows verification failures per-line
- Code actions → suggests fixes

Integrates via `lsp-max`.

### Observable Spans (Telemetry)

With the `otel` feature:

```bash
cargo build --features otel
cargo run --bin affi -- verify receipt.json
  # Emits OpenTelemetry spans to local Jaeger
```

See `examples/observable_spans.rs` for span structure.

### Type Discovery & Plugins

Via `linkme`, custom event handlers and type schemas can be discovered:

```rust
#[linkme::distributed_slice(CUSTOM_HANDLERS)]
pub static MY_HANDLER: EventHandler = EventHandler::new("my-type", handle_my_event);
```

---

## Code Conventions

### File Layout

- **Private modules** (`src/discovery.rs`): internal machinery
- **Public API** (`src/lib.rs`): re-exports & documentation
- **Verbs** (`src/verbs/*.rs`): one command per file
- **Examples** (`examples/*.rs`): runnable demonstrations
- **Tests** inline (unit) or in `tests/` (integration)

### Naming

- **Event types:** lowercase + dash (`build`, `test`, `audit-log`)
- **Object IDs:** `id:type[:qualifier]` format (`repo:main`, `suite:unit:fast`)
- **Commitment digests:** lowercase hex, no prefix
- **Receipt files:** blake3 hash of canonical bytes OR named `.json`

### Error Handling

- **Admission gates** (`src/admission.rs`): reject invalid events early
- **Verifier stages** (`src/verifier.rs`): fail fast, report first failure
- **CLI** (`src/cli.rs`): map internal errors to user-friendly messages

### No Unwrap Policy

All fallible operations use `Result<T, E>` with proper error propagation. Tests may use `.unwrap()` for brevity.

---

## Common Tasks

### Add a New Verb (Command)

1. Create `src/verbs/myverb.rs`
2. Implement `pub async fn handle_myverb(args: MyVerbArgs) -> Result<()>`
3. Add to `src/verbs/mod.rs` and `src/cli.rs`
4. Add a test in the module
5. (Optional) Add an example in `examples/`

### Extend the Verifier

1. Add a new stage struct in `src/verifier.rs`
2. Implement the `VerificationStage` trait
3. Insert into the pipeline (order matters!)
4. Update the stage table in README & this file
5. Add a unit test

### Add a Custom Event Handler

1. Create `src/handlers/myhandler.rs`
2. Implement the handler function signature
3. Register via `#[linkme::distributed_slice]` or manual registry
4. Add tests

### Integrate with New Ecosystem

1. Add dependency to `Cargo.toml`
2. Create integration module (`src/myecosystem.rs`)
3. Re-export from `src/lib.rs`
4. Add example in `examples/`
5. Document in this file (Integration Points section)

---

## Documentation Ecosystem

- **README.md** — Quick start, doctrine, CLI surface, worked example
- **CLAUDE.md** — Full project guide (this file)
- **Cargo.toml** — Package metadata, dependencies, features
- **src/lib.rs** — Public API docs & module overview
- **examples/** — Runnable demonstrations of key workflows
- **CHANGELOG.md** — Version history & breaking changes
- **LSP_MAX_INTEGRATION_*.md** — Language server integration details
- **CLNRM_INTEGRATION_*.md** — Canonicalization integration
- **WASM4PM_*.md** — Package manager integration

---

## Testing Strategy

### Test Organization

```
Unit tests (19)
├── src/chain.rs (5) — sealing, rolling hash
├── src/admission.rs (4) — gate validation
├── src/types.rs (3) — type parsing
├── src/discovery.rs (3) — schema resolution
└── src/verifier.rs (4) — stage correctness

Dispatch tests (6)
└── src/handlers.rs — handler routing

E2E tests (4)
└── tests/ — full CLI pipeline

UI tests (1)
└── src/cli.rs — argument parsing & help text
```

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_chain_integrity

# With logging
RUST_LOG=debug cargo test -- --nocapture

# Single-threaded (for determinism checks)
cargo test -- --test-threads=1

# Benches
cargo bench
```

### Determinism Testing

For receipt determinism guarantees, use `--test-threads=1`:

```bash
cargo test receipt_determinism -- --test-threads=1
```

---

## Performance Characteristics

### Benchmarked Operations (see `benches/receipt_operations/`)

- **Emit event:** ~100µs (JSON parse + admission + sealing)
- **Assemble 100 events:** ~50ms (rolling BLAKE3 computation)
- **Verify 100-event receipt:** ~75ms (full 7-stage pipeline)
- **Memory (100 events):** ~500KB (typical)

### Optimization Strategies

- **Lazy chain hashing:** Recompute only on verify, not on emit
- **Streaming JSON:** For large receipts, stream decode via `serde_json::StreamDeserializer`
- **Parallel verification:** Stages 3–6 could run in parallel (stage 7 is terminal)

---

## Troubleshooting

### Receipt Fails at `chain_integrity` Stage

**Symptom:** `chain hash mismatch`

**Cause:** Receipt was tampered with (event field modified).

**Debug:**
```bash
affi inspect receipt.json --stage chain_integrity
affi diagnose receipt.json --suggest-fixes
```

### Event Rejected at Admission Gate

**Symptom:** `emit` returns "event rejected"

**Cause:** Event violates a validation rule (e.g., object ID malformed, commitment invalid).

**Fix:**
```bash
affi diagnose receipt.json --verbose
# Check object IDs are "id:type[:qualifier]"
# Check commitments are valid BLAKE3 digests (hex, 64 chars)
```

### Receipt Doesn't Verify in LSP Hover

**Symptom:** Hover on receipt path shows "failed to verify"

**Debug:**
```bash
affi verify receipt.json --format json
# Use JSON output for machine parsing; check first failure stage
```

---

## Roadmap & Future Work

- **Parallel verification:** Stages 3–6 as concurrent tasks
- **Streaming receipts:** Support event appends without re-finalization
- **Multi-profile validation:** Support multiple `core/vX` standards
- **Distributed chain:** Merkle-proof verification across shards
- **Web dashboard:** Visual receipt exploration

---

## License

MIT OR Apache-2.0

---

## References

- [BLAKE3](https://github.com/BLAKE3-team/BLAKE3) — Content addressing
- [Serde](https://serde.rs/) — Serialization framework
- [OpenTelemetry](https://opentelemetry.io/) — Observability standards
- [OCEL](https://www.ocel-standard.org/) — Object-Centric Event Logs
- [C4 Model](https://c4model.com/) — Architecture diagramming

---

**Last Updated:** 2026-06-22  
**Maintained by:** Sean Chatman (xpointsh@gmail.com)
