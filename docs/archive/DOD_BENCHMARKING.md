# Definition of Done — Benchmarking & Performance Regression Detection

**Initiative:** affidavit DX/QOL 1000x  
**Phase:** Benchmarking & Performance  
**Branch:** `claude/zen-cerf-oq87br`  
**Version:** 26.6.17  
**Last Updated:** 2026-06-14

---

## Table of Contents

1. [Overview & Doctrine](#overview--doctrine)
2. [Performance Target Table](#performance-target-table)
3. [Scaling Curve Expectations](#scaling-curve-expectations)
4. [Regression Detection Algorithm](#regression-detection-algorithm)
5. [Feature 1 — `affi bench receipt-throughput`](#feature-1--affi-bench-receipt-throughput)
6. [Feature 2 — `affi bench variance`](#feature-2--affi-bench-variance)
7. [Feature 3 — Criterion HTML Dashboard](#feature-3--criterion-html-dashboard)
8. [Feature 4 — `affi bench profile`](#feature-4--affi-bench-profile)
9. [Feature 5 — Baseline Comparisons & CI Integration](#feature-5--baseline-comparisons--ci-integration)
10. [Cross-Feature DoD Gates](#cross-feature-dod-gates)
11. [Flamegraph Interpretation Guide](#flamegraph-interpretation-guide)

---

## Overview & Doctrine

The benchmarking phase instruments the three core operations of the affidavit provenance pipeline — **emit**, **assemble**, and **verify** — with Criterion-driven benchmarks, automated regression gates, flamegraph profiling, and a styled HTML dashboard. The performance contracts in this document are derived from the CLAUDE.md baseline figures and constitute binding acceptance criteria.

**Guiding principle:** A benchmark that does not block merge on regression is advisory, not a contract. Every performance contract defined here must be wired into CI such that a pull request that violates the contract cannot merge.

**Immutability constraint:** Receipt hashing uses BLAKE3 and canonical JSON serialization. Benchmark harnesses must not bypass sealing (via `ChainAssembler::finalize`) or the admission layer. All measurements target the genuine code paths used in production.

---

## Performance Target Table

All latency targets are for release builds (`cargo bench` / `cargo build --release`). Debug builds are excluded from regression gating.

| Operation | Event Count | p50 target | p99 target | Max acceptable | Scaling class |
|-----------|-------------|-----------|-----------|----------------|---------------|
| `emit` (build + append) | 1 | ≤ 100 µs | ≤ 200 µs | 500 µs | O(1) |
| `emit` (build + append) | 5 | ≤ 100 µs | ≤ 200 µs | 500 µs | O(1) |
| `emit` (build + append) | 10 | ≤ 100 µs | ≤ 200 µs | 500 µs | O(1) |
| `emit` (build + append) | 50 | ≤ 100 µs | ≤ 200 µs | 500 µs | O(1) |
| `emit` (build + append) | 100 | ≤ 100 µs | ≤ 200 µs | 500 µs | O(1) |
| `assemble` (full chain) | 1 | ≤ 0.5 ms | ≤ 1 ms | 2 ms | O(n) |
| `assemble` (full chain) | 5 | ≤ 2.5 ms | ≤ 5 ms | 10 ms | O(n) |
| `assemble` (full chain) | 10 | ≤ 5 ms | ≤ 10 ms | 20 ms | O(n) |
| `assemble` (full chain) | 50 | ≤ 25 ms | ≤ 40 ms | 75 ms | O(n) |
| `assemble` (full chain) | 100 | ≤ 50 ms | ≤ 80 ms | 150 ms | O(n) |
| `verify` (7-stage pipeline) | 1 | ≤ 0.75 ms | ≤ 1.5 ms | 3 ms | O(n) |
| `verify` (7-stage pipeline) | 5 | ≤ 3.75 ms | ≤ 7 ms | 15 ms | O(n) |
| `verify` (7-stage pipeline) | 10 | ≤ 7.5 ms | ≤ 15 ms | 30 ms | O(n) |
| `verify` (7-stage pipeline) | 50 | ≤ 37.5 ms | ≤ 60 ms | 100 ms | O(n) |
| `verify` (7-stage pipeline) | 100 | ≤ 75 ms | ≤ 120 ms | 200 ms | O(n) |
| `recompute_chain` (hash only) | 100 | ≤ 30 ms | ≤ 50 ms | 100 ms | O(n) |
| `variance` (surprise metric) | 10 | ≤ 5 ms | ≤ 10 ms | 25 ms | O(n) |
| `profile` (flamegraph sample) | 100 | ≤ 150 ms | ≤ 250 ms | 500 ms | O(n) |

### Notes

- p50 and p99 are derived from Criterion's wall-time measurements over ≥ 100 iterations.
- "Max acceptable" is the hard gate: if p99 exceeds this threshold the CI job fails unconditionally, regardless of regression percentage.
- All targets assume no I/O (receipts held in memory). Disk benchmarks are advisory only.

---

## Scaling Curve Expectations

### Assemble: O(n) linear in event count

`ChainAssembler::append` folds each event into the running BLAKE3 hash in O(1); `finalize` is O(1) over the stored running hash. The total cost of assembling n events is therefore O(n), dominated by n BLAKE3 compressions and n canonical JSON serializations.

| Events | Expected assemble time | Acceptable range |
|--------|------------------------|-----------------|
| 1 | ~0.5 ms | 0.1 – 2 ms |
| 5 | ~2.5 ms | 0.5 – 10 ms |
| 10 | ~5 ms | 1 – 20 ms |
| 50 | ~25 ms | 5 – 75 ms |
| 100 | ~50 ms | 10 – 150 ms |
| 200 | ~100 ms | 20 – 300 ms (advisory) |
| 500 | ~250 ms | 50 – 750 ms (advisory) |

Linearity check: the ratio `assemble(100) / assemble(10)` must be in the range `[8, 12]` (i.e., within 20% of the ideal 10×). A ratio outside this band indicates super-linear growth and is a CI failure.

### Verify: O(n) linear in event count

The 7-stage pipeline traverses the event list once per relevant stage. Stages 3 (`chain_integrity`) and 4 (`continuity`) both iterate over all events; stage 3 additionally performs n BLAKE3 compressions via `recompute_chain`. Total verify cost is dominated by `recompute_chain`, making the pipeline O(n).

| Events | Expected verify time | Acceptable range |
|--------|---------------------|-----------------|
| 1 | ~0.75 ms | 0.1 – 3 ms |
| 5 | ~3.75 ms | 0.75 – 15 ms |
| 10 | ~7.5 ms | 1.5 – 30 ms |
| 50 | ~37.5 ms | 7.5 – 100 ms |
| 100 | ~75 ms | 15 – 200 ms |

Linearity check: the ratio `verify(100) / verify(10)` must be in the range `[8, 12]`.

### Emit: O(1) per event

A single `emit` call (build one event + append to assembler) must exhibit O(1) behavior. The benchmark measures single-event append latency across chain sizes of 1, 5, 10, 50, and 100 pre-populated events. The measured latency must not increase by more than 20% between the 1-event and 100-event baselines.

---

## Regression Detection Algorithm

### Algorithm Description

Criterion's built-in regression detection uses a statistical comparison against a stored baseline. The detection pipeline used in this project is:

1. **Baseline capture:** On the `main` branch (or any designated reference commit), run `cargo bench -- --save-baseline <name>` to persist mean and confidence intervals to `target/criterion/<bench>/<group>/base/`.

2. **Comparison run:** In CI for each PR, run `cargo bench -- --baseline <name>` to load the stored baseline and produce a comparison. Criterion outputs a structured comparison report per benchmark function.

3. **Threshold evaluation:** Parse Criterion's JSON output (stored in `target/criterion/<bench>/<group>/new/estimates.json`) and apply the following rules:

   | Change magnitude | Action |
   |-----------------|--------|
   | Improvement (any) | Log and continue; optionally update baseline |
   | Regression ≤ 5% | Warning annotation on the PR; merge allowed |
   | Regression 5–10% | Warning annotation + required human review sign-off |
   | Regression > 10% | CI job fails; merge is blocked |
   | p99 exceeds max acceptable | CI job fails unconditionally |

4. **Criterion confidence interval:** Criterion reports a 95% CI. A regression is only flagged if the lower bound of the CI for the new measurement exceeds the upper bound of the CI for the baseline (i.e., the regression is statistically unambiguous). Noise-induced fluctuations within overlapping CIs are ignored.

5. **Threshold script:** A Rust binary or shell script at `benches/check_regression.sh` parses the Criterion JSON estimates and exits 0 (pass) or 1 (fail) for CI consumption.

### Regression Thresholds — Summary

```
> 10% regression in any benchmarked function  → CI FAILURE (blocks merge)
  5%–10% regression                           → CI WARNING (requires sign-off)
≤  5% regression                              → CI PASS (logged only)
p99 > max_acceptable for any function         → CI FAILURE (unconditional)
linearity_ratio outside [8, 12]               → CI FAILURE for assemble/verify
```

### Baseline Storage

Baselines are stored as committed artifacts in the repository to allow deterministic comparison across PR builds:

```
benches/baselines/
├── receipt-throughput/      # baselines for throughput bench
├── variance/                # baselines for variance bench
└── profile/                 # baselines for profile bench
```

Each baseline directory mirrors the Criterion `base/` format:
- `estimates.json` — mean, median, std_dev, confidence_interval
- `sample.json` — raw sample points

CI writes new measurements to `target/criterion/` (gitignored) and compares against `benches/baselines/` (committed).

---

## Feature 1 — `affi bench receipt-throughput`

### Overview

A new Criterion benchmark file `benches/throughput.rs` measures the end-to-end latency of the emit → assemble → verify pipeline at 1, 5, 10, 50, and 100 events. This is the primary regression gate for the core operation chain.

### New Files

- `benches/throughput.rs` — Criterion benchmark groups for emit, assemble, verify, and the composite pipeline
- Entry in `Cargo.toml`: `[[bench]] name = "throughput" harness = false`

### Benchmark Function Signatures

The following Criterion group and function names are normative. CI scripts parse these exact names from `target/criterion/` paths.

```
throughput/emit_single_event
throughput/assemble_pipeline/1
throughput/assemble_pipeline/5
throughput/assemble_pipeline/10
throughput/assemble_pipeline/50
throughput/assemble_pipeline/100
throughput/verify_pipeline/1
throughput/verify_pipeline/5
throughput/verify_pipeline/10
throughput/verify_pipeline/50
throughput/verify_pipeline/100
throughput/full_pipeline/1
throughput/full_pipeline/5
throughput/full_pipeline/10
throughput/full_pipeline/50
throughput/full_pipeline/100
throughput/emit_scaling/1
throughput/emit_scaling/5
throughput/emit_scaling/10
throughput/emit_scaling/50
throughput/emit_scaling/100
```

### Benchmark Code Template

```rust
// benches/throughput.rs
//
// Criterion benchmarks for the emit → assemble → verify pipeline.
// Measures per-operation latency at 1, 5, 10, 50, and 100 events.
//
// Run: cargo bench --bench throughput
// Save baseline: cargo bench --bench throughput -- --save-baseline main
// Compare: cargo bench --bench throughput -- --baseline main

use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use affidavit::chain::ChainAssembler;
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::verifier::verify;

const EVENT_COUNTS: &[usize] = &[1, 5, 10, 50, 100];

/// Build a receipt with `n` events and return both the assembler and the receipt.
/// This helper is NOT benchmarked; it is setup work.
fn build_receipt(n: usize) -> affidavit::types::Receipt {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    for i in 0..n {
        let event = build_event(
            "throughput-op",
            vec![object_ref(&format!("obj-{i}"), "artifact")],
            format!("payload-{i}").as_bytes(),
            &mut counter,
        )
        .expect("build event");
        asm.append(event).expect("append");
    }
    asm.finalize()
}

/// Bench: single emit (build one event and append to a fresh assembler).
/// Must be O(1) — verified by emit_scaling group below.
fn bench_emit_single(c: &mut Criterion) {
    c.bench_function("throughput/emit_single_event", |b| {
        b.iter(|| {
            let mut asm = ChainAssembler::new();
            let mut counter = SeqCounter::new();
            let event = build_event(
                "emit-op",
                vec![object_ref("obj", "artifact")],
                black_box(b"payload"),
                &mut counter,
            )
            .expect("build event");
            asm.append(black_box(event)).expect("append");
            black_box(asm)
        })
    });
}

/// Bench: full assemble pipeline (build n events + finalize).
/// Must scale O(n) — linearity verified by CI script.
fn bench_assemble_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput/assemble_pipeline");
    for &n in EVENT_COUNTS {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                let mut asm = ChainAssembler::new();
                let mut counter = SeqCounter::new();
                for i in 0..n {
                    let event = build_event(
                        "assemble-op",
                        vec![object_ref(&format!("obj-{i}"), "artifact")],
                        black_box(format!("payload-{i}").as_bytes()),
                        &mut counter,
                    )
                    .expect("build event");
                    asm.append(event).expect("append");
                }
                black_box(asm.finalize())
            })
        });
    }
    group.finish();
}

/// Bench: 7-stage verify pipeline over a pre-assembled receipt.
/// Receipt construction is setup (not measured).
fn bench_verify_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput/verify_pipeline");
    for &n in EVENT_COUNTS {
        let receipt = build_receipt(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(verify(black_box(&receipt))))
        });
    }
    group.finish();
}

/// Bench: composite emit → assemble → verify pipeline, end-to-end.
fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput/full_pipeline");
    for &n in EVENT_COUNTS {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                let mut asm = ChainAssembler::new();
                let mut counter = SeqCounter::new();
                for i in 0..n {
                    let event = build_event(
                        "pipeline-op",
                        vec![object_ref(&format!("obj-{i}"), "artifact")],
                        black_box(format!("payload-{i}").as_bytes()),
                        &mut counter,
                    )
                    .expect("build event");
                    asm.append(event).expect("append");
                }
                let receipt = asm.finalize();
                black_box(verify(black_box(&receipt)))
            })
        });
    }
    group.finish();
}

/// Bench: single-event append into a chain of n pre-populated events.
/// Verifies O(1) per-event emit: latency must not grow with chain size.
fn bench_emit_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput/emit_scaling");
    for &n in EVENT_COUNTS {
        // Setup: build a chain of n events (not measured)
        let mut asm = ChainAssembler::new();
        let mut counter = SeqCounter::new();
        for i in 0..n {
            let event = build_event(
                "setup-op",
                vec![object_ref(&format!("setup-obj-{i}"), "artifact")],
                format!("setup-payload-{i}").as_bytes(),
                &mut counter,
            )
            .expect("build event");
            asm.append(event).expect("append");
        }

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter_batched(
                || {
                    // Clone the assembler for each iteration (setup, not measured)
                    (asm.clone(), SeqCounter::with_start(n as u64))
                },
                |(mut asm_clone, mut counter_clone)| {
                    // Measure only the single append
                    let event = build_event(
                        "scaling-op",
                        vec![object_ref("new-obj", "artifact")],
                        black_box(b"new-payload"),
                        &mut counter_clone,
                    )
                    .expect("build event");
                    asm_clone.append(black_box(event)).expect("append");
                    black_box(asm_clone)
                },
                criterion::BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

criterion_group!(
    throughput_benches,
    bench_emit_single,
    bench_assemble_pipeline,
    bench_verify_pipeline,
    bench_full_pipeline,
    bench_emit_scaling,
);
criterion_main!(throughput_benches);
```

### Acceptance Criteria

**Given** a release build of affidavit on the `claude/zen-cerf-oq87br` branch,  
**When** `cargo bench --bench throughput` completes,  
**Then** Criterion produces HTML reports under `target/criterion/throughput/` for all 20 named benchmark functions without error.

---

**Given** the `throughput/emit_single_event` benchmark runs 100+ iterations,  
**When** the Criterion mean is computed,  
**Then** the measured mean latency is ≤ 100 µs and the p99 is ≤ 200 µs.

---

**Given** the `throughput/assemble_pipeline/100` benchmark runs,  
**When** the Criterion mean is computed,  
**Then** the measured mean latency is ≤ 50 ms and the p99 is ≤ 80 ms.

---

**Given** the `throughput/verify_pipeline/100` benchmark runs,  
**When** the Criterion mean is computed,  
**Then** the measured mean latency is ≤ 75 ms and the p99 is ≤ 120 ms.

---

**Given** the `throughput/emit_scaling` group measures single-append latency at chain sizes 1, 5, 10, 50, and 100,  
**When** the ratio of `emit_scaling/100` mean to `emit_scaling/1` mean is computed,  
**Then** the ratio is in the range `[0.5, 1.5]`, confirming O(1) append behavior.

---

**Given** the `throughput/assemble_pipeline` group measures full-assembly latency at sizes 1, 5, 10, 50, and 100,  
**When** the ratio of `assemble_pipeline/100` mean to `assemble_pipeline/10` mean is computed,  
**Then** the ratio is in the range `[8, 12]`, confirming O(n) linear scaling.

---

**Given** the `throughput/verify_pipeline` group measures verify latency at sizes 1, 5, 10, 50, and 100,  
**When** the ratio of `verify_pipeline/100` mean to `verify_pipeline/10` mean is computed,  
**Then** the ratio is in the range `[8, 12]`, confirming O(n) linear scaling.

---

**Given** a stored baseline from the `main` branch and a PR branch that introduces a change,  
**When** `cargo bench --bench throughput -- --baseline main` runs in CI,  
**Then** any benchmark function showing > 10% regression causes the CI job to exit non-zero.

---

**Given** any benchmark function in the throughput suite,  
**When** the p99 latency exceeds the "max acceptable" value in the Performance Target Table,  
**Then** the CI regression check script exits non-zero, blocking merge regardless of comparison to baseline.

---

**Given** the full pipeline benchmark at 100 events,  
**When** measured on a fresh clone with no prior baselines,  
**Then** the absolute wall-time mean is ≤ 200 ms (combined emit + assemble + verify budget).

### Performance Contracts

| Contract | Value | Enforcement |
|----------|-------|-------------|
| Emit p50 | ≤ 100 µs | CI failure if exceeded |
| Emit p99 | ≤ 200 µs | CI failure if exceeded |
| Assemble-100 p50 | ≤ 50 ms | CI failure if exceeded |
| Assemble-100 p99 | ≤ 80 ms | CI failure if exceeded |
| Verify-100 p50 | ≤ 75 ms | CI failure if exceeded |
| Verify-100 p99 | ≤ 120 ms | CI failure if exceeded |
| Emit O(1) ratio (100/1) | [0.5, 1.5] | CI failure if outside range |
| Assemble O(n) ratio (100/10) | [8, 12] | CI failure if outside range |
| Verify O(n) ratio (100/10) | [8, 12] | CI failure if outside range |

### Scaling Constraints

- **Emit:** O(1) — `fold_event` folds a single event into the running BLAKE3 hash in constant time. The append operation must not exhibit growth proportional to chain length.
- **Assemble:** O(n) — n BLAKE3 compressions + n canonical JSON serializations; `finalize` is O(1) after the last append.
- **Verify:** O(n) — `recompute_chain` iterates all events once; stages 4 and 5 each iterate once. Total: 3n operations.

---

## Feature 2 — `affi bench variance`

### Overview

A new Criterion benchmark file `benches/variance.rs` measures **control-flow surprise**: the degree to which a receipt's event ordering deviates from the expected process model discovered by the `wasm4pm` variance module. The surprise metric is a 0–1 float, where 0 means the observed sequence perfectly matches the discovered model and 1 means maximum deviation.

This benchmark captures the cost of the discover-then-conform pipeline invoked by `affi bench variance`, including the `quality_metrics` call chain through `wasm4pm::ilp_discovery`.

### New Files

- `benches/variance.rs` — Criterion benchmark groups for control-flow surprise measurement
- Entry in `Cargo.toml`: `[[bench]] name = "variance" harness = false`

### Benchmark Function Signatures

```
variance/surprise_sequential/1
variance/surprise_sequential/5
variance/surprise_sequential/10
variance/surprise_sequential/50
variance/surprise_sequential/100
variance/surprise_interleaved/5
variance/surprise_interleaved/10
variance/surprise_interleaved/50
variance/quality_metrics_pipeline/4
variance/quality_metrics_pipeline/10
variance/dfg_discovery/5
variance/dfg_discovery/10
variance/dfg_discovery/50
```

### Surprise Metric Definition

The surprise metric `S(receipt)` is derived from the `quality_metrics` pipeline:

```
S(receipt) = 1.0 - fitness
```

Where `fitness` is the token-replay fitness returned by `wasm4pm::ilp_discovery::discover_ilp_petri_net_from_log`. A receipt whose events perfectly follow the discovered model yields `fitness ≈ 1.0` and `S ≈ 0.0`. A receipt with maximally anomalous ordering yields `fitness ≈ 0.0` and `S ≈ 1.0`.

**Invariant:** `S(receipt)` ∈ `[0.0, 1.0]` for any valid receipt. A value outside this range is a bug in the wasm4pm integration and must cause the benchmark run to panic.

### Benchmark Code Template

```rust
// benches/variance.rs
//
// Criterion benchmarks for control-flow surprise (wasm4pm variance module).
// Measures the cost of discover-then-conform and the 0–1 surprise metric.
//
// Run: cargo bench --bench variance
// Save baseline: cargo bench --bench variance -- --save-baseline main
// Compare: cargo bench --bench variance -- --baseline main

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use affidavit::chain::ChainAssembler;
use affidavit::discovery::{conformance_metrics, discover_dfg_summary, quality_metrics};
use affidavit::ocel::{build_event, object_ref, SeqCounter};

const EVENT_COUNTS: &[usize] = &[1, 5, 10, 50, 100];

/// Activity sequences for sequential (predictable) receipts.
const SEQUENTIAL_ACTIVITIES: &[&str] =
    &["create", "transform", "validate", "audit", "release",
      "archive", "review", "approve", "sign", "certify"];

/// Activity sequences for interleaved (higher-surprise) receipts.
/// Interleaving events that wouldn't normally co-occur drives higher surprise scores.
const INTERLEAVED_ACTIVITIES: &[&str] =
    &["release", "create", "certify", "transform", "archive",
      "sign", "validate", "approve", "audit", "review"];

fn build_receipt_sequential(n: usize) -> affidavit::types::Receipt {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    for i in 0..n {
        let act = SEQUENTIAL_ACTIVITIES[i % SEQUENTIAL_ACTIVITIES.len()];
        let event = build_event(
            act,
            vec![object_ref(&format!("obj-{i}"), "artifact")],
            format!("payload-{i}").as_bytes(),
            &mut counter,
        )
        .expect("build sequential event");
        asm.append(event).expect("append");
    }
    asm.finalize()
}

fn build_receipt_interleaved(n: usize) -> affidavit::types::Receipt {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    for i in 0..n {
        let act = INTERLEAVED_ACTIVITIES[i % INTERLEAVED_ACTIVITIES.len()];
        let event = build_event(
            act,
            vec![object_ref(&format!("iobj-{i}"), "artifact")],
            format!("ipayload-{i}").as_bytes(),
            &mut counter,
        )
        .expect("build interleaved event");
        asm.append(event).expect("append");
    }
    asm.finalize()
}

/// Compute the 0–1 surprise metric for a receipt.
/// Panics if the result is outside [0, 1] — that is a bug.
fn surprise(receipt: &affidavit::types::Receipt) -> f64 {
    let (fitness, _activity_coverage) = conformance_metrics(receipt);
    assert!(
        (0.0..=1.0).contains(&fitness),
        "fitness from wasm4pm out of range: {fitness}"
    );
    1.0 - fitness
}

/// Bench: surprise metric for sequential receipts of increasing size.
fn bench_surprise_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("variance/surprise_sequential");
    for &n in EVENT_COUNTS {
        let receipt = build_receipt_sequential(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(surprise(black_box(&receipt))))
        });
    }
    group.finish();
}

/// Bench: surprise metric for interleaved (higher-variance) receipts.
fn bench_surprise_interleaved(c: &mut Criterion) {
    let mut group = c.benchmark_group("variance/surprise_interleaved");
    for &n in [5_usize, 10, 50] {
        let receipt = build_receipt_interleaved(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(surprise(black_box(&receipt))))
        });
    }
    group.finish();
}

/// Bench: full quality_metrics pipeline (fitness + activity_coverage + simplicity).
fn bench_quality_metrics_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("variance/quality_metrics_pipeline");
    for &n in &[4_usize, 10] {
        let receipt = build_receipt_sequential(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let (f, p, s) = quality_metrics(black_box(&receipt));
                black_box((f, p, s))
            })
        });
    }
    group.finish();
}

/// Bench: DFG discovery from receipts of increasing size.
fn bench_dfg_discovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("variance/dfg_discovery");
    for &n in &[5_usize, 10, 50] {
        let receipt = build_receipt_sequential(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(discover_dfg_summary(black_box(&receipt))))
        });
    }
    group.finish();
}

criterion_group!(
    variance_benches,
    bench_surprise_sequential,
    bench_surprise_interleaved,
    bench_quality_metrics_pipeline,
    bench_dfg_discovery,
);
criterion_main!(variance_benches);
```

### Acceptance Criteria

**Given** a receipt with n events whose activities follow the sequential pattern,  
**When** the surprise metric `S(receipt)` is computed via the variance bench,  
**Then** the result is in `[0.0, 1.0]` and the computation completes within the latency budget in the Performance Target Table.

---

**Given** a receipt with n events whose activities follow the interleaved (anomalous) pattern,  
**When** the surprise metric is computed,  
**Then** the interleaved surprise score is greater than or equal to the sequential surprise score for the same n, reflecting that interleaved orderings produce higher control-flow anomaly.

---

**Given** the `variance/surprise_sequential/10` benchmark,  
**When** it runs at least 50 Criterion iterations,  
**Then** the mean latency is ≤ 5 ms and p99 is ≤ 10 ms.

---

**Given** the `variance/quality_metrics_pipeline/4` benchmark,  
**When** it runs,  
**Then** all three returned values (`fitness`, `activity_coverage`, `simplicity`) are within `[0.0, 1.0]` (validated in the harness, not just the bench).

---

**Given** a stored baseline and a PR that modifies `discovery.rs` or the `wasm4pm` integration,  
**When** `cargo bench --bench variance -- --baseline main` runs,  
**Then** any regression > 10% in any variance benchmark function blocks merge.

---

**Given** `bench_surprise_sequential` runs at n = 1, 5, 10, 50, and 100,  
**When** the ratio of `surprise_sequential/100` mean to `surprise_sequential/10` mean is computed,  
**Then** the ratio is ≤ 15 (allowing for wasm4pm ILP solver overhead, which may not be strictly linear).

---

**Given** a receipt produced with zero events,  
**When** surprise is computed,  
**Then** the function does not panic; it returns `1.0 - fitness` where fitness for an empty log is defined by the wasm4pm engine's behavior.

---

**Given** the variance bench suite runs in CI,  
**When** any benchmark function panics (e.g., wasm4pm returns fitness outside `[0, 1]`),  
**Then** the CI job fails and the panic message is surfaced in the job log.

---

**Given** the `variance/dfg_discovery` group runs at n = 5, 10, 50,  
**When** DFG results are computed,  
**Then** the number of DFG nodes equals the number of distinct `event_type` values in the receipt.

### Performance Contracts

| Contract | Value | Enforcement |
|----------|-------|-------------|
| Surprise metric range | [0.0, 1.0] | Panic in harness if violated |
| Surprise at n=10 p50 | ≤ 5 ms | CI warning if exceeded |
| Surprise at n=10 p99 | ≤ 10 ms | CI failure if exceeded |
| Interleaved ≥ Sequential surprise | For same n | Assert in acceptance test |
| Regression threshold | > 10% = CI failure | Baseline comparison in CI |

### Scaling Constraints

The `wasm4pm` ILP solver does not provide a strict O(n) guarantee for arbitrarily large n. For the event counts exercised in this benchmark (1–100), the practical scaling is expected to be sub-quadratic. The linearity ratio constraint is relaxed to `≤ 15×` for the variance suite (vs. `[8, 12]` for the throughput suite).

---

## Feature 3 — Criterion HTML Dashboard

### Overview

A custom CSS theme in `benches/theme/` overrides Criterion's default report styling to match the affidavit visual identity (monochrome, BLAKE3-green accent, monospace throughout). A markdown summary generator script produces a `BENCH_SUMMARY.md` from the latest Criterion JSON estimates for use in PR descriptions and release notes.

### New Files

- `benches/theme/criterion.css` — Custom CSS overriding Criterion's default styles
- `benches/theme/README.md` — Instructions for applying the theme
- `benches/gen_summary.sh` — Shell script that reads `target/criterion/` JSON and writes `BENCH_SUMMARY.md`
- `benches/gen_summary.py` — Python alternative to `gen_summary.sh` for environments with Python 3.x

### Dashboard Spec

The Criterion HTML dashboard at `target/criterion/report/index.html` must display the following panels after `cargo bench` completes:

| Panel | Metric displayed | Required for feature completion |
|-------|-----------------|--------------------------------|
| Throughput overview | Grouped bar chart: assemble and verify at 1/5/10/50/100 events | Yes |
| Regression timeline | Line chart showing mean latency per function across baseline comparisons | Yes |
| Variance surprise | Bar chart comparing sequential vs. interleaved surprise scores | Yes |
| Flamegraph link | Hyperlink to most recent `target/flamegraph.svg` | Yes |
| Scaling curve | Scatter plot: event count (x) vs. latency (y) for assemble and verify | Yes |
| CI status badge | Text indicator: PASS / WARN / FAIL based on last regression check | Yes |
| Per-function summary | Table: function name, mean, p99, baseline delta, status | Yes |

### Custom CSS Theme Spec

The custom theme in `benches/theme/criterion.css` must:

1. Override the background color to `#0d1117` (GitHub dark).
2. Set the primary accent to `#00c853` (BLAKE3-green, evoking hash success).
3. Use `font-family: 'JetBrains Mono', 'Fira Code', monospace` throughout.
4. Set regression bars to `#ff5252` (red) and improvement bars to `#00c853` (green).
5. Set warning bars to `#ffd740` (amber).
6. Apply a `border-left: 3px solid #00c853` to the receipt-chain metaphor elements.
7. Scale the KDE plot to fill the panel width with no horizontal scrolling at 1280px viewport.

### Applying the Theme

Criterion's HTML output is self-contained; the custom CSS must be injected post-generation. The `benches/gen_summary.sh` script includes a step that copies `benches/theme/criterion.css` into `target/criterion/` and patches the `<link>` tags in the generated HTML files.

### Markdown Summary Generator Spec

`benches/gen_summary.sh` must produce `BENCH_SUMMARY.md` with the following structure:

```markdown
## Benchmark Summary — affidavit vX.Y.Z

**Run date:** YYYY-MM-DD  
**Branch:** <branch>  
**Baseline:** <baseline name or "none">

### Throughput

| Benchmark | Mean | p99 | vs. Baseline | Status |
|-----------|------|-----|-------------|--------|
| ...       | ...  | ... | ...         | PASS   |

### Variance (Control-Flow Surprise)

| Benchmark | Mean | p99 | Surprise score | vs. Baseline |
|-----------|------|-----|---------------|-------------|

### Profile Hot Functions

| Function | % CPU | Bench group |
|----------|-------|-------------|

### Regression Gate Result

Overall: PASS / WARN / FAIL
```

### Acceptance Criteria

**Given** `cargo bench` completes successfully,  
**When** `benches/gen_summary.sh` is run,  
**Then** `BENCH_SUMMARY.md` is created in the repository root with all four required sections populated.

---

**Given** `benches/theme/criterion.css` exists,  
**When** `benches/gen_summary.sh` runs the post-processing step,  
**Then** all HTML files under `target/criterion/` reference the custom CSS, and the `#0d1117` background color is present in at least the `report/index.html` file.

---

**Given** the dashboard opens in a browser at 1280px width,  
**When** all panels are rendered,  
**Then** no horizontal scrollbar appears on any panel, and the scaling curve scatter plot is legible without zoom.

---

**Given** a baseline comparison has been run,  
**When** the dashboard is viewed,  
**Then** the regression timeline panel shows at least two data points (baseline and current), with the delta expressed as a percentage and color-coded green (improvement), amber (warn), or red (regression).

---

**Given** the summary generator runs in CI with no prior baseline,  
**When** `BENCH_SUMMARY.md` is generated,  
**Then** the "vs. Baseline" column shows "N/A (no baseline)" and the "Status" column shows "UNCHECKED" rather than erroring.

---

**Given** a flamegraph SVG has been generated by the profile bench,  
**When** the dashboard HTML is generated,  
**Then** the Flamegraph link panel contains an `<a href="...">` pointing to a relative path that resolves to `target/flamegraph.svg`.

---

**Given** the dashboard is regenerated after a PR that improves `verify` latency by 15%,  
**When** the regression timeline panel is viewed,  
**Then** the `verify_pipeline/100` entry shows a green delta of approximately `-15%`.

---

**Given** the custom CSS theme is applied,  
**When** the Criterion KDE plot for `assemble_pipeline/100` is rendered,  
**Then** the distribution curve is rendered in `#00c853` and the x-axis label reads "Time (ms)" with monospace font.

---

**Given** the summary generator runs after a bench with a > 10% regression,  
**When** `BENCH_SUMMARY.md` is generated,  
**Then** the "Regression Gate Result" section shows `FAIL` in bold, and the failing function names are listed with their regression percentages.

### Dashboard Panels — Detailed Metric Specification

| Panel | Source data | Criterion JSON path | Refresh trigger |
|-------|-------------|--------------------|-----------------|
| Throughput overview | Mean latency per bench function | `target/criterion/<bench>/<group>/<n>/new/estimates.json` → `mean.point_estimate` | `cargo bench` |
| Regression timeline | Delta vs. baseline | `target/criterion/<bench>/<group>/<n>/change/estimates.json` → `mean.point_estimate` | `--baseline` run |
| Variance surprise | Surprise score (1 - fitness) | Computed from `quality_metrics_pipeline` bench output | `cargo bench --bench variance` |
| Flamegraph link | SVG path | `target/flamegraph.svg` | `cargo flamegraph` |
| Scaling curve | (n, mean) pairs | All `assemble_pipeline` and `verify_pipeline` groups | `cargo bench` |
| Per-function table | All bench functions | `target/criterion/*/*/new/estimates.json` | `cargo bench` |

---

## Feature 4 — `affi bench profile`

### Overview

A new Criterion benchmark file `benches/profile.rs` provides a profiling harness designed for use with `cargo-flamegraph`. The harness runs the verify pipeline at 100 events for a sustained duration (≥ 30 seconds of CPU time) to produce a statistically meaningful flamegraph. Hot functions expected in the flamegraph are `blake3`, `serde_json`, and `canonical_bytes`.

### New Files

- `benches/profile.rs` — Profiling harness with long-duration bench for flamegraph capture
- Entry in `Cargo.toml`: `[[bench]] name = "profile" harness = false`
- `benches/profile.sh` — Shell script wrapping `cargo flamegraph` with the correct bench target

### Benchmark Function Signatures

```
profile/verify_hot_path/100
profile/chain_recompute_hot_path/100
profile/assemble_hot_path/100
profile/canonical_bytes_hot_path/100
profile/blake3_hash_hot_path/100
```

### Benchmark Code Template

```rust
// benches/profile.rs
//
// Profiling harness for cargo-flamegraph.
// Designed for long-duration runs that produce meaningful flamegraphs.
//
// Usage:
//   # Flamegraph (requires cargo-flamegraph and perf):
//   cargo flamegraph --bench profile -- profile/verify_hot_path/100
//
//   # Standard Criterion run (shorter):
//   cargo bench --bench profile
//
//   # Save baseline:
//   cargo bench --bench profile -- --save-baseline main

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;
use affidavit::chain::{recompute_chain, ChainAssembler};
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::{canonical_bytes, Blake3Hash};
use affidavit::verifier::verify;

/// Profile target event count — large enough for meaningful flamegraph samples.
const PROFILE_N: usize = 100;

fn build_profile_receipt(n: usize) -> affidavit::types::Receipt {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    for i in 0..n {
        let event = build_event(
            "profile-op",
            vec![object_ref(&format!("obj-{i}"), "artifact")],
            format!("profile-payload-{i}").as_bytes(),
            &mut counter,
        )
        .expect("build profile event");
        asm.append(event).expect("append");
    }
    asm.finalize()
}

/// Profile bench: the full 7-stage verify pipeline.
/// Expected hot functions: blake3 (via recompute_chain), serde_json (via canonical_bytes).
fn bench_verify_hot_path(c: &mut Criterion) {
    let receipt = build_profile_receipt(PROFILE_N);
    let mut group = c.benchmark_group("profile/verify_hot_path");

    // Longer measurement time for better flamegraph coverage.
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(200);

    group.bench_with_input(
        BenchmarkId::from_parameter(PROFILE_N),
        &PROFILE_N,
        |b, _| b.iter(|| black_box(verify(black_box(&receipt)))),
    );
    group.finish();
}

/// Profile bench: the chain recompute path (the BLAKE3 inner loop).
/// Expected hot functions: blake3::hash, affidavit::chain::fold_event.
fn bench_chain_recompute_hot_path(c: &mut Criterion) {
    let receipt = build_profile_receipt(PROFILE_N);
    let mut group = c.benchmark_group("profile/chain_recompute_hot_path");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(200);
    group.bench_with_input(
        BenchmarkId::from_parameter(PROFILE_N),
        &PROFILE_N,
        |b, _| b.iter(|| black_box(recompute_chain(black_box(&receipt.events)))),
    );
    group.finish();
}

/// Profile bench: full assemble pipeline.
/// Expected hot functions: canonical_bytes (serde_json), blake3::hash.
fn bench_assemble_hot_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("profile/assemble_hot_path");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);
    group.bench_with_input(
        BenchmarkId::from_parameter(PROFILE_N),
        &PROFILE_N,
        |b, &n| {
            b.iter(|| {
                let mut asm = ChainAssembler::new();
                let mut counter = SeqCounter::new();
                for i in 0..n {
                    let event = build_event(
                        "assemble-profile-op",
                        vec![object_ref(&format!("aobj-{i}"), "artifact")],
                        black_box(format!("apayload-{i}").as_bytes()),
                        &mut counter,
                    )
                    .expect("build event");
                    asm.append(event).expect("append");
                }
                black_box(asm.finalize())
            })
        },
    );
    group.finish();
}

/// Profile bench: canonical JSON serialization in isolation.
/// Isolates serde_json cost from blake3 cost for flamegraph attribution.
fn bench_canonical_bytes_hot_path(c: &mut Criterion) {
    let receipt = build_profile_receipt(PROFILE_N);
    let mut group = c.benchmark_group("profile/canonical_bytes_hot_path");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(200);
    group.bench_with_input(
        BenchmarkId::from_parameter(PROFILE_N),
        &PROFILE_N,
        |b, _| b.iter(|| black_box(canonical_bytes(black_box(&receipt)))),
    );
    group.finish();
}

/// Profile bench: BLAKE3 hashing in isolation.
/// Isolates the hash function cost from JSON serialization cost.
fn bench_blake3_hash_hot_path(c: &mut Criterion) {
    // Pre-compute canonical bytes so we isolate only the BLAKE3 call.
    let receipt = build_profile_receipt(PROFILE_N);
    let bytes = canonical_bytes(&receipt).expect("canonical bytes");
    let mut group = c.benchmark_group("profile/blake3_hash_hot_path");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(500);
    group.bench_with_input(
        BenchmarkId::from_parameter(PROFILE_N),
        &PROFILE_N,
        |b, _| b.iter(|| black_box(Blake3Hash::from_bytes(black_box(&bytes)))),
    );
    group.finish();
}

criterion_group!(
    profile_benches,
    bench_verify_hot_path,
    bench_chain_recompute_hot_path,
    bench_assemble_hot_path,
    bench_canonical_bytes_hot_path,
    bench_blake3_hash_hot_path,
);
criterion_main!(profile_benches);
```

### Flamegraph Requirements

The flamegraph generated from `cargo flamegraph --bench profile -- profile/verify_hot_path/100` must satisfy the following:

| Requirement | Threshold | Rationale |
|-------------|-----------|-----------|
| `blake3` family appears in flamegraph | ≥ 20% of samples | `recompute_chain` performs n BLAKE3 compressions |
| `serde_json` appears in flamegraph | ≥ 10% of samples | `canonical_bytes` serializes each event to JSON |
| `affidavit::chain::fold_event` appears | ≥ 15% of samples | Inner loop of chain computation |
| `affidavit::verifier::verify` is root | Appears as root frame | The bench calls `verify` directly |
| No I/O-related frames in top 20% | No `std::fs` frames | Bench operates on in-memory receipts |
| Sample count | ≥ 10,000 samples | Sufficient for statistical attribution |

**Expected hot path in flamegraph (depth order):**

```
[bench iter]
└─ affidavit::verifier::verify
   ├─ affidavit::chain::recompute_chain        (largest slice)
   │  └─ affidavit::chain::fold_event
   │     ├─ affidavit::types::canonical_bytes
   │     │  └─ serde_json::to_value / serde_json::to_vec
   │     └─ blake3::hash
   │        └─ blake3::Hasher::finalize
   ├─ affidavit::verifier::stage_continuity    (BTreeSet operations)
   └─ affidavit::verifier::stage_verify_commitments
```

### `benches/profile.sh` Script

```bash
#!/usr/bin/env bash
# benches/profile.sh — Generate flamegraph for the verify hot path.
# Requires: cargo-flamegraph, Linux perf (or dtrace on macOS)
#
# Usage: bash benches/profile.sh [bench_function]
# Default bench: profile/verify_hot_path/100

set -euo pipefail

BENCH_FN="${1:-profile/verify_hot_path/100}"
OUTPUT="target/flamegraph.svg"

echo "Generating flamegraph for: $BENCH_FN"
echo "Output: $OUTPUT"

cargo flamegraph \
  --bench profile \
  --output "$OUTPUT" \
  -- "$BENCH_FN"

echo "Flamegraph written to $OUTPUT"
echo "Open with: xdg-open $OUTPUT  (Linux) or open $OUTPUT  (macOS)"
```

### Acceptance Criteria

**Given** `benches/profile.rs` is compiled as part of `cargo bench`,  
**When** `cargo bench --bench profile` runs,  
**Then** all five profile bench groups complete without error and produce Criterion HTML reports.

---

**Given** `cargo-flamegraph` is installed and Linux `perf` is available,  
**When** `bash benches/profile.sh` runs,  
**Then** `target/flamegraph.svg` is created and contains SVG content with frame labels mentioning `blake3`, `serde_json`, and `affidavit`.

---

**Given** the flamegraph SVG is opened in a browser,  
**When** the frame for `blake3::hash` or `blake3::Hasher::finalize` is clicked,  
**Then** the frame is highlighted and accounts for ≥ 20% of total CPU samples.

---

**Given** the profile bench runs `bench_canonical_bytes_hot_path` in isolation,  
**When** the flamegraph is captured for that function alone,  
**Then** `serde_json` frames account for ≥ 50% of that bench's samples (since BLAKE3 is excluded from that bench).

---

**Given** the profile bench runs `bench_blake3_hash_hot_path` in isolation,  
**When** the flamegraph is captured,  
**Then** the `blake3` frames account for ≥ 80% of that bench's samples.

---

**Given** the `profile/verify_hot_path/100` benchmark runs with `measurement_time = 15s`,  
**When** Criterion reports sample count,  
**Then** at least 100 samples are collected (Criterion minimum) and the SVG has ≥ 10,000 perf samples.

---

**Given** a stored baseline and a PR that modifies `src/chain.rs` or `src/verifier.rs`,  
**When** `cargo bench --bench profile -- --baseline main` runs,  
**Then** any regression > 10% in `profile/verify_hot_path/100` blocks merge.

---

**Given** the `profile/chain_recompute_hot_path/100` bench runs,  
**When** its mean latency is compared to `throughput/verify_pipeline/100` mean minus `throughput/assemble_pipeline/100` mean,  
**Then** the `chain_recompute` bench accounts for ≥ 40% of `verify` latency (chain recomputation dominates verification cost).

### Performance Contracts

| Contract | Value | Enforcement |
|----------|-------|-------------|
| `verify_hot_path/100` p50 | ≤ 150 ms | CI failure |
| `verify_hot_path/100` p99 | ≤ 250 ms | CI failure |
| `blake3` frames in flamegraph | ≥ 20% | Manual verification on PR |
| `serde_json` frames in flamegraph | ≥ 10% | Manual verification on PR |
| Regression threshold | > 10% = CI failure | Baseline comparison |

---

## Feature 5 — Baseline Comparisons & CI Integration

### Overview

Criterion baselines for all three benchmark suites (`throughput`, `variance`, `profile`) are captured on the `main` branch and committed to `benches/baselines/`. CI runs on every pull request perform a baseline comparison and fail the job if any benchmark regresses by more than 10%. Warnings are emitted for 5–10% regressions.

### New Files

- `benches/baselines/throughput/` — Criterion baseline JSON for throughput bench
- `benches/baselines/variance/` — Criterion baseline JSON for variance bench
- `benches/baselines/profile/` — Criterion baseline JSON for profile bench
- `benches/check_regression.sh` — Regression gate script (parses Criterion JSON, exits 0/1)
- `.github/workflows/bench.yml` — CI workflow for benchmark regression detection
- `.cargo/config.toml` (updated) — Bench target configuration

### CI Integration

#### Exact `cargo bench` Commands

```bash
# Capture baseline (run on main branch before merging)
cargo bench --bench throughput -- --save-baseline main
cargo bench --bench variance  -- --save-baseline main
cargo bench --bench profile   -- --save-baseline main

# Copy baselines to committed directory
cp -r target/criterion/throughput/*/base/ benches/baselines/throughput/
cp -r target/criterion/variance/*/base/   benches/baselines/variance/
cp -r target/criterion/profile/*/base/    benches/baselines/profile/

# PR comparison run (in CI)
cargo bench --bench throughput -- --baseline main 2>&1 | tee bench_throughput.log
cargo bench --bench variance   -- --baseline main 2>&1 | tee bench_variance.log
cargo bench --bench profile    -- --baseline main 2>&1 | tee bench_profile.log

# Regression gate
bash benches/check_regression.sh target/criterion/ benches/baselines/
```

#### Baseline Storage Path Convention

```
benches/baselines/
├── throughput/
│   ├── emit_single_event/
│   │   └── base/
│   │       ├── estimates.json
│   │       └── sample.json
│   ├── assemble_pipeline/
│   │   ├── 1/base/estimates.json
│   │   ├── 5/base/estimates.json
│   │   ├── 10/base/estimates.json
│   │   ├── 50/base/estimates.json
│   │   └── 100/base/estimates.json
│   ├── verify_pipeline/
│   │   └── <same structure as assemble_pipeline>
│   ├── full_pipeline/
│   │   └── <same structure>
│   └── emit_scaling/
│       └── <same structure>
├── variance/
│   ├── surprise_sequential/
│   │   └── <1, 5, 10, 50, 100>/base/
│   ├── surprise_interleaved/
│   │   └── <5, 10, 50>/base/
│   ├── quality_metrics_pipeline/
│   │   └── <4, 10>/base/
│   └── dfg_discovery/
│       └── <5, 10, 50>/base/
└── profile/
    ├── verify_hot_path/
    │   └── 100/base/
    ├── chain_recompute_hot_path/
    │   └── 100/base/
    ├── assemble_hot_path/
    │   └── 100/base/
    ├── canonical_bytes_hot_path/
    │   └── 100/base/
    └── blake3_hash_hot_path/
        └── 100/base/
```

#### `benches/check_regression.sh` Script

```bash
#!/usr/bin/env bash
# benches/check_regression.sh
# Parses Criterion change/estimates.json files and applies regression thresholds.
#
# Exit codes:
#   0 — all benchmarks pass (no regression > 10%)
#   1 — one or more benchmarks regressed > 10% (CI failure)
#
# Usage: bash benches/check_regression.sh <criterion_dir> <baselines_dir>

set -euo pipefail

CRITERION_DIR="${1:-target/criterion}"
WARN_THRESHOLD=0.05    # 5% regression → warning
FAIL_THRESHOLD=0.10    # 10% regression → failure

overall_status=0
warnings=0
failures=0

echo "=== Criterion Regression Check ==="
echo "Criterion dir: $CRITERION_DIR"
echo "Warn threshold: ${WARN_THRESHOLD} (5%)"
echo "Fail threshold: ${FAIL_THRESHOLD} (10%)"
echo ""

# Find all change/estimates.json files produced by --baseline comparison
while IFS= read -r estimates_file; do
    # Extract the bench group path from the file path
    rel_path="${estimates_file#$CRITERION_DIR/}"
    group=$(echo "$rel_path" | sed 's|/change/estimates.json||')

    # Parse mean.point_estimate from the change estimates
    # Criterion stores the relative change as a fraction (e.g., 0.12 = +12%)
    if ! change=$(python3 -c "
import json, sys
with open('$estimates_file') as f:
    d = json.load(f)
# 'mean' key holds ConfidenceInterval with point_estimate
mean_change = d.get('mean', {}).get('point_estimate', 0.0)
print(f'{mean_change:.6f}')
" 2>/dev/null); then
        echo "SKIP $group (could not parse estimates)"
        continue
    fi

    # Determine status
    if python3 -c "import sys; sys.exit(0 if float('$change') > $FAIL_THRESHOLD else 1)" 2>/dev/null; then
        echo "FAIL  $group  (+$(python3 -c "print(f'{float(\"$change\")*100:.1f}')") %)"
        failures=$((failures + 1))
        overall_status=1
    elif python3 -c "import sys; sys.exit(0 if float('$change') > $WARN_THRESHOLD else 1)" 2>/dev/null; then
        echo "WARN  $group  (+$(python3 -c "print(f'{float(\"$change\")*100:.1f}')") %)"
        warnings=$((warnings + 1))
    else
        echo "PASS  $group"
    fi
done < <(find "$CRITERION_DIR" -name "estimates.json" -path "*/change/*" 2>/dev/null)

echo ""
echo "=== Summary ==="
echo "Failures (>10% regression): $failures"
echo "Warnings (5-10% regression): $warnings"

if [ "$overall_status" -ne 0 ]; then
    echo "RESULT: FAIL — one or more benchmarks regressed beyond the 10% threshold."
    exit 1
else
    echo "RESULT: PASS — no benchmarks regressed beyond the 10% threshold."
    exit 0
fi
```

#### GitHub Actions Workflow — `.github/workflows/bench.yml`

```yaml
name: Benchmark Regression Detection

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  bench:
    name: Criterion Benchmarks
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-bench-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-bench-

      - name: Build benches (release)
        run: cargo build --release --benches

      - name: Run throughput bench (with baseline)
        run: |
          cargo bench --bench throughput -- --baseline main \
            2>&1 | tee bench_throughput.log || true

      - name: Run variance bench (with baseline)
        run: |
          cargo bench --bench variance -- --baseline main \
            2>&1 | tee bench_variance.log || true

      - name: Run profile bench (with baseline)
        run: |
          cargo bench --bench profile -- --baseline main \
            2>&1 | tee bench_profile.log || true

      - name: Check regressions
        run: bash benches/check_regression.sh target/criterion/ benches/baselines/

      - name: Generate bench summary
        if: always()
        run: bash benches/gen_summary.sh

      - name: Upload Criterion reports
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: criterion-reports
          path: target/criterion/

      - name: Upload bench summary
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: bench-summary
          path: BENCH_SUMMARY.md

      - name: Post summary to PR
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const summary = fs.readFileSync('BENCH_SUMMARY.md', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: summary
            });

  baseline-update:
    name: Update Baselines (main branch only)
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main' && github.event_name == 'push'
    needs: bench

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Run throughput bench (save baseline)
        run: cargo bench --bench throughput -- --save-baseline main

      - name: Run variance bench (save baseline)
        run: cargo bench --bench variance -- --save-baseline main

      - name: Run profile bench (save baseline)
        run: cargo bench --bench profile -- --save-baseline main

      - name: Commit updated baselines
        run: |
          mkdir -p benches/baselines
          cp -r target/criterion/throughput benches/baselines/ 2>/dev/null || true
          cp -r target/criterion/variance benches/baselines/ 2>/dev/null || true
          cp -r target/criterion/profile benches/baselines/ 2>/dev/null || true
          git config user.email "ci@affidavit.rs"
          git config user.name "affidavit CI"
          git add benches/baselines/
          git diff --cached --quiet || git commit -m "chore: update bench baselines [skip ci]"
          git push
```

### Acceptance Criteria

**Given** a PR is opened against `main` that does not modify any performance-critical code,  
**When** the `bench` CI job runs,  
**Then** the job passes (exit 0) and a bench summary comment is posted to the PR within 10 minutes of the job starting.

---

**Given** a PR introduces a change that degrades `assemble_pipeline/100` mean latency by 15%,  
**When** the CI regression check runs,  
**Then** `check_regression.sh` exits 1, the CI job fails, and the PR is blocked from merge.

---

**Given** a PR introduces a change that improves `verify_pipeline/100` mean latency by 20%,  
**When** the CI regression check runs,  
**Then** `check_regression.sh` exits 0, and the bench summary comment shows a green `-20%` delta for that function.

---

**Given** a commit is pushed to `main`,  
**When** the `baseline-update` CI job runs,  
**Then** updated baseline JSON files are committed to `benches/baselines/` with the commit message `chore: update bench baselines [skip ci]`.

---

**Given** `benches/baselines/` contains no prior baseline for a function,  
**When** `check_regression.sh` runs and finds no `change/estimates.json` for that function,  
**Then** the script skips that function with a `SKIP` log entry and does not fail.

---

**Given** a PR introduces a change that degrades a bench function by exactly 7%,  
**When** the CI regression check runs,  
**Then** `check_regression.sh` exits 0 (not a failure), logs `WARN` for that function, and the bench summary comment shows an amber delta of `+7%`.

---

**Given** the CI job uploads `target/criterion/` as an artifact,  
**When** the artifact is downloaded and opened,  
**Then** the Criterion HTML report at `report/index.html` renders all benchmarks with the custom CSS theme applied.

---

**Given** the `bench` CI job runs on a PR,  
**When** the `check_regression.sh` step fails,  
**Then** the CI job is marked as failed (not just as a warning), and the GitHub merge button shows a required check failure.

---

**Given** the full CI pipeline runs including all three bench suites,  
**When** all suites pass their regression checks,  
**Then** the total CI job time for the `bench` job is ≤ 30 minutes (including build time and all three bench runs).

### Before/After Test Fixtures

The `benches/baselines/` directory serves as the "before" fixture. The "after" is always the current run's `target/criterion/` output. To manually test regression detection without CI:

```bash
# Step 1: Establish a known-good baseline
cargo bench --bench throughput -- --save-baseline known-good

# Step 2: Introduce a synthetic regression (e.g., add a sleep)
# (make the change, then run:)
cargo bench --bench throughput -- --baseline known-good

# Step 3: Check regression gate
bash benches/check_regression.sh target/criterion/ benches/baselines/

# Step 4: Restore the change and update the baseline
git checkout src/
cargo bench --bench throughput -- --save-baseline known-good
```

---

## Cross-Feature DoD Gates

The following gates must ALL be met before the Benchmarking & Performance phase is considered complete:

| Gate | Description | Owner |
|------|-------------|-------|
| All bench files compile | `cargo bench --no-run` exits 0 | CI |
| All bench functions named correctly | Criterion group names match the normative list in each Feature section | CI |
| Throughput targets met | All p50/p99 values within Performance Target Table | CI |
| Scaling constraints verified | Linearity ratios in [8, 12] for assemble and verify | CI |
| O(1) emit verified | Emit scaling ratio in [0.5, 1.5] across chain sizes 1–100 | CI |
| Regression gate wired | `check_regression.sh` is called in `.github/workflows/bench.yml` | CI |
| Baselines committed | `benches/baselines/` contains base JSON for all three suites | Git |
| Dashboard generated | `gen_summary.sh` produces valid `BENCH_SUMMARY.md` | CI |
| Custom CSS applied | `benches/theme/criterion.css` present and patched into HTML output | CI |
| Flamegraph captures blake3 | `target/flamegraph.svg` exists and contains `blake3` frame labels | Manual |
| Flamegraph captures serde_json | `target/flamegraph.svg` contains `serde_json` frame labels | Manual |
| Surprise metric in range | `variance/surprise_sequential` returns values in [0.0, 1.0] | CI |
| No `unwrap` in bench files | Bench files use `.expect("message")` per No-Unwrap Policy | Code review |
| Existing bench unmodified | `benches/receipt_operations.rs` behavior unchanged | CI regression |
| All tests still pass | `cargo test` exits 0 after adding bench files | CI |

---

## Flamegraph Interpretation Guide

### Prerequisites

```bash
# Linux (perf-based)
cargo install flamegraph
sudo apt-get install linux-perf  # or linux-tools-$(uname -r)
echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid
echo 0 | sudo tee /proc/sys/kernel/kptr_restrict

# macOS (dtrace-based)
cargo install flamegraph
# No additional setup needed; uses dtrace natively
```

### Generating a Flamegraph

```bash
# Full verify pipeline flamegraph (recommended starting point)
cargo flamegraph --bench profile --output target/flamegraph.svg \
  -- profile/verify_hot_path/100

# Chain recompute only (isolates BLAKE3 cost)
cargo flamegraph --bench profile --output target/flamegraph_chain.svg \
  -- profile/chain_recompute_hot_path/100

# Canonical JSON serialization only (isolates serde_json cost)
cargo flamegraph --bench profile --output target/flamegraph_json.svg \
  -- profile/canonical_bytes_hot_path/100
```

### Reading the Flamegraph

A Criterion flamegraph is a stack trace visualization where:

- **x-axis:** Proportion of CPU samples (width = percentage of CPU time)
- **y-axis:** Call stack depth (bottom = entry point, top = leaf functions)
- **Color:** Arbitrary (used to distinguish frames, not to indicate heat)
- **Frame width:** Proportional to CPU time spent in that function and its callees

### Expected Frame Layout for `verify_hot_path/100`

```
[wide base frame — bench harness / criterion iter]
 └── affidavit::verifier::verify               ~100% (root)
      ├── affidavit::chain::recompute_chain     ~45–55% (dominant)
      │    └── affidavit::chain::fold_event     ~40–50%
      │         ├── affidavit::types::canonical_bytes   ~20–25%
      │         │    └── serde_json::to_vec / sort_value ~15–20%
      │         └── blake3::hash                ~20–25%
      │              └── blake3::platform::hash_many_neon / avx2 etc.
      ├── affidavit::verifier::stage_continuity  ~10–15%
      │    └── std::collections::BTreeSet::insert ~8–12%
      └── affidavit::verifier::stage_verify_commitments ~5–10%
           └── affidavit::types::Blake3Hash::as_hex ~5%
```

### Interpreting Optimization Opportunities

| Observation | Interpretation | Recommended action |
|-------------|---------------|-------------------|
| `serde_json` > 30% of total | JSON serialization dominates | Consider pre-computing canonical bytes at append time (memoization) |
| `blake3` < 10% of total | BLAKE3 is not the bottleneck | Look at `serde_json` and `BTreeSet` operations instead |
| `sort_value` recursion visible | Key-sorting in canonical JSON is slow | Consider a faster sorted-map implementation or pre-sorted events |
| `BTreeSet::insert` > 20% | Continuity stage is slow for large receipts | Consider a pre-pass to validate seq contiguity instead of a set |
| No `affidavit::` frames in top 5 | Inlining is aggressive | Check that `--release` was used; flamegraph on release builds only |
| `std::alloc` > 10% | Excessive allocation | Profile allocation patterns; consider arena allocation for events |

### Red Flags in Flamegraph

The following frame patterns indicate problems that must be investigated before merge:

- `std::fs` or `std::io` in top 20% of samples from the `verify_hot_path` bench — the bench must operate on in-memory receipts only.
- `std::thread::sleep` in any sample — indicates accidental timing code in the hot path.
- `criterion::` frames consuming > 5% of bench time — indicates the bench is too fast and Criterion overhead dominates; increase `n` or use `iter_batched`.
- `affidavit::admission` frames in `verify_hot_path` — the verifier does not call admission gates; their presence indicates a regression in code organization.
- Any `panic` or `unwrap` frames — indicates a `.unwrap()` call in a hot path, violating the No-Unwrap Policy.

### Sharing Flamegraphs

Commit `target/flamegraph.svg` to the PR description (not to the repository) using a GitHub gist or the CI artifact. The `target/` directory is gitignored. Include the following metadata in the PR description when sharing:

```
Flamegraph: <link to SVG>
Bench function: profile/verify_hot_path/100
Event count: 100
Build: release (cargo flamegraph)
Platform: <linux/macos>
CPU: <model>
Dominant frame: <function name> (<percentage>%)
Notable: <any unexpected observations>
```

---

## Cargo.toml Additions Required

The following bench targets must be added to `Cargo.toml` under `[dev-dependencies]` and as `[[bench]]` entries:

```toml
[[bench]]
name = "throughput"
harness = false

[[bench]]
name = "variance"
harness = false

[[bench]]
name = "profile"
harness = false
```

No new `[dev-dependencies]` are required: `criterion` with `html_reports` is already present. `cargo-flamegraph` is a separate tool installed via `cargo install flamegraph` and does not appear in `Cargo.toml`.

---

*End of Definition of Done — Benchmarking & Performance Regression Detection*
