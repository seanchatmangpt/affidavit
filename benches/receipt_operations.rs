// Benchmarks for receipt operations (emit, assemble, verify).
//
// Measures the performance of the core receipt sealing and verification pipeline.
// Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use affidavit::chain::{ChainAssembler, recompute_chain};
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::verifier::verify;

fn bench_chain_append(c: &mut Criterion) {
    c.bench_function("chain_append_single_event", |b| {
        b.iter(|| {
            let mut asm = ChainAssembler::new();
            let mut counter = SeqCounter::new();
            let event = build_event(
                "test",
                vec![object_ref("obj", "artifact")],
                black_box(b"payload"),
                &mut counter,
            ).expect("build event");
            asm.append(event).expect("append");
            asm.finalize()
        })
    });
}

fn bench_chain_finalize(c: &mut Criterion) {
    let mut group = c.benchmark_group("chain_finalize");
    for event_count in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(event_count),
            event_count,
            |b, &event_count| {
                b.iter(|| {
                    let mut asm = ChainAssembler::new();
                    let mut counter = SeqCounter::new();
                    for _ in 0..event_count {
                        let event = build_event(
                            "op",
                            vec![object_ref("obj", "artifact")],
                            black_box(b"data"),
                            &mut counter,
                        ).expect("build event");
                        asm.append(event).expect("append");
                    }
                    asm.finalize()
                })
            },
        );
    }
    group.finish();
}

fn bench_verifier_pipeline(c: &mut Criterion) {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();

    // Create a 10-event receipt
    for _ in 0..10 {
        let event = build_event(
            "test",
            vec![object_ref("obj", "artifact")],
            b"payload",
            &mut counter,
        ).expect("build event");
        asm.append(event).expect("append");
    }
    let receipt = black_box(asm.finalize());

    c.bench_function("verifier_pipeline_10_events", |b| {
        b.iter(|| {
            verify(&receipt)
        })
    });
}

fn bench_chain_recompute(c: &mut Criterion) {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();

    for _ in 0..100 {
        let event = build_event(
            "op",
            vec![object_ref("obj", "artifact")],
            b"data",
            &mut counter,
        ).expect("build event");
        asm.append(event).expect("append");
    }
    let receipt = asm.finalize();

    c.bench_function("recompute_chain_100_events", |b| {
        b.iter(|| {
            recompute_chain(&receipt.events)
        })
    });
}

// Benchmark the wasm4pm discovery + conformance path on a receipt (the
// discover-then-conform pipeline run end-to-end).
fn bench_conformance_metrics(c: &mut Criterion) {
    use affidavit::discovery::quality_metrics;
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    for act in ["create", "transform", "validate", "release"] {
        let ev = build_event(act, vec![object_ref("o", "artifact")], act.as_bytes(), &mut counter)
            .expect("event");
        asm.append(ev).expect("append");
    }
    let receipt = asm.finalize();
    c.bench_function("conformance_metrics_4_events", |b| {
        b.iter(|| {
            let (f, p, s) = quality_metrics(black_box(&receipt));
            black_box((f, p, s))
        })
    });
}

criterion_group!(
    benches,
    bench_chain_append,
    bench_chain_finalize,
    bench_verifier_pipeline,
    bench_chain_recompute,
    bench_conformance_metrics,
);
criterion_main!(benches);
