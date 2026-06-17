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

use affidavit::chain::{recompute_chain, ChainAssembler};
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::{canonical_bytes, Blake3Hash};
use affidavit::verifier::verify;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;

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
