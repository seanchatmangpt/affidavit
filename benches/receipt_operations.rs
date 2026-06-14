// Benchmarks for receipt operations (emit, assemble, verify).
//
// Measures the performance of the core receipt sealing and verification pipeline.
// Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use affidavit::chain::{ChainAssembler, recompute_chain, FORMAT_VERSION};
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::verifier::verify;

/// Build an honest n-event receipt using the standard assembler flow.
fn build_receipt(n: usize) -> affidavit::types::Receipt {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    for i in 0..n {
        let ty = ["create", "transform", "validate", "release"][i % 4];
        let ev = build_event(ty, vec![object_ref("obj", "artifact")], b"data", &mut counter)
            .expect("build event");
        asm.append(ev).expect("append");
    }
    asm.finalize()
}

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

fn bench_admission_gate(c: &mut Criterion) {
    use affidavit::admission::admit;
    let mut group = c.benchmark_group("admission_gate");
    for event_count in [1usize, 5, 20, 100].iter() {
        let receipt = build_receipt(*event_count);
        group.bench_with_input(
            BenchmarkId::from_parameter(event_count),
            &receipt,
            |b, receipt| {
                b.iter(|| admit(receipt.clone()))
            },
        );
    }
    group.finish();
}

fn bench_lsp_diagnostics(c: &mut Criterion) {
    use affidavit::lsp::verdict_to_diagnostics;
    use affidavit::types::{Blake3Hash, OperationEvent};
    use affidavit::ocel::object_ref as ocel_object_ref;

    let mut group = c.benchmark_group("lsp_diagnostics");

    // Case 1: clean receipt (0 diagnostics)
    let clean = build_receipt(3);
    let clean_verdict = verify(&clean);
    group.bench_function("clean_receipt_0_diags", |b| {
        b.iter(|| verdict_to_diagnostics(black_box(&clean_verdict)))
    });

    // Case 2: forged receipt (1+ diagnostics) — seq starts at 5
    let forged_event = OperationEvent {
        id: "evt-5".to_string(),
        seq: 5,
        event_type: "create".to_string(),
        objects: vec![ocel_object_ref("o", "artifact")],
        payload_commitment: Blake3Hash::from_bytes(b"x"),
    };
    let chain_hash = recompute_chain(std::slice::from_ref(&forged_event)).expect("chain");
    let forged: affidavit::types::Receipt = serde_json::from_value(serde_json::json!({
        "format_version": FORMAT_VERSION,
        "events": [forged_event],
        "chain_hash": chain_hash,
    })).expect("deserialize");
    let forged_verdict = verify(&forged);
    group.bench_function("forged_receipt_2_diags", |b| {
        b.iter(|| verdict_to_diagnostics(black_box(&forged_verdict)))
    });

    group.finish();
}

fn bench_discovery_pipeline(c: &mut Criterion) {
    use affidavit::discovery::{discover_dfg_summary, quality_metrics};

    let mut group = c.benchmark_group("discovery_pipeline");
    for event_count in [4usize, 20, 100].iter() {
        let receipt = build_receipt(*event_count);
        group.bench_with_input(
            BenchmarkId::new("dfg_summary", event_count),
            &receipt,
            |b, receipt| {
                b.iter(|| discover_dfg_summary(black_box(receipt)))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("quality_metrics", event_count),
            &receipt,
            |b, receipt| {
                b.iter(|| quality_metrics(black_box(receipt)))
            },
        );
    }
    group.finish();
}

fn bench_object_ref_parsing(c: &mut Criterion) {
    use affidavit::ocel::parse_object_ref;

    let mut group = c.benchmark_group("object_ref_parsing");
    group.bench_function("simple_id_type", |b| {
        b.iter(|| parse_object_ref(black_box("file.txt:artifact")))
    });
    group.bench_function("qualified_id_type_qualifier", |b| {
        b.iter(|| parse_object_ref(black_box("file.txt:artifact:source")))
    });
    group.bench_function("error_empty_string", |b| {
        b.iter(|| parse_object_ref(black_box("")))
    });
    group.bench_function("long_id", |b| {
        let long = "a".repeat(200) + ":artifact";
        b.iter(|| parse_object_ref(black_box(long.as_str())))
    });
    group.finish();
}

fn bench_verifier_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("verifier_scaling");
    for event_count in [1usize, 10, 100, 500].iter() {
        let receipt = build_receipt(*event_count);
        group.bench_with_input(
            BenchmarkId::from_parameter(event_count),
            &receipt,
            |b, receipt| {
                b.iter(|| verify(black_box(receipt)))
            },
        );
    }
    group.finish();
}

fn bench_build_event_multi_object(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_event_object_count");
    for obj_count in [1usize, 5, 10].iter() {
        let objects: Vec<_> = (0..*obj_count)
            .map(|i| object_ref(&format!("obj-{i}"), "artifact"))
            .collect();
        group.bench_with_input(
            BenchmarkId::from_parameter(obj_count),
            &objects,
            |b, objects| {
                b.iter(|| {
                    let mut c = SeqCounter::new();
                    build_event(black_box("create"), black_box(objects.clone()), b"payload", &mut c)
                })
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_chain_append,
    bench_chain_finalize,
    bench_verifier_pipeline,
    bench_chain_recompute,
    bench_conformance_metrics,
    bench_admission_gate,
    bench_lsp_diagnostics,
    bench_discovery_pipeline,
    bench_object_ref_parsing,
    bench_verifier_scaling,
    bench_build_event_multi_object,
);
criterion_main!(benches);
