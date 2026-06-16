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
