#![cfg(feature = "crypto-trust")]

//! QG-05: constant-time (dudect) timing-invariance court.
//!
//! Real-collaborator harness (Chicago style — no mocks): measures actual
//! execution times of two secret-dependent classes against the real
//! `wire_ctrt::WireCtrt::deserialize` and the real deterministic ML-DSA-65
//! signer, computes an inline Welch t-statistic over batched medians, and
//! gates on |t| < 4.5 with an idle-retry law mirroring
//! `chicago-tdd-tools/src/nato/timing.rs` (`run_court`, up to 3 idle reruns).
//!
//! Welch implementation ported with attribution from
//! `chicago-tdd-tools/src/nato/timing.rs::welch_t` (same math, total_cmp
//! medians, NaN on zero variance).
//!
//! Positive control: a deliberately timing-leaky comparison function that
//! spins only on class 1. The court MUST flag it (|t| >= 4.5) — otherwise
//! every PASS this harness issues is vacuous.
//!
//! Commands:
//! - default (10^5 samples/class): `cargo test --release -p affidavit --test ct_timing_invariance`
//! - full   (10^6 samples/class): `cargo test --release -p affidavit --test ct_timing_invariance -- --ignored qg-05-full`

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use affidavit::crypto_trust_pqc::{ml_dsa65_from_seed, ml_dsa65_sign, ML_DSA_65_SEED_LEN};
use affidavit::wire_ctrt::{
    deserialize_ct, VerdictStatus, WireCtrt, WireCtrtBody, CTRT_PAYLOAD_SIZE, CTRT_WIRE_SIZE,
};

/// Welch t pass threshold (same law as nato/timing.rs `T_THRESHOLD`).
const T_THRESHOLD: f64 = 4.5;
/// Max attempts total (1 + 3 idle reruns), mirroring nato/timing.rs run_court.
const MAX_ATTEMPTS: usize = 4;
/// Idle cooldown between attempts.
const RETRY_SLEEP: Duration = Duration::from_millis(500);

/// Optimizer sink: consumed into an atomic never folded at -O2/3.
#[inline(never)]
fn sink(x: u64) -> u64 {
    AtomicU64::new(0).fetch_xor(x, Ordering::Relaxed)
}

/// Median of `v` (mutates: sorts) with `f64::total_cmp` — ported from
/// nato/timing.rs (NaN on empty, mean of middle pair on even length).
fn median(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n == 0 {
        f64::NAN
    } else if n % 2 == 0 {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    } else {
        v[n / 2]
    }
}

/// Welch's two-sample t-statistic (unequal variances, unequal n).
///
/// Ported with attribution from
/// `chicago-tdd-tools/src/nato/timing.rs::welch_t`. Returns NaN when either
/// sample has zero variance (both classes quantize to the same timer
/// quantum — the strongest no-divergence signal this clock can produce) or
/// either slice has fewer than 2 elements.
fn welch_t(a: &[f64], b: &[f64]) -> f64 {
    if a.len() < 2 || b.len() < 2 {
        return f64::NAN;
    }
    let n1 = a.len() as f64;
    let n2 = b.len() as f64;
    let m1 = a.iter().sum::<f64>() / n1;
    let m2 = b.iter().sum::<f64>() / n2;
    let v1 = a.iter().map(|x| (x - m1) * (x - m1)).sum::<f64>() / (n1 - 1.0);
    let v2 = b.iter().map(|x| (x - m2) * (x - m2)).sum::<f64>() / (n2 - 1.0);
    (m1 - m2) / ((v1 / n1) + (v2 / n2)).sqrt()
}

/// Pass predicate: NaN (zero variance) or |t| < 4.5.
fn passes(t: f64) -> bool {
    t.is_nan() || t.abs() < T_THRESHOLD
}

/// Fixed 64-bit splitmix64-style generator — deterministic class assignment
/// interleaving (no RNG dependency; real distribution over the class bit).
struct ClassRng(u64);
impl ClassRng {
    fn next_bit(&mut self) -> u8 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15).rotate_left(29) ^ (self.0 >> 32);
        (self.0 ^ (self.0 >> 61)) as u8 & 1
    }
}

/// One dudect attempt over a two-class measurement closure: interleaved
/// class selection, batched inner repeats, per-class batch medians, Welch t.
///
/// `measure_once(class) -> u64` must execute the class's real workload and
/// return a value consumed by the sink.
fn dudect_t<F>(samples_per_class: usize, repeats: usize, mut measure_once: F) -> f64
where
    F: FnMut(u8) -> u64,
{
    let n_batches = (samples_per_class / repeats).max(4);
    let mut rng = ClassRng(0xD0DE_C7A5_B1A5_0001);
    let mut t_valid: Vec<f64> = Vec::with_capacity(n_batches);
    let mut t_other: Vec<f64> = Vec::with_capacity(n_batches);
    let mut counts = [0usize; 2];

    for _ in 0..(2 * n_batches) {
        let class = rng.next_bit();
        let start = Instant::now();
        let mut acc = 0u64;
        for _ in 0..repeats {
            acc ^= measure_once(class);
        }
        let d = start.elapsed().as_nanos() as f64;
        sink(acc);
        let per_call = d / repeats as f64;
        if class == 0 {
            t_valid.push(per_call);
        } else {
            t_other.push(per_call);
        }
        counts[class as usize] += 1;
    }

    let med_a = median(&mut t_valid);
    let med_b = median(&mut t_other);
    let t = welch_t(&t_valid, &t_other);
    eprintln!(
        "  [dudect] batches: class0={} class1={} med0={:.1}ns med1={:.1}ns t={:.4}",
        counts[0], counts[1], med_a, med_b, t
    );
    t
}

/// nato/timing.rs `run_court` law: full measurement; on |t| >= 4.5 rerun
/// idle (up to MAX_ATTEMPTS total) before concluding — machine noise vs
/// genuine divergence, all attempt t-statistics reported.
fn court<F>(samples_per_class: usize, repeats: usize, f: F) -> Vec<f64>
where
    F: FnMut(u8) -> u64 + Clone,
{
    let mut attempts: Vec<f64> = Vec::new();
    for attempt in 0..MAX_ATTEMPTS {
        if attempt > 0 {
            eprintln!("  [court] |t| >= 4.5 on attempt {} — idle rerun", attempt);
            std::thread::sleep(RETRY_SLEEP);
        }
        let t = dudect_t(samples_per_class, repeats, f.clone());
        attempts.push(t);
        if passes(t) {
            break;
        }
    }
    attempts
}

fn samples_from_env(default: usize) -> usize {
    std::env::var("QG05_SAMPLES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

// --- Subjects --------------------------------------------------------------

/// Build a deterministic valid 352-byte CTRT wire record for `subject` id.
fn ctrt_record(subject: u8) -> [u8; CTRT_WIRE_SIZE] {
    let mut rec = WireCtrt::new(
        [0x11u8; 32],
        [0x22u8; 32],
        [0x33u8; 32],
        [0x44u8; 32],
        0x0000_00FF,
        0x0000_0100,
        VerdictStatus::Accepted,
        [0u8; CTRT_PAYLOAD_SIZE],
    );
    rec.subject_merkle_root[0] = subject;
    rec.serialize()
}

/// Class 0: valid magic. Class 1: mutated magic byte (deliberately invalid
/// record — the deserialize hot path still processes all payload bytes
/// branchlessly before the single error-sink return).
fn ctrt_class_input(class: u8, subject: u8) -> [u8; CTRT_WIRE_SIZE] {
    let mut bytes = ctrt_record(subject);
    if class == 1 {
        bytes[0] = b'X'; // invalid magic: CTRT -> XTRT
    }
    bytes
}

/// Deliberately timing-leaky twin (positive control): dependent ~spin only
/// on class 1 — structurally identical to nato/timing.rs `leaky_refusal`.
#[inline(never)]
fn leaky_ctrt(class: u8, bytes: &[u8; CTRT_WIRE_SIZE]) -> u64 {
    let mut acc = bytes[0] as u64;
    if class == 1 {
        for i in 0..400u64 {
            acc = acc.rotate_left(7)
                ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^ bytes[i as usize % CTRT_WIRE_SIZE] as u64;
        }
    }
    acc
}

/// Fixture: fixed seed + fixed rnd (deterministic Signer); per-class distinct
/// 32-byte subjects.
struct MlDsaFixture {
    seed: [u8; ML_DSA_65_SEED_LEN],
    rnd: [u8; 32],
}

impl MlDsaFixture {
    fn new() -> Self {
        let mut seed = [0u8; ML_DSA_65_SEED_LEN];
        for (i, b) in seed.iter_mut().enumerate() {
            *b = (i as u8) ^ 0xA5;
        }
        let mut rnd = [0u8; 32];
        for (i, b) in rnd.iter_mut().enumerate() {
            *b = (i as u8) ^ 0x5A;
        }
        Self { seed, rnd }
    }

    fn sign(&self, subject: &[u8; 32]) -> u64 {
        match ml_dsa65_sign(&self.seed, subject, &self.rnd) {
            Ok(sig) => {
                let l = sig.len();
                (sig[l / 2] as u64) << 8 | l as u64
            }
            Err(_) => 0,
        }
    }
}

// --- Tests -----------------------------------------------------------------

/// Court 1: the constant-time decode core `wire_ctrt::deserialize_ct` must
/// not leak the magic/verdict validity class. Both classes perform identical
/// work: all fields are copied into a caller-fixed-size `WireCtrtBody` and a
/// machine code is returned instead of a size-varying `Result`.
#[test]
fn qg05_ctrt_deserialize_timing_invariance() {
    let samples = samples_from_env(100_000);
    let rec = ctrt_record(0);
    // Warmup: page in the hot path, pre-fault allocations.
    let mut body = WireCtrtBody::default();
    let mut code = 0u16;
    for _ in 0..2000 {
        code ^= deserialize_ct(&mut body, &rec);
        code ^= deserialize_ct(&mut body, &ctrt_class_input(1, 1));
        std::hint::black_box((code, body.execution_ticks));
    }

    // Each measurement call owns a fresh fixed-size `WireCtrtBody` (a plain
    // Default struct — no allocation, same work per class), so the closure is
    // `Clone` as `court` requires; previously it captured `&mut body`, which
    // is not Clone (E0277). dudect methodology unchanged.
    let attempts = court(samples, 64, |class| {
        let mut body = WireCtrtBody::default();
        let bytes = ctrt_class_input(class, 0);
        let code = deserialize_ct(&mut body, &bytes);
        u64::from(code) | body.execution_ticks
    });
    let t = *attempts.last().expect("at least one attempt");
    assert!(
        passes(t),
        "QG-05 FAIL: wire_ctrt::deserialize_ct leaks timing class (attempts {:?})",
        attempts
    );
}

/// Court 2: deterministic ML-DSA-65 sign must not leak the subject class.
#[test]
fn qg05_ml_dsa65_sign_timing_invariance() {
    let samples = samples_from_env(100_000);
    let fx = MlDsaFixture::new();
    let kp = ml_dsa65_from_seed(&fx.seed);
    let subj0 = [0xA5u8; 32];
    let subj1 = {
        let mut s = [0x5Au8; 32];
        s[0] = 0xFF;
        s
    };
    // Warmup + sanity: both subjects sign, deterministic same-signature.
    let s0a = fx.sign(&subj0);
    let s0b = fx.sign(&subj0);
    assert_eq!(s0a, s0b, "deterministic signer violated");
    for _ in 0..50 {
        std::hint::black_box((fx.sign(&subj0), fx.sign(&subj1)));
    }
    assert!(!kp.public.is_empty());

    let attempts = court(samples, 8, |class| {
        if class == 0 {
            fx.sign(&subj0)
        } else {
            fx.sign(&subj1)
        }
    });
    let t = *attempts.last().expect("at least one attempt");
    assert!(
        passes(t),
        "QG-05 FAIL: ml_dsa65_sign leaks timing subject class (attempts {:?})",
        attempts
    );
}

/// Positive control: the leaky twin MUST trip the court (|t| >= 4.5 within
/// the allowed attempts) — otherwise the harness is vacuous.
#[test]
fn qg05_positive_control_leaky_twin_must_fire() {
    let samples = samples_from_env(20_000);
    let rec = ctrt_record(0);
    let attempts = court(samples, 64, |class| {
        let bytes = ctrt_class_input(class, 0);
        if class == 1 {
            leaky_ctrt(class, &bytes)
        } else {
            sink(rec[0] as u64)
        }
    });
    let fired = attempts
        .iter()
        .any(|t| !t.is_nan() && t.abs() >= T_THRESHOLD);
    assert!(
        fired,
        "QG-05 vacuous court: leaky positive control was NOT detected (attempts {:?})",
        attempts
    );
}
