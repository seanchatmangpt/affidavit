// benches/throughput.rs
//
// Criterion benchmarks for the emit → assemble → verify pipeline.
// Measures per-operation latency at 1, 5, 10, 50, and 100 events.
//
// Run: cargo bench --bench throughput
// Save baseline: cargo bench --bench throughput -- --save-baseline main
// Compare: cargo bench --bench throughput -- --baseline main

use affidavit::chain::ChainAssembler;
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::verifier::verify;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

const EVENT_COUNTS: &[usize] = &[1, 5, 10, 50, 100];

/// Build a receipt with `n` events and return both the assembler and the receipt.
/// This helper is NOT benchmarked; it is setup work.
fn build_receipt(n: usize) -> affidavit::types::Receipt {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    for i in 0..n {
        let event = build_event(
            "throughput-op",
            vec![object_ref(format!("obj-{i}"), "artifact")],
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
                        vec![object_ref(format!("obj-{i}"), "artifact")],
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
                        vec![object_ref(format!("obj-{i}"), "artifact")],
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
                vec![object_ref(format!("setup-obj-{i}"), "artifact")],
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
                    (asm.clone(), SeqCounter::starting_at(n as u64))
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
