//! Register row 45: seeded deterministic adversarial corpus of event orderings
//! against the root crate's 7-stage verify pipeline.
//!
//! The corpus generator is a pure function of a u64 seed (splitmix64, no
//! external dep): 32 derivations per seed covering the corrupt classes
//! {reorder-stale, reorder-resealed, seq-gap, duplicate-id, truncate-stale,
//! malformed-commitment, empty-event-type} and the lawful classes {honest,
//! truncate-resealed}. Each derivation runs the full verify court. The court
//! asserts the verdict matrix is exactly the lawful set, runs the falsifier
//! (a one-event type swap in one permutation must flip at least one verdict
//! cell; an all-accept corpus is vacuous FAIL), and proves seeded determinism
//! (two runs, same seed -> identical corpus + identical verdicts).

use affidavit::chain::{recompute_chain, ChainAssembler};
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::{Blake3Hash, Receipt};
use affidavit::verifier::verify;

/// Derivation laws applied to the honest base receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    /// Unmodified base receipt — must ACCEPT.
    Honest,
    /// Drop trailing events, recompute the chain hash — must ACCEPT.
    TruncateResealed,
    /// Swap two adjacent events, leave chain_hash stale — chain_integrity.
    ReorderStale,
    /// Swap two adjacent events, recompute chain hash — continuity.
    ReorderResealed,
    /// Bump one interior seq by 1 (gap), recompute — continuity.
    SeqGap,
    /// Rename a later event's id to duplicate an earlier one, recompute — continuity.
    DuplicateId,
    /// Drop trailing events, leave chain_hash stale — chain_integrity.
    TruncateStale,
    /// Corrupt one commitment to malformed hex, recompute — verify_commitments.
    MalformedCommitment,
    /// Blank one event_type, recompute — evaluate_profile.
    EmptyEventType,
}

const ALL_CLASSES: [Class; 9] = [
    Class::Honest,
    Class::TruncateResealed,
    Class::ReorderStale,
    Class::ReorderResealed,
    Class::SeqGap,
    Class::DuplicateId,
    Class::TruncateStale,
    Class::MalformedCommitment,
    Class::EmptyEventType,
];

/// (class, expected first-failing stage) for every corrupt class.
const CORRUPT_EXPECT: [(Class, &str); 7] = [
    (Class::ReorderStale, "chain_integrity"),
    (Class::ReorderResealed, "continuity"),
    (Class::SeqGap, "continuity"),
    (Class::DuplicateId, "continuity"),
    (Class::TruncateStale, "chain_integrity"),
    (Class::MalformedCommitment, "verify_commitments"),
    (Class::EmptyEventType, "evaluate_profile"),
];

/// splitmix64: pure, dependency-free, deterministic PRNG.
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// The honest base receipt: 6 chained events.
fn base_receipt() -> Receipt {
    let mut asm = ChainAssembler::new();
    let mut c = SeqCounter::new();
    let types = ["create", "build", "test", "audit", "sign", "release"];
    for (i, ty) in types.iter().enumerate() {
        let ev = build_event(
            *ty,
            vec![object_ref(format!("obj-{i}"), "artifact")],
            format!("payload-{i}").as_bytes(),
            &mut c,
        )
        .unwrap();
        asm.append(ev).unwrap();
    }
    asm.finalize()
}

/// Reseal a receipt's chain hash over its (mutated) event list.
fn reseal(r: &mut Receipt) {
    r.chain_hash = recompute_chain(&r.events).unwrap();
}

/// Apply one derivation law. Pure function of (seed, class).
fn derive(seed: u64, class: Class) -> Receipt {
    // Per-class seed decorrelation so no two classes share a permutation.
    let mut rng = SplitMix64(seed ^ (class as u64).wrapping_mul(0xD1B5_4A32_D192_ED03));
    let mut r = base_receipt();
    let n = r.events.len();
    match class {
        Class::Honest => r,
        Class::TruncateResealed => {
            let k = 1 + rng.below(n - 1);
            r.events.truncate(k);
            reseal(&mut r);
            r
        }
        Class::ReorderStale | Class::ReorderResealed => {
            let i = rng.below(n - 1);
            r.events.swap(i, i + 1);
            if class == Class::ReorderResealed {
                reseal(&mut r);
            }
            r
        }
        Class::SeqGap => {
            let k = rng.below(n - 2) + 1; // interior position
            r.events[k].seq += 1;
            reseal(&mut r);
            r
        }
        Class::DuplicateId => {
            let j = rng.below(n - 1) + 1;
            r.events[j].id = r.events[0].id.clone();
            reseal(&mut r);
            r
        }
        Class::TruncateStale => {
            let k = 1 + rng.below(n - 1);
            r.events.truncate(k);
            r
        }
        Class::MalformedCommitment => {
            let k = rng.below(n);
            r.events[k].payload_commitment = Blake3Hash::from_hex("NOT-A-DIGEST");
            reseal(&mut r);
            r
        }
        Class::EmptyEventType => {
            let k = rng.below(n);
            r.events[k].event_type = String::new();
            reseal(&mut r);
            r
        }
    }
}

/// A derivation's verdict cell: (accepted, first failing stage or "-").
fn verdict_cell(r: &Receipt) -> (bool, String) {
    let v = verify(r);
    let stage = v
        .outcomes
        .iter()
        .find(|o| !o.passed)
        .map(|o| o.stage.clone())
        .unwrap_or_else(|| "-".to_string());
    (v.accepted, stage)
}

/// Full corpus for one seed: 32 derivations, round-robin over the 9 classes
/// with per-index seed decorrelation (each class gets >= 3 instantiations).
fn corpus(seed: u64) -> Vec<(Class, Receipt, (bool, String))> {
    (0..32u64)
        .map(|i| {
            let class = ALL_CLASSES[(i % 9) as usize];
            let r = derive(seed.wrapping_add(i), class);
            let cell = verdict_cell(&r);
            (class, r, cell)
        })
        .collect()
}

/// Fingerprint of the corpus: per-derivation (class, event count, chain hex,
/// verdict cell) — the determinism court compares these vectors.
fn fingerprint(c: &[(Class, Receipt, (bool, String))]) -> Vec<String> {
    c.iter()
        .map(|(class, r, cell)| {
            format!(
                "{:?}|n={}|chain={}|acc={}|fail={}",
                class,
                r.events.len(),
                r.chain_hash.as_hex(),
                cell.0,
                cell.1
            )
        })
        .collect()
}

#[test]
fn verdict_matrix_is_exactly_lawful() {
    let seed = 45;
    let c = corpus(seed);
    assert_eq!(c.len(), 32);

    let mut accepts = 0usize;
    let mut rejects = 0usize;
    for (class, _r, cell) in &c {
        match class {
            Class::Honest | Class::TruncateResealed => {
                assert!(cell.0, "{class:?} must ACCEPT, got {cell:?}");
                accepts += 1;
            }
            _ => {
                let expected = CORRUPT_EXPECT
                    .iter()
                    .find(|(k, _)| k == class)
                    .map(|(_, s)| *s)
                    .unwrap();
                assert!(!cell.0, "{class:?} must REJECT, got {cell:?}");
                assert_eq!(cell.1, expected, "{class:?} rejected at the wrong stage");
                rejects += 1;
            }
        }
    }
    // Class mix: 32 = 3 full rounds of 9 (27) + 5 extra (Honest..SeqGap), so
    // Honest and TruncateResealed each get 4 cells, the other 7 classes 3 each.
    assert_eq!(accepts, 8, "honest + truncate-resealed cells");
    assert_eq!(rejects, 24, "corrupt cells");
    println!(
        "verdict matrix: {accepts} accept / {rejects} reject over 32 derivations (seed {seed})"
    );
}

#[test]
fn falsifier_swap_flips_at_least_one_cell() {
    // Falsifier: mutate one event in one permutation; at least one verdict
    // cell must flip. Swapping event 0's type to "" in a sealed honest
    // receipt flips accept -> reject at evaluate_profile.
    let seed = 45;
    let before = corpus(seed);
    let mut flipped = 0usize;
    for (class, r, cell) in &before {
        if *class != Class::Honest {
            continue;
        }
        let mut m = r.clone();
        m.events[0].event_type = String::new();
        reseal(&mut m);
        if verdict_cell(&m) != *cell {
            flipped += 1;
        }
    }
    assert!(
        flipped >= 1,
        "falsifier: no verdict cell flipped under the one-event swap"
    );
}

#[test]
fn corpus_is_not_vacuously_all_accept() {
    let c = corpus(45);
    let all_accept = c.iter().all(|(_, _, cell)| cell.0);
    assert!(!all_accept, "vacuous corpus: every derivation accepted");
}

#[test]
fn seeded_determinism_two_runs_identical() {
    let a = fingerprint(&corpus(45));
    let b = fingerprint(&corpus(45));
    assert_eq!(
        a, b,
        "same seed must reproduce corpus + verdicts byte-for-byte"
    );
}

#[test]
fn different_seed_moves_the_derivations() {
    // Not strictly required by the row, but proves the seed is load-bearing
    // (permutations actually vary with it) rather than decoration.
    let a = fingerprint(&corpus(45));
    let b = fingerprint(&corpus(46));
    assert_ne!(a, b, "different seeds must not produce identical corpora");
}
