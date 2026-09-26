// benches/architecture_receipts.rs
//
// Criterion benchmarks for architecture qualification receipts (v26.9.26):
// certify, replay, supersede+chain, and verified JSON parse, at evidence set
// sizes 1, 16 and 256 (shuffled + duplicated input to exercise canonicalization).
//
// Run: cargo bench --bench architecture_receipts
// Recorded baseline + regression bound: benches/architecture_receipts_baseline.json

use affidavit::{ArchitectureQualificationReceipt as Receipt, ArchitectureStanding};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

const SIZES: &[usize] = &[1, 16, 256];

fn evidence(n: usize) -> Vec<String> {
    // Reverse order plus a duplicate of every other element: worst case for
    // the canonical sort/dedup path.
    let mut v: Vec<String> = (0..n).rev().map(|i| format!("sha256:ev-{i:06}")).collect();
    let dups: Vec<String> = v.iter().step_by(2).cloned().collect();
    v.extend(dups);
    v
}

fn certify(n: usize) -> Receipt {
    Receipt::certify(
        "sha256:abb",
        "sha256:contract",
        "sha256:sbb",
        "git:subject",
        evidence(n),
        "sha256:producer",
        vec!["sha256:artifact".into()],
        ArchitectureStanding::Qualified,
    )
    .expect("certify")
}

fn bench_certify(c: &mut Criterion) {
    let mut g = c.benchmark_group("architecture/certify");
    for &n in SIZES {
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || evidence(n),
                |ev| {
                    Receipt::certify(
                        "sha256:abb",
                        "sha256:contract",
                        "sha256:sbb",
                        "git:subject",
                        black_box(ev),
                        "sha256:producer",
                        vec!["sha256:artifact".into()],
                        ArchitectureStanding::Qualified,
                    )
                    .expect("certify")
                },
                criterion::BatchSize::SmallInput,
            )
        });
    }
    g.finish();
}

fn bench_replay(c: &mut Criterion) {
    let mut g = c.benchmark_group("architecture/verify_replay");
    for &n in SIZES {
        let r = certify(n);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &r, |b, r| {
            b.iter(|| {
                black_box(r)
                    .verify_replay("sha256:abb", "sha256:contract", "sha256:sbb", "git:subject")
                    .expect("replay")
            })
        });
    }
    g.finish();
}

fn bench_supersede_chain(c: &mut Criterion) {
    let r = certify(16);
    c.bench_function("architecture/supersede_and_verify_chain/16", |b| {
        b.iter(|| {
            let next = black_box(&r)
                .supersede("sha256:sbb-b", "git:subject-b", vec!["sha256:ev-b".into()])
                .expect("supersede");
            next.verify_chain(&r).expect("chain");
        })
    });
}

fn bench_from_json_verified(c: &mut Criterion) {
    let json = certify(16).to_json();
    c.bench_function("architecture/from_json_verified/16", |b| {
        b.iter(|| Receipt::from_json_verified(black_box(&json)).expect("parse"))
    });
}

criterion_group!(
    benches,
    bench_certify,
    bench_replay,
    bench_supersede_chain,
    bench_from_json_verified
);
criterion_main!(benches);
