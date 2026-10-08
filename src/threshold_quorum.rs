//! FROST threshold quorum certification (RFC 9591, Ed25519).
//!
//! `t`-of-`n` independent witness nodes co-sign a message; the result is a
//! single, compact Ed25519 Schnorr signature verifying against the *group*
//! public key. Downstream effectors verify one signature — they need not
//! know which nodes signed, only the group key and the threshold law under
//! which it was formed.
//!
//! ## Doctrine: Certify, Don't Decide
//!
//! A quorum signature is a witness of coordinated admission. The engine
//! refuses malformed shares, non-participants, and under-threshold
//! aggregation with typed errors; it never decides whether the co-signed
//! proposal is *wise*.

use std::collections::BTreeMap;

use frost_ed25519 as frost;
use frost_ed25519::keys::{PublicKeyPackage, SecretShare};
use frost_ed25519::round1::SigningNonces;
use frost_ed25519::{aggregate, Identifier, Signature, SigningPackage, VerifyingKey};
use rand_core::{CryptoRng, RngCore};
use thiserror::Error;

/// Errors produced by threshold quorum operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum QuorumError {
    /// The FROST protocol rejected a parameter, share, or package.
    #[error("FROST protocol failure: {0}")]
    Protocol(String),
    /// A signature did not verify against the group key.
    #[error("quorum signature verification failed")]
    VerificationFailed,
    /// Key material could not be serialized or deserialized.
    #[error("malformed quorum key material")]
    MalformedKeyMaterial,
    /// Fewer than the threshold participants contributed.
    #[error("below threshold: expected at least {expected} shares, got {got}")]
    BelowThreshold {
        /// Required threshold.
        expected: usize,
        /// Shares actually supplied.
        got: usize,
    },
}

/// The group public key, serialized (32 bytes, Ed25519).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GroupPublicKey(pub [u8; 32]);

/// A participant identifier, mapped from a `1..=n` index (index 0 is
/// refused: FROST identifiers are 1-based).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParticipantId(pub u16);

impl ParticipantId {
    fn to_frost(self) -> Result<Identifier, QuorumError> {
        Identifier::try_from(self.0).map_err(|e| QuorumError::Protocol(e.to_string()))
    }
}

/// A dealt quorum: one secret share per participant, the public key package
/// needed for aggregation, and the group key for downstream verifiers.
#[derive(Debug, Clone)]
pub struct DealtQuorum {
    /// Secret share per participant index. Custody: each share goes to its
    /// participant and must never be aggregated with another's.
    pub shares: BTreeMap<ParticipantId, SecretShare>,
    /// Public key package (verifying shares per identifier).
    pub public_key_package: PublicKeyPackage,
    /// The single group public key every downstream verifier needs.
    pub group_key: GroupPublicKey,
    /// The threshold `t` this quorum was dealt with.
    pub threshold: u16,
}

/// Deal a fresh `threshold`-of-`total` key generation ceremony.
///
/// # Errors
///
/// Returns [`QuorumError::Protocol`] if FROST's dealer rejects the
/// parameters (e.g. threshold 0 or threshold above total).
pub fn generate_quorum<R: RngCore + CryptoRng>(
    total: u16,
    threshold: u16,
    rng: &mut R,
) -> Result<DealtQuorum, QuorumError> {
    let (shares, public_key_package) = frost::keys::generate_with_dealer(
        total,
        threshold,
        frost::keys::IdentifierList::Default,
        rng,
    )
    .map_err(|e| QuorumError::Protocol(e.to_string()))?;
    let mut by_participant = BTreeMap::new();
    for index in 1..=total {
        let identifier =
            Identifier::try_from(index).map_err(|e| QuorumError::Protocol(e.to_string()))?;
        let share = shares
            .get(&identifier)
            .ok_or_else(|| QuorumError::Protocol("dealer omitted participant".to_string()))?
            .clone();
        by_participant.insert(ParticipantId(index), share);
    }
    let verifying_key = public_key_package.verifying_key();
    let bytes = verifying_key
        .serialize()
        .map_err(|_| QuorumError::MalformedKeyMaterial)?;
    let group_key: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| QuorumError::MalformedKeyMaterial)?;
    Ok(DealtQuorum {
        shares: by_participant,
        public_key_package,
        group_key: GroupPublicKey(group_key),
        threshold,
    })
}

/// Round 1: produce this participant's single-use signing nonces (secret)
/// and the commitments to broadcast to the coordinator.
///
/// # Errors
///
/// See [`QuorumError::Protocol`].
pub fn round1_commit<R: RngCore + CryptoRng>(
    secret_share: &SecretShare,
    rng: &mut R,
) -> Result<(SigningNonces, frost::round1::SigningCommitments), QuorumError> {
    let nonces = SigningNonces::new(secret_share.signing_share(), rng);
    let commitments = *nonces.commitments();
    Ok((nonces, commitments))
}

/// Assemble the signing package from coordinator-collected commitments.
///
/// # Errors
///
/// Returns [`QuorumError::BelowThreshold`] if fewer than `threshold`
/// commitments were supplied.
pub fn signing_package(
    commitments: &[(ParticipantId, frost::round1::SigningCommitments)],
    message: &[u8],
    threshold: u16,
) -> Result<SigningPackage, QuorumError> {
    if commitments.len() < usize::from(threshold) {
        return Err(QuorumError::BelowThreshold {
            expected: usize::from(threshold),
            got: commitments.len(),
        });
    }
    let mut map = BTreeMap::new();
    for (participant, commitment) in commitments {
        map.insert(participant.to_frost()?, *commitment);
    }
    Ok(SigningPackage::new(map, message))
}

/// Round 2: produce this participant's signature share over the package.
///
/// The participant derives their key package from their secret share; the
/// signing itself is deterministic (RFC 9591 round 2 needs no entropy).
///
/// # Errors
///
/// See [`QuorumError::Protocol`].
pub fn round2_sign(
    secret_share: &SecretShare,
    nonces: &SigningNonces,
    package: &SigningPackage,
) -> Result<frost::round2::SignatureShare, QuorumError> {
    let key_package = frost::keys::KeyPackage::try_from(secret_share.clone())
        .map_err(|e| QuorumError::Protocol(e.to_string()))?;
    frost::round2::sign(package, nonces, &key_package)
        .map_err(|e| QuorumError::Protocol(e.to_string()))
}

/// Aggregate signature shares into the single group signature (64 bytes).
///
/// # Errors
///
/// Returns [`QuorumError::BelowThreshold`] for under-threshold aggregation
/// and [`QuorumError::Protocol`] for malformed or inconsistent shares.
pub fn aggregate_signature(
    package: &SigningPackage,
    shares: &[(ParticipantId, frost::round2::SignatureShare)],
    public_key_package: &PublicKeyPackage,
    threshold: u16,
) -> Result<[u8; 64], QuorumError> {
    if shares.len() < usize::from(threshold) {
        return Err(QuorumError::BelowThreshold {
            expected: usize::from(threshold),
            got: shares.len(),
        });
    }
    let mut map = BTreeMap::new();
    for (participant, share) in shares {
        map.insert(participant.to_frost()?, *share);
    }
    let signature = aggregate(package, &map, public_key_package)
        .map_err(|e| QuorumError::Protocol(e.to_string()))?;
    let bytes = signature
        .serialize()
        .map_err(|_| QuorumError::MalformedKeyMaterial)?;
    bytes
        .as_slice()
        .try_into()
        .map_err(|_| QuorumError::MalformedKeyMaterial)
}

/// Verify a quorum signature against the group public key.
///
/// # Errors
///
/// Returns [`QuorumError::VerificationFailed`] (not a panic, not a bool) so
/// the refusal is typed and loggable.
pub fn verify_quorum(
    group_public_key: &GroupPublicKey,
    message: &[u8],
    signature_bytes: &[u8; 64],
) -> Result<(), QuorumError> {
    let verifying_key = VerifyingKey::deserialize(&group_public_key.0)
        .map_err(|_| QuorumError::MalformedKeyMaterial)?;
    let signature =
        Signature::deserialize(signature_bytes).map_err(|_| QuorumError::MalformedKeyMaterial)?;
    verifying_key
        .verify(message, &signature)
        .map_err(|_| QuorumError::VerificationFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::OsRng;

    fn sign_with_quorum(message: &[u8], participants: usize) -> (DealtQuorum, [u8; 64]) {
        let quorum = generate_quorum(5, 3, &mut OsRng).expect("dealer");
        let chosen: Vec<ParticipantId> = quorum.shares.keys().copied().take(participants).collect();

        let mut commitments = Vec::new();
        let mut nonces = Vec::new();
        for p in &chosen {
            let (n, c) = round1_commit(&quorum.shares[p], &mut OsRng).expect("round1");
            nonces.push(n);
            commitments.push((*p, c));
        }
        let package = signing_package(&commitments, message, quorum.threshold).expect("package");

        let mut shares = Vec::new();
        for p in chosen.iter() {
            let share =
                round2_sign(&quorum.shares[p], &nonces[shares.len()], &package).expect("round2");
            shares.push((*p, share));
        }
        let signature = aggregate_signature(
            &package,
            &shares,
            &quorum.public_key_package,
            quorum.threshold,
        )
        .expect("aggregate");
        (quorum, signature)
    }

    #[test]
    fn three_of_five_quorum_signs_and_group_key_verifies() {
        let message = b"SELECT -> CONSTRUCT proposal digest";
        let (quorum, signature) = sign_with_quorum(message, 3);
        assert!(verify_quorum(&quorum.group_key, message, &signature).is_ok());
    }

    #[test]
    fn signature_binds_the_message() {
        let (quorum, signature) = sign_with_quorum(b"admit this", 3);
        assert_eq!(
            verify_quorum(&quorum.group_key, b"admit THAT", &signature),
            Err(QuorumError::VerificationFailed)
        );
    }

    #[test]
    fn group_key_binds_the_quorum() {
        let (quorum, signature) = sign_with_quorum(b"admit this", 3);
        let other = generate_quorum(5, 3, &mut OsRng).expect("other dealer");
        assert_ne!(quorum.group_key, other.group_key);
        assert_eq!(
            verify_quorum(&other.group_key, b"admit this", &signature),
            Err(QuorumError::VerificationFailed)
        );
    }

    #[test]
    fn under_threshold_aggregation_is_typed_refusal() {
        let quorum = generate_quorum(5, 3, &mut OsRng).expect("dealer");
        let err = signing_package(&[], b"m", quorum.threshold);
        assert_eq!(
            err,
            Err(QuorumError::BelowThreshold {
                expected: 3,
                got: 0
            })
        );
    }

    #[test]
    fn zero_participant_index_is_refused() {
        assert!(ParticipantId(0).to_frost().is_err());
    }
}
