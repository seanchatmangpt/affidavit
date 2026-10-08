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
use rand_core::RngCore;
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
pub fn prove_range<R: RngCore>(
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

    let (proof, commitment) =
        RangeProof::prove_single(&gens, &pc_gens, &mut transcript, value, &blinding, bits)
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
