//! BLS signature aggregation over BLS12-381 (w3f-bls): many independent
//! witnesses, one pairing check.
//!
//! FROST ([`crate::threshold_quorum`]) is interactive: signers cooperate in
//! rounds. BLS is **non-interactive**: each committee member signs the same
//! receipt digest whenever they choose; the auditor later sums the
//! signatures and the public keys and verifies the committee's approval with
//! a single pairing check.
//!
//! Same-message aggregation is used (the committee approves one digest),
//! which is the sound aggregation mode without proof-of-possession
//! infrastructure. Wire serialization of aggregated points is UNSUPPORTED in
//! this module — the capability is aggregate-and-verify within one runtime.
//!
//! ## Doctrine: Certify, Don't Decide
//!
//! One pairing check certifies that the summed committee approved the exact
//! digest. It does not decide whether the committee *should* have.

use rand_core::{CryptoRng, RngCore};
use thiserror::Error;
use w3f_bls::{KeypairVT, Message, PublicKey, Signature, TinyBLS381};

/// Errors produced by BLS aggregation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BlsError {
    /// Signatures and public keys must be provided in equal, non-empty
    /// numbers.
    #[error("committee mismatch: {sigs} signatures for {keys} public keys")]
    CommitteeMismatch {
        /// Number of signatures supplied.
        sigs: usize,
        /// Number of public keys supplied.
        keys: usize,
    },
    /// The aggregated committee signature failed verification.
    #[error("committee aggregation verification failed")]
    VerificationFailed,
}

/// A committee member's key pair (vartime secret for signing ceremonies).
#[derive(Clone)]
pub struct CommitteeKey(pub KeypairVT<TinyBLS381>);

impl CommitteeKey {
    /// Generate a committee member's key pair.
    #[must_use]
    pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        Self(KeypairVT::<TinyBLS381>::generate(rng))
    }

    /// The member's public key.
    #[must_use]
    pub fn public(&self) -> PublicKey<TinyBLS381> {
        self.0.public
    }

    /// Sign a message (context + bytes).
    #[must_use]
    pub fn sign(&self, context: &[u8], message: &[u8]) -> Signature<TinyBLS381> {
        self.0.sign(&Message::new(context, message))
    }
}

/// Sum a group of signature (or public key) points into one aggregate.
///
/// monocoordinate BLS: the sum of signatures and sum of keys verify with a
/// single pairing when every component signed the same message.
fn sum_points<G>(items: impl Iterator<Item = G>) -> G
where
    G: ark_ec::CurveGroup,
{
    // Group: Zero (num-traits); arkworks projective points default to the
    // identity element, so Default is the additive zero of the group.
    items.fold(G::default(), |acc, item| acc + item)
}

/// Aggregate a same-message committee: sum signatures and public keys.
///
/// # Errors
///
/// Returns [`BlsError::CommitteeMismatch`] on unequal or empty inputs.
pub fn aggregate_committee(
    signatures: &[Signature<TinyBLS381>],
    public_keys: &[PublicKey<TinyBLS381>],
) -> Result<(Signature<TinyBLS381>, PublicKey<TinyBLS381>), BlsError> {
    if signatures.len() != public_keys.len() || signatures.is_empty() {
        return Err(BlsError::CommitteeMismatch {
            sigs: signatures.len(),
            keys: public_keys.len(),
        });
    }
    let aggregate_signature = Signature(sum_points(signatures.iter().map(|signature| signature.0)));
    let aggregate_key = PublicKey(sum_points(public_keys.iter().map(|key| key.0)));
    Ok((aggregate_signature, aggregate_key))
}

/// Verify an aggregated committee over a message: one pairing check.
#[must_use]
pub fn verify_committee(
    context: &[u8],
    message: &[u8],
    aggregate_signature: &Signature<TinyBLS381>,
    aggregate_key: &PublicKey<TinyBLS381>,
) -> bool {
    aggregate_signature.verify(&Message::new(context, message), aggregate_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::OsRng;

    const CONTEXT: &[u8] = b"affidavit:bls:v1:committee";

    fn committee(
        n: usize,
        message: &[u8],
    ) -> (Vec<Signature<TinyBLS381>>, Vec<PublicKey<TinyBLS381>>) {
        let mut signatures = Vec::new();
        let mut public_keys = Vec::new();
        for _ in 0..n {
            let member = CommitteeKey::generate(&mut OsRng);
            signatures.push(member.sign(CONTEXT, message));
            public_keys.push(member.public());
        }
        (signatures, public_keys)
    }

    #[test]
    fn full_committee_aggregates_and_verifies() {
        let (signatures, public_keys) = committee(7, b"admit proposal 41");
        let (sig, key) = aggregate_committee(&signatures, &public_keys).expect("aggregate");
        assert!(verify_committee(CONTEXT, b"admit proposal 41", &sig, &key));
    }

    #[test]
    fn tampered_message_breaks_aggregate() {
        let (signatures, public_keys) = committee(5, b"admit proposal 41");
        let (sig, key) = aggregate_committee(&signatures, &public_keys).expect("aggregate");
        assert!(!verify_committee(CONTEXT, b"admit proposal 42", &sig, &key));
    }

    #[test]
    fn a_tampered_member_breaks_the_aggregate() {
        let (mut signatures, public_keys) = committee(5, b"admit");
        let (sig, key) = aggregate_committee(&signatures, &public_keys).expect("aggregate");
        assert!(verify_committee(CONTEXT, b"admit", &sig, &key));

        // One member signs a different digest (dissent or compromise): the
        // summed signature no longer matches the summed keys.
        let dissenter = CommitteeKey::generate(&mut OsRng);
        signatures[0] = dissenter.sign(CONTEXT, b"admit something else");
        let (sig, key) = aggregate_committee(&signatures, &public_keys).expect("aggregate");
        assert!(!verify_committee(CONTEXT, b"admit", &sig, &key));
    }

    #[test]
    fn mismatched_committee_is_typed_refusal() {
        let (signatures, public_keys) = committee(3, b"m");
        assert!(matches!(
            aggregate_committee(&signatures[..2], &public_keys),
            Err(BlsError::CommitteeMismatch { sigs: 2, keys: 3 })
        ));
        assert!(matches!(
            aggregate_committee(&[], &[]),
            Err(BlsError::CommitteeMismatch { .. })
        ));
    }
}
