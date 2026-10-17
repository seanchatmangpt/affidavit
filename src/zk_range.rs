//! Zero-knowledge bounded-metric witnesses (bulletproofs range proofs).
//!
//! An agent can commit to a secret metric `v` (tokens, latency, cost) and
//! prove `0 <= v < 2^bits` — e.g. "effect_metric <= budget_ceiling" —
//! without disclosing `v`. The commitment is a Pedersen commitment; the
//! blinding factor is the prover's custody and must stay secret.
//!
//! ## Doctrine: Certify, Don't Decide
//!
//! The proof certifies a *bound*, not the truth of the underlying
//! measurement. A prover can prove a false measurement is within range; the
//! fence proves the committed number respects the ceiling, and the ceiling
//! law lives in the ERRC claim, not here.

use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use curve25519_dalek_ng::ristretto::CompressedRistretto;
use curve25519_dalek_ng::scalar::Scalar;
use merlin::Transcript;
use rand_core::{CryptoRng, RngCore};
use thiserror::Error;

/// Errors produced by range-proof operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RangeProofError {
    /// The proof could not be generated (e.g. value out of the declared
    /// bit range).
    #[error("range proof generation failed: {0}")]
    Generation(String),
    /// The proof or commitment bytes are malformed.
    #[error("malformed range proof encoding")]
    MalformedEncoding,
    /// A Pedersen commitment does not open under the supplied
    /// `(value, blinding)` pair.
    #[error("pedersen opening mismatch: {0}")]
    OpeningMismatch(String),
    /// The proof does not verify against the commitment.
    #[error("range proof verification failed")]
    VerificationFailed,
}

/// A prover-side range witness: the public commitment, the secret blinding
/// (custody of the prover), and the serialized proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeWitness {
    /// 32-byte compressed Pedersen commitment to the value.
    pub commitment: [u8; 32],
    /// 32-byte blinding scalar. **Secret.** Losing it forfeits opening;
    /// leaking it voids the zero-knowledge property.
    pub blinding: [u8; 32],
    /// Serialized range proof.
    pub proof: Vec<u8>,
    /// The bit range this proof was constructed over.
    pub bits: usize,
}

/// Prove `0 <= value < 2^bits` under a Pedersen commitment without
/// disclosing the value.
///
/// # Errors
///
/// Returns [`RangeProofError::Generation`] if `bits` is outside bulletproofs'
/// supported range (1..=64) or the value does not fit.
///
/// # Panics
///
/// Never panics on any input value; range violations are typed refusals.
pub fn prove_range<R: RngCore + CryptoRng>(
    value: u64,
    bits: usize,
    label: &'static [u8],
    rng: &mut R,
) -> Result<RangeWitness, RangeProofError> {
    if bits == 0 || bits > 64 {
        return Err(RangeProofError::Generation(format!(
            "unsupported bit range: {bits}"
        )));
    }
    // Bulletproofs' prover truncates over-range values to `bits` instead of
    // refusing, which would certify a bound the value does not satisfy.
    // The fence refuses before the prover can.
    if bits < 64 && (value >> bits) != 0 {
        return Err(RangeProofError::Generation(format!(
            "value {value} does not fit in {bits} bits"
        )));
    }
    let gens = BulletproofGens::new(bits.next_power_of_two().max(32), 1);
    let pc_gens = PedersenGens::default();
    let mut transcript = Transcript::new(label);
    // Blinding entropy comes from the caller's RNG as bytes; this avoids
    // depending on the -ng fork's optional rand_core feature surface.
    let mut blinding_bytes = [0u8; 32];
    rng.fill_bytes(&mut blinding_bytes);
    let blinding = Scalar::from_bytes_mod_order(blinding_bytes);

    let (proof, commitment) = RangeProof::prove_single_with_rng(
        &gens,
        &pc_gens,
        &mut transcript,
        value,
        &blinding,
        bits,
        rng,
    )
    .map_err(|e| RangeProofError::Generation(e.to_string()))?;

    let mut commitment_bytes = [0u8; 32];
    commitment_bytes.copy_from_slice(commitment.as_bytes());
    let blinding_bytes: [u8; 32] = blinding.to_bytes();

    Ok(RangeWitness {
        commitment: commitment_bytes,
        blinding: blinding_bytes,
        proof: proof.to_bytes(),
        bits,
    })
}

/// Derive a deterministic blinding scalar: `blake3(seed || label || value-le)`,
/// reduced mod the group order. Same `(seed, label, value)` triple always
/// yields the same scalar, so the resulting range proof is byte-identical
/// across runs and processes.
fn seeded_blinding(seed: &[u8; 32], label: &[u8], value: u64) -> Scalar {
    let mut hasher = blake3::Hasher::new();
    hasher.update(seed);
    hasher.update(label);
    hasher.update(&value.to_le_bytes());
    Scalar::from_bytes_mod_order(hasher.finalize().into())
}

/// Prove `0 <= value < 2^bits` with a **seeded deterministic blinding**
/// derived as `Scalar::from_bytes_mod_order(blake3(seed || label ||
/// value-le-bytes))`.
///
/// Because the blinding is a pure function of `(seed, label, value)`, two
/// invocations with identical inputs produce byte-identical proofs
/// (`cmp`-equal) — no RNG involvement. Any change to the seed, the label, or
/// the value flips proof bytes. The blinding remains secret *custody of the
/// holder of the seed*: leaking `seed` is equivalent to leaking every
/// blinding derived from it.
///
/// The existing [`prove_range`] (OsRng) path is untouched.
///
/// # Errors
///
/// Same refusals as [`prove_range`].
///
/// # Panics
///
/// Never panics; out-of-range values are typed refusals.
pub fn prove_range_seeded(
    seed: &[u8; 32],
    value: u64,
    bits: usize,
    label: &'static [u8],
) -> Result<RangeWitness, RangeProofError> {
    let blinding = seeded_blinding(seed, label, value);
    let mut rng = SeededRng::new(*seed, label, value);
    prove_single_with_blinding(value, bits, label, None, blinding, &mut rng)
}

/// Verify a Pedersen commitment opens under `(value, blinding)`: recompute
/// `C = value*G + blinding*H` and compare against the commitment bytes.
///
/// This is the commitment-opening seam the receipt-bound derivation relies
/// on: a [`RangeWitness`] always carries an opening consistent with its
/// commitment, and a commitment bound to receipt hash `H1` (via
/// [`prove_range_bound`]'s transcript append) still opens under its own
/// `(value, blinding)` — but its proof refuses verification under any other
/// binding.
///
/// # Errors
///
/// [`RangeProofError::OpeningMismatch`] when the recomputed commitment
/// differs — wrong value or wrong blinding under the same `C`.
pub fn pedersen_open(
    commitment: &[u8; 32],
    value: u64,
    blinding: &[u8; 32],
) -> Result<(), RangeProofError> {
    let pc_gens = PedersenGens::default();
    let mut v_le = [0u8; 32];
    v_le[..8].copy_from_slice(&value.to_le_bytes());
    let v = Scalar::from_bytes_mod_order(v_le);
    let b = Scalar::from_bytes_mod_order(*blinding);
    let recomputed = pc_gens.commit(v, b).compress();
    if recomputed == CompressedRistretto(*commitment) {
        Ok(())
    } else {
        Err(RangeProofError::OpeningMismatch(format!(
            "commitment {} does not open under the supplied (value, blinding)",
            hex_encode(commitment)
        )))
    }
}

/// Lowercase-hex encode for typed error messages (no extra dep).
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Deterministic blake3-keystream CPRNG for bulletproofs' internal
/// polynomial blinding factors. Same key always yields the same byte stream,
/// so a seeded proof is byte-identical across runs.
pub struct SeededRng {
    key: [u8; 32],
    counter: u64,
    buf: [u8; 32],
    pos: usize,
}

impl SeededRng {
    /// New keystream keyed by `blake3(seed || label || value-le || nonce)`.
    fn new(seed: [u8; 32], label: &[u8], value: u64) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&seed);
        hasher.update(b"affidavit:zk-range:seeded-rng:v1");
        hasher.update(label);
        hasher.update(&value.to_le_bytes());
        Self {
            key: hasher.finalize().into(),
            counter: 0,
            buf: [0u8; 32],
            pos: 32,
        }
    }

    fn refill(&mut self) {
        let mut hasher = blake3::Hasher::new_keyed(&self.key);
        hasher.update(&self.counter.to_le_bytes());
        self.buf = hasher.finalize().into();
        hasher.reset();
        hasher.update(&self.counter.to_le_bytes());
        self.counter += 1;
    }
}

impl RngCore for SeededRng {
    fn next_u32(&mut self) -> u32 {
        let mut b = [0u8; 4];
        self.fill_bytes(&mut b);
        u32::from_le_bytes(b)
    }

    fn next_u64(&mut self) -> u64 {
        let mut b = [0u8; 8];
        self.fill_bytes(&mut b);
        u64::from_le_bytes(b)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for byte in dest.iter_mut() {
            if self.pos == self.buf.len() {
                self.refill();
                self.pos = 0;
            }
            *byte = self.buf[self.pos];
            self.pos += 1
        }
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl CryptoRng for SeededRng {}

/// Shared single-prover path for the seeded and RNG paths: appends the
/// receipt hash to the transcript (when bound) before proving with the
/// caller-supplied blinding scalar.
fn prove_single_with_blinding<R: RngCore + CryptoRng>(
    value: u64,
    bits: usize,
    label: &'static [u8],
    receipt_hash: Option<&[u8; 32]>,
    blinding: Scalar,
    rng: &mut R,
) -> Result<RangeWitness, RangeProofError> {
    if bits == 0 || bits > 64 {
        return Err(RangeProofError::Generation(format!(
            "unsupported bit range: {bits}"
        )));
    }
    if bits < 64 && (value >> bits) != 0 {
        return Err(RangeProofError::Generation(format!(
            "value {value} does not fit in {bits} bits"
        )));
    }
    let gens = BulletproofGens::new(bits.next_power_of_two().max(32), 1);
    let pc_gens = PedersenGens::default();
    let mut transcript = Transcript::new(label);
    if let Some(h) = receipt_hash {
        transcript.append_message(b"receipt-hash", h);
    }

    let (proof, commitment) = RangeProof::prove_single_with_rng(
        &gens,
        &pc_gens,
        &mut transcript,
        value,
        &blinding,
        bits,
        rng,
    )
    .map_err(|e| RangeProofError::Generation(e.to_string()))?;

    let mut commitment_bytes = [0u8; 32];
    commitment_bytes.copy_from_slice(commitment.as_bytes());
    let blinding_bytes: [u8; 32] = blinding.to_bytes();

    Ok(RangeWitness {
        commitment: commitment_bytes,
        blinding: blinding_bytes,
        proof: proof.to_bytes(),
        bits,
    })
}

/// Verify that a commitment carries a value within `[0, 2^bits)` under the
/// same domain label used at proving time.
///
/// # Errors
///
/// Returns [`RangeProofError::MalformedEncoding`] for undecodable proofs and
/// [`RangeProofError::VerificationFailed`] when the proof does not hold.
pub fn verify_range(
    commitment: &[u8; 32],
    proof_bytes: &[u8],
    bits: usize,
    label: &'static [u8],
) -> Result<(), RangeProofError> {
    if bits == 0 || bits > 64 {
        return Err(RangeProofError::Generation(format!(
            "unsupported bit range: {bits}"
        )));
    }
    let gens = BulletproofGens::new(bits.next_power_of_two().max(32), 1);
    let pc_gens = PedersenGens::default();
    let proof =
        RangeProof::from_bytes(proof_bytes).map_err(|_| RangeProofError::MalformedEncoding)?;
    let mut transcript = Transcript::new(label);
    proof
        .verify_single(
            &gens,
            &pc_gens,
            &mut transcript,
            &CompressedRistretto(*commitment),
            bits,
        )
        .map_err(|_| RangeProofError::VerificationFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::OsRng;

    const LABEL: &[u8] = b"affidavit:zk-range:v1:effect-budget";

    #[test]
    fn in_range_value_proves_and_verifies() {
        let witness = prove_range(42, 32, LABEL, &mut OsRng).expect("prove");
        assert!(verify_range(&witness.commitment, &witness.proof, 32, LABEL).is_ok());
    }

    #[test]
    fn ceiling_boundary_value_proves() {
        // v = 2^16 - 1 within 16 bits: exactly at the ceiling.
        let witness = prove_range((1 << 16) - 1, 16, LABEL, &mut OsRng).expect("prove");
        assert!(verify_range(&witness.commitment, &witness.proof, 16, LABEL).is_ok());
    }

    #[test]
    fn out_of_range_value_is_a_generation_refusal() {
        // 2^16 does not fit in 16 bits.
        assert!(prove_range(1 << 16, 16, LABEL, &mut OsRng).is_err());
    }

    #[test]
    fn wrong_label_breaks_verification() {
        let witness = prove_range(7, 32, LABEL, &mut OsRng).expect("prove");
        assert_eq!(
            verify_range(&witness.commitment, &witness.proof, 32, b"other domain"),
            Err(RangeProofError::VerificationFailed)
        );
    }

    #[test]
    fn tampered_proof_is_refused() {
        let witness = prove_range(7, 32, LABEL, &mut OsRng).expect("prove");
        let mut proof = witness.proof.clone();
        let last = proof.len() - 1;
        proof[last] ^= 0x01;
        let result = verify_range(&witness.commitment, &proof, 32, LABEL);
        assert!(matches!(
            result,
            Err(RangeProofError::VerificationFailed) | Err(RangeProofError::MalformedEncoding)
        ));
    }

    #[test]
    fn unsupported_bit_range_is_typed_refusal() {
        assert!(prove_range(1, 0, LABEL, &mut OsRng).is_err());
        assert!(prove_range(1, 65, LABEL, &mut OsRng).is_err());
    }
}
/// A member of an aggregated range-proof batch: the public commitment and
/// the secret blinding (prover custody), as in [`RangeWitness`] but without
/// a per-member proof — the batch shares one aggregated proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeBatchMember {
    /// 32-byte compressed Pedersen commitment to the member's value.
    pub commitment: [u8; 32],
    /// 32-byte blinding scalar. **Secret**, prover custody only.
    pub blinding: [u8; 32],
}

/// An aggregated range-proof batch: one [`RangeProof`] covering `m`
/// commitments simultaneously (`RangeProof::prove_multiple` at prove time,
/// `RangeProof::verify_multiple` at verify time, shared [`BulletproofGens`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeBatch {
    /// Per-member commitments and blindings.
    pub members: Vec<RangeBatchMember>,
    /// The single aggregated proof covering every member.
    pub proof: Vec<u8>,
    /// The bit range every member was constructed over.
    pub bits: usize,
}

/// Bit ranges accepted for aggregated batches. `RangeProof::verify_multiple`
/// refuses any `n` outside `{8, 16, 32, 64}` (`InvalidBitsize`), so the batch
/// surface gates up front with a typed refusal instead of letting the
/// external crate's error leak as a generic failure.
const BATCH_SUPPORTED_BITS: [usize; 4] = [8, 16, 32, 64];

fn check_batch_bits(bits: usize) -> Result<(), RangeProofError> {
    if !BATCH_SUPPORTED_BITS.contains(&bits) {
        return Err(RangeProofError::Generation(format!(
            "batch requires bits in {BATCH_SUPPORTED_BITS:?}, got {bits}"
        )));
    }
    Ok(())
}

/// Prove `0 <= value_i < 2^bits` for every member simultaneously as one
/// aggregated proof. All members share the domain label and the bit range.
///
/// # Errors
///
/// [`RangeProofError::Generation`] for an empty batch, an unsupported bit
/// range, or any value that does not fit in `bits`.
///
/// # Panics
///
/// Never panics; out-of-range members are typed refusals.
pub fn prove_range_batch<R: RngCore>(
    values: &[u64],
    bits: usize,
    label: &'static [u8],
    rng: &mut R,
) -> Result<RangeBatch, RangeProofError> {
    prove_range_batch_bound(values, bits, label, None, rng)
}

/// Prove an aggregated batch bound to an optional receipt hash.
///
/// When `receipt_hash` is `Some`, the hash is appended to the Merlin
/// transcript under the domain-separated key `b"receipt-hash"` *before* the
/// protocol runs, so the aggregated proof (and its per-member commitments)
/// is cryptographically bound to that receipt: a proof generated under H1
/// fails verification under any other hash H2 because the Fiat-Shamir
/// challenges diverge.
///
/// **Drop-append mutation reasoning.** Both prover and verifier append
/// unconditionally whenever the binding is present, so the two dangerous
/// mutations both fail loudly rather than silently degrading: (a) a prover
/// that drops the append produces a proof that a correct verifier refuses
/// (challenge divergence), and (b) a verifier that drops the append refuses
/// a correctly-bound proof — the binding is lost as a hard `Err`, never as a
/// silent accept. The residual non-cryptographic risk — both sides dropping
/// the append in lockstep, silently unbinding the proof from the receipt —
/// is a source-level invariant, covered by
/// [`tests::bound_proof_rejects_verification_without_the_receipt_hash`],
/// which fails if either side stops threading the append through
/// `prove_multiple`/`verify_multiple`.
///
/// # Errors
///
/// Same refusals as [`prove_range_batch`].
pub fn prove_range_batch_bound<R: RngCore>(
    values: &[u64],
    bits: usize,
    label: &'static [u8],
    receipt_hash: Option<&[u8; 32]>,
    rng: &mut R,
) -> Result<RangeBatch, RangeProofError> {
    if values.is_empty() {
        return Err(RangeProofError::Generation(
            "batch must contain at least one member".into(),
        ));
    }
    check_batch_bits(bits)?;
    for &v in values {
        if bits < 64 && (v >> bits) != 0 {
            return Err(RangeProofError::Generation(format!(
                "value {v} does not fit in {bits} bits"
            )));
        }
    }
    // The aggregated protocol requires the aggregation size m to be a power
    // of two. Short batches are padded with value-0 members (0 is always in
    // range, so the aggregate stays sound and leaks nothing); the pads are
    // carried as ordinary members so the verifier sees exactly the
    // commitments the proof covers.
    let m = values.len().next_power_of_two();
    let mut padded: Vec<u64> = values.to_vec();
    padded.resize(m, 0);
    let gens = BulletproofGens::new(bits, m);
    let pc_gens = PedersenGens::default();
    let mut transcript = Transcript::new(label);
    if let Some(h) = receipt_hash {
        transcript.append_message(b"receipt-hash", h);
    }
    let blindings: Vec<Scalar> = padded
        .iter()
        .map(|_| {
            let mut b = [0u8; 32];
            rng.fill_bytes(&mut b);
            Scalar::from_bytes_mod_order(b)
        })
        .collect();

    let (proof, commitments) =
        RangeProof::prove_multiple(&gens, &pc_gens, &mut transcript, &padded, &blindings, bits)
            .map_err(|e| RangeProofError::Generation(e.to_string()))?;

    let members = commitments
        .iter()
        .zip(blindings)
        .map(|(c, b)| {
            let mut commitment = [0u8; 32];
            commitment.copy_from_slice(c.as_bytes());
            RangeBatchMember {
                commitment,
                blinding: b.to_bytes(),
            }
        })
        .collect();

    Ok(RangeBatch {
        members,
        proof: proof.to_bytes(),
        bits,
    })
}

/// Verify an aggregated batch over the same label (and receipt hash, if the
/// proof was bound to one) used at proving time.
///
/// Wraps [`RangeProof::verify_multiple`] with a single [`BulletproofGens`]
/// sized for the whole batch. If any member's commitment does not carry a
/// value in `[0, 2^bits)` — or the proof bytes were swapped, tampered with,
/// or generated under a different transcript — the *whole* batch fails with
/// [`RangeProofError::VerificationFailed`]; a failing member can never
/// surface as a silent pass.
///
/// # Errors
///
/// [`RangeProofError::MalformedEncoding`] for undecodable proof bytes,
/// [`RangeProofError::Generation`] for an empty member list or unsupported
/// bit range, [`RangeProofError::VerificationFailed`] when the batch does
/// not hold.
pub fn verify_range_batch(
    batch: &RangeBatch,
    label: &'static [u8],
    receipt_hash: Option<&[u8; 32]>,
) -> Result<(), RangeProofError> {
    if batch.members.is_empty() {
        return Err(RangeProofError::Generation(
            "batch must contain at least one member".into(),
        ));
    }
    check_batch_bits(batch.bits)?;
    if !batch.members.len().is_power_of_two() {
        return Err(RangeProofError::Generation(format!(
            "batch size must be a power of two, got {}",
            batch.members.len()
        )));
    }
    let gens = BulletproofGens::new(batch.bits, batch.members.len());
    let pc_gens = PedersenGens::default();
    let proof =
        RangeProof::from_bytes(&batch.proof).map_err(|_| RangeProofError::MalformedEncoding)?;
    let mut transcript = Transcript::new(label);
    if let Some(h) = receipt_hash {
        transcript.append_message(b"receipt-hash", h);
    }
    let commitments: Vec<CompressedRistretto> = batch
        .members
        .iter()
        .map(|m| CompressedRistretto(m.commitment))
        .collect();
    proof
        .verify_multiple(&gens, &pc_gens, &mut transcript, &commitments, batch.bits)
        .map_err(|_| RangeProofError::VerificationFailed)
}
/// Prove `0 <= value < 2^bits` bound to an optional receipt hash, threading
/// `transcript.append_message(b"receipt-hash", h)` before the protocol runs.
///
/// Without a binding (`None`) this is byte-behaviorally identical to
/// [`prove_range`]; existing callers are unaffected.
///
/// See [`prove_range_batch_bound`] for the drop-append mutation reasoning,
/// which applies identically here.
///
/// # Errors
///
/// Same refusals as [`prove_range`].
pub fn prove_range_bound<R: RngCore + CryptoRng>(
    value: u64,
    bits: usize,
    label: &'static [u8],
    receipt_hash: Option<&[u8; 32]>,
    rng: &mut R,
) -> Result<RangeWitness, RangeProofError> {
    prove_range_with_hash(value, bits, label, receipt_hash, rng)
}

/// Verify a range proof over the same label (and receipt hash, if bound)
/// used at proving time. `None` matches [`verify_range`]'s behavior.
///
/// # Errors
///
/// Same refusals as [`verify_range`].
pub fn verify_range_bound(
    commitment: &[u8; 32],
    proof_bytes: &[u8],
    bits: usize,
    label: &'static [u8],
    receipt_hash: Option<&[u8; 32]>,
) -> Result<(), RangeProofError> {
    verify_range_with_hash(commitment, proof_bytes, bits, label, receipt_hash)
}

/// Shared prove path for [`prove_range`] / [`prove_range_bound`]: appends
/// the receipt hash to the transcript (when bound) before proving.
fn prove_range_with_hash<R: RngCore + CryptoRng>(
    value: u64,
    bits: usize,
    label: &'static [u8],
    receipt_hash: Option<&[u8; 32]>,
    rng: &mut R,
) -> Result<RangeWitness, RangeProofError> {
    let mut blinding_bytes = [0u8; 32];
    rng.fill_bytes(&mut blinding_bytes);
    prove_single_with_blinding(
        value,
        bits,
        label,
        receipt_hash,
        Scalar::from_bytes_mod_order(blinding_bytes),
        rng,
    )
}

/// Shared verify path for [`verify_range`] / [`verify_range_bound`]: appends
/// the receipt hash to the transcript (when bound) before verifying.
fn verify_range_with_hash(
    commitment: &[u8; 32],
    proof_bytes: &[u8],
    bits: usize,
    label: &'static [u8],
    receipt_hash: Option<&[u8; 32]>,
) -> Result<(), RangeProofError> {
    if bits == 0 || bits > 64 {
        return Err(RangeProofError::Generation(format!(
            "unsupported bit range: {bits}"
        )));
    }
    let gens = BulletproofGens::new(bits.next_power_of_two().max(32), 1);
    let pc_gens = PedersenGens::default();
    let proof =
        RangeProof::from_bytes(proof_bytes).map_err(|_| RangeProofError::MalformedEncoding)?;
    let mut transcript = Transcript::new(label);
    if let Some(h) = receipt_hash {
        transcript.append_message(b"receipt-hash", h);
    }
    proof
        .verify_single(
            &gens,
            &pc_gens,
            &mut transcript,
            &CompressedRistretto(*commitment),
            bits,
        )
        .map_err(|_| RangeProofError::VerificationFailed)
}

#[cfg(test)]
mod bound_batch_tests {
    //! Falsifier courts for registers #33 (receipt-bound witnesses) and
    //! #34 (aggregated batch verification).

    use super::*;
    use rand_core::OsRng;

    const LABEL: &[u8] = b"affidavit:zk-range:v1:effect-budget";
    const H1: [u8; 32] = [1u8; 32];
    const H2: [u8; 32] = [2u8; 32];

    #[test]
    fn bound_proof_verifies_under_its_own_receipt_hash() {
        let witness = prove_range_bound(42, 32, LABEL, Some(&H1), &mut OsRng).expect("prove");
        assert_eq!(
            verify_range_bound(&witness.commitment, &witness.proof, 32, LABEL, Some(&H1)),
            Ok(())
        );
    }

    #[test]
    fn bound_proof_prove_h1_verify_h2_is_refused() {
        // Falsifier: the receipt hash is Fiat-Shamir-bound; a different hash
        // diverges the challenges, so the proof cannot be replayed against
        // another receipt.
        let witness = prove_range_bound(42, 32, LABEL, Some(&H1), &mut OsRng).expect("prove");
        assert_eq!(
            verify_range_bound(&witness.commitment, &witness.proof, 32, LABEL, Some(&H2)),
            Err(RangeProofError::VerificationFailed)
        );
    }

    #[test]
    fn bound_proof_rejects_verification_without_the_receipt_hash() {
        // Drop-append mutation control: a verifier that omits the
        // b"receipt-hash" append must get a loud Err, never a silent accept;
        // symmetrically, a proof proven unbound must not verify under a
        // hash. If either side stops threading the append, this fails.
        let bound = prove_range_bound(42, 32, LABEL, Some(&H1), &mut OsRng).expect("prove");
        assert_eq!(
            verify_range_bound(&bound.commitment, &bound.proof, 32, LABEL, None),
            Err(RangeProofError::VerificationFailed)
        );
        let unbound = prove_range(42, 32, LABEL, &mut OsRng).expect("prove");
        assert_eq!(
            verify_range_bound(&unbound.commitment, &unbound.proof, 32, LABEL, Some(&H1)),
            Err(RangeProofError::VerificationFailed)
        );
    }

    #[test]
    fn bound_with_none_matches_unbound_behavior() {
        let a = prove_range_bound(9, 16, LABEL, None, &mut OsRng).expect("prove");
        assert_eq!(verify_range(&a.commitment, &a.proof, 16, LABEL), Ok(()));
    }

    #[test]
    fn bound_batch_verifies_only_under_its_receipt_hash() {
        let batch = prove_range_batch_bound(&[1, 2, 3], 32, LABEL, Some(&H1), &mut OsRng)
            .expect("prove batch");
        assert_eq!(verify_range_batch(&batch, LABEL, Some(&H1)), Ok(()));
        assert_eq!(
            verify_range_batch(&batch, LABEL, Some(&H2)),
            Err(RangeProofError::VerificationFailed)
        );
        assert_eq!(
            verify_range_batch(&batch, LABEL, None),
            Err(RangeProofError::VerificationFailed)
        );
    }

    // ---- Register #34: aggregated batch verification ----

    #[test]
    fn batch_of_eight_valid_proofs_verifies() {
        let values: Vec<u64> = (0..8).map(|i| i * 1_000).collect();
        let batch = prove_range_batch(&values, 32, LABEL, &mut OsRng).expect("prove batch");
        assert_eq!(batch.members.len(), 8);
        assert_eq!(verify_range_batch(&batch, LABEL, None), Ok(()));
    }

    #[test]
    fn swapping_batch_proof_bytes_fails_loudly() {
        let values: Vec<u64> = (0..8).map(|i| i * 500).collect();
        let batch = prove_range_batch(&values, 32, LABEL, &mut OsRng).expect("prove batch");

        // Cross-batch swap: a valid proof from a different aggregate must
        // not be accepted for this batch's commitments.
        let other = prove_range_batch(&values, 32, LABEL, &mut OsRng).expect("prove second batch");
        let mut swapped = batch.clone();
        swapped.proof = other.proof;
        assert_eq!(
            verify_range_batch(&swapped, LABEL, None),
            Err(RangeProofError::VerificationFailed)
        );

        // Byte-level tamper on a member-carrying aggregate: any corruption
        // of the shared proof surfaces as a typed failure, not a pass.
        let mut tampered = batch.clone();
        let last = tampered.proof.len() - 1;
        tampered.proof[last] ^= 0x01;
        assert!(matches!(
            verify_range_batch(&tampered, LABEL, None),
            Err(RangeProofError::VerificationFailed) | Err(RangeProofError::MalformedEncoding)
        ));
    }

    #[test]
    fn batch_matches_sequential_outcomes_on_mixed_corpus() {
        // Mixed corpus: zero, one, ceiling boundary, and large in-range
        // values across 8 members.
        let mixed: Vec<u64> = vec![
            0,
            1,
            4_294_967_295, // 2^32 - 1, exactly at the ceiling
            123_456_789,
            2_000_000_000,
            999,
            65_535,
            4_294_967_294,
        ];
        assert_eq!(mixed.len(), 8);
        let batch = prove_range_batch(&mixed, 32, LABEL, &mut OsRng).expect("prove batch");
        assert_eq!(verify_range_batch(&batch, LABEL, None), Ok(()));

        // Sequential baseline: every member's value proves and verifies on
        // its own, so batch and sequential agree (both accept).
        for &v in &mixed {
            let w = prove_range(v, 32, LABEL, &mut OsRng).expect("sequential prove");
            assert_eq!(verify_range(&w.commitment, &w.proof, 32, LABEL), Ok(()));
        }

        // Disagreement case: a corpus with an out-of-range member is
        // refused by both paths (batch refuses at generation; sequential
        // refuses per member).
        let bad: Vec<u64> = vec![1, 2, 1 << 33];
        assert!(prove_range_batch(&bad, 32, LABEL, &mut OsRng).is_err());
        assert!(prove_range(1 << 33, 32, LABEL, &mut OsRng).is_err());
    }

    #[test]
    fn empty_batch_is_a_typed_refusal() {
        assert!(prove_range_batch(&[], 32, LABEL, &mut OsRng).is_err());
        let one = prove_range_batch(&[5], 32, LABEL, &mut OsRng).expect("prove");
        let mut empty = one.clone();
        empty.members.clear();
        assert_eq!(
            verify_range_batch(&empty, LABEL, None),
            Err(RangeProofError::Generation(
                "batch must contain at least one member".into()
            ))
        );
    }

    #[test]
    fn batch_rejects_unsupported_bit_ranges() {
        assert!(prove_range_batch(&[1], 12, LABEL, &mut OsRng).is_err());
        let mut batch = prove_range_batch(&[1], 32, LABEL, &mut OsRng).expect("prove");
        batch.bits = 12;
        assert_eq!(
            verify_range_batch(&batch, LABEL, None),
            Err(RangeProofError::Generation(
                "batch requires bits in [8, 16, 32, 64], got 12".into()
            ))
        );
    }
}

#[cfg(test)]
mod seeded_opening_tests {
    //! Register row 63: seeded deterministic blinding. Cycle-4 row B
    //! (Pedersen half): commitment-opening consistency seam.

    use super::*;
    use rand_core::OsRng;

    const LABEL: &[u8] = b"affidavit:zk-range:v1:effect-budget";
    const SEED: [u8; 32] = [7u8; 32];
    const H1: [u8; 32] = [1u8; 32];
    const H2: [u8; 32] = [2u8; 32];

    // ---- Row 63: seeded deterministic blinding ----

    #[test]
    fn same_seed_label_value_is_byte_identical_across_invocations() {
        let a = prove_range_seeded(&SEED, 42, 32, LABEL).expect("prove a");
        let b = prove_range_seeded(&SEED, 42, 32, LABEL).expect("prove b");
        assert_eq!(a, b);
        assert_eq!(cmp_bytes(&a.proof, &b.proof), std::cmp::Ordering::Equal);
        assert_eq!(
            cmp_bytes(&a.commitment, &b.commitment),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn changing_any_of_seed_label_value_flips_proof_bytes() {
        let base = prove_range_seeded(&SEED, 42, 32, LABEL).expect("prove base");

        let mut other_seed = SEED;
        other_seed[0] ^= 0x01;
        let alt_seed = prove_range_seeded(&other_seed, 42, 32, LABEL).expect("prove alt seed");
        assert_ne!(base.proof, alt_seed.proof);
        assert_ne!(base.commitment, alt_seed.commitment);

        let alt_label = prove_range_seeded(&SEED, 42, 32, b"affidavit:zk-range:v1:other-domain")
            .expect("prove alt label");
        assert_ne!(base.proof, alt_label.proof);
        assert_ne!(base.commitment, alt_label.commitment);

        let alt_value = prove_range_seeded(&SEED, 43, 32, LABEL).expect("prove alt value");
        assert_ne!(base.proof, alt_value.proof);
        assert_ne!(base.commitment, alt_value.commitment);
    }

    #[test]
    fn seeded_proof_verifies_and_opens() {
        let witness = prove_range_seeded(&SEED, 1_000, 32, LABEL).expect("prove");
        assert_eq!(
            verify_range(&witness.commitment, &witness.proof, 32, LABEL),
            Ok(())
        );
        assert_eq!(
            pedersen_open(&witness.commitment, 1_000, &witness.blinding),
            Ok(())
        );
    }

    #[test]
    fn seeded_out_of_range_value_is_refused() {
        assert!(prove_range_seeded(&SEED, 1 << 32, 32, LABEL).is_err());
    }

    #[test]
    fn osrng_path_remains_nondeterministic() {
        // Falsifier for "existing OsRng path untouched": two fresh random
        // witnesses of the same value must differ (fresh entropy each run).
        let a = prove_range(42, 32, LABEL, &mut OsRng).expect("prove a");
        let b = prove_range(42, 32, LABEL, &mut OsRng).expect("prove b");
        assert_ne!(a.proof, b.proof);
        assert_ne!(a.commitment, b.commitment);
    }

    // ---- Cycle-4 row B (Pedersen half): opening consistency ----

    #[test]
    fn commitment_opens_under_correct_pair_only() {
        let w = prove_range(42, 32, LABEL, &mut OsRng).expect("prove");
        assert_eq!(pedersen_open(&w.commitment, 42, &w.blinding), Ok(()));
        assert!(matches!(
            pedersen_open(&w.commitment, 43, &w.blinding),
            Err(RangeProofError::OpeningMismatch(_))
        ));
        assert!(matches!(
            pedersen_open(&w.commitment, 42, &wrong_blinding(&w)),
            Err(RangeProofError::OpeningMismatch(_))
        ));
        // A zero blinding under a nonzero-value commitment is refused too.
        assert!(matches!(
            pedersen_open(&w.commitment, 42, &[0u8; 32]),
            Err(RangeProofError::OpeningMismatch(_))
        ));
    }

    fn wrong_blinding(w: &RangeWitness) -> [u8; 32] {
        let mut b = w.blinding;
        b[0] ^= 0x01;
        b
    }

    #[test]
    fn receipt_bound_derivation_carries_through_the_opening_seam() {
        // Row 33 binding carries through: a witness bound to H1 opens under
        // its own (value, blinding) and verifies under H1, but its proof
        // refuses under a different receipt-hash binding — the opening seam
        // stays consistent while the transcript binding stays exclusive.
        let w = prove_range_bound(42, 32, LABEL, Some(&H1), &mut OsRng).expect("prove bound");
        assert_eq!(pedersen_open(&w.commitment, 42, &w.blinding), Ok(()));
        assert_eq!(
            verify_range_bound(&w.commitment, &w.proof, 32, LABEL, Some(&H1)),
            Ok(())
        );
        assert_eq!(
            verify_range_bound(&w.commitment, &w.proof, 32, LABEL, Some(&H2)),
            Err(RangeProofError::VerificationFailed)
        );
    }

    fn cmp_bytes(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
        a.cmp(b)
    }
}
