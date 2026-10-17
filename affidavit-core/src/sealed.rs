//! The PQ-sealed-receipt capability: bind a receipt chain head to an ML-DSA-65
//! signature and certify the binding through the [`PqcCourt`] verifier seam.
//!
//! Operator law: **all post-quantum crypto comes from external validation
//! dependencies — no hand-rolled PQ arithmetic in this crate, ever.** This
//! module implements only the *envelope*: it fixes the domain-separated
//! message that a consumer must sign, and on unseal it routes the sealed
//! bytes through the injected [`MlDsa65Verifier`](crate::pqc::MlDsa65Verifier) and reports what it said.
//! **This crate has no signer role** — signing belongs to consumers (the root
//! `affi` CLI signs; this crate is the verify surface only).
//!
//! ## Envelope format
//!
//! The signed message is the fixed domain-separation tag
//! [`DOMAIN_SEAL`] concatenated with the 32-byte BLAKE3 chain head (the
//! `Receipt::chain_hash()` of the receipt being sealed — wire the accessor to
//! `crate::chain::Receipt` at integration; this module stays standalone over
//! the raw `[u8; 32]` head). The [`DOMAIN_SEAL`] tag keeps a seal message from
//! ever colliding with any other protocol message signed under the same key.
//!
//! The sealing key is bound into the envelope by fingerprint:
//! [`key_id`] is `blake3(DOMAIN_KEY_ID || pk)` — a domain-separated BLAKE3
//! digest consumers can publish alongside the seal so verifiers can pin the
//! expected key without trusting envelope-carried key material.
//!
//! ## Refusals, typed
//!
//! * zeroed chain head (the canonical "empty chain" encoding) — refused at
//!   construction.
//! * wrong ML-DSA-65 public-key length — refused at construction.
//! * malformed signature encoding — refused at construction.
//! * verifier rejects `(msg, sig, pk)` — [`SealRefusal::MlDsa65Rejected`] from
//!   [`PqcSealedReceipt::unseal`].
//! * key fingerprint mismatch under an expected id —
//!   [`SealRefusal::KeyIdMismatch`].
//!
//! integration: coordinator adds `pub mod sealed;` to src/lib.rs

use crate::pqc::{PqcCourt, PqcProfile, PqcVerdict, Suite};
use blake3::Hasher;

/// Domain-separation tag prepended to every sealed message. Fixed forever;
/// changing it invalidates every existing seal by design (the envelope format
/// version *is* the tag).
pub const DOMAIN_SEAL: &[u8] = b"affidavit-core/seal/v1";

/// Domain-separation tag for the key fingerprint ([`key_id`]).
pub const DOMAIN_KEY_ID: &[u8] = b"affidavit-core/seal/key-id/v1";

/// The FIPS 204 ML-DSA-65 encoded public-key length in bytes.
pub const ML_DSA_65_PK_LEN: usize = 1952;

/// The FIPS 204 ML-DSA-65 encoded signature length in bytes.
pub const ML_DSA_65_SIG_LEN: usize = 3309;

/// Length of [`seal_message`] output: [`DOMAIN_SEAL`] (23 bytes) + 32-byte
/// chain head.
pub const SEAL_MSG_LEN: usize = DOMAIN_SEAL.len() + 32;

/// The domain-separated message a consumer must sign to seal the chain head
/// `head`: `DOMAIN_SEAL || head`. Returned by value; no allocation.
///
/// Signing is out of scope here by operator law — feed this exact byte string
/// to your external ML-DSA-65 signer, then hand the resulting signature to
/// [`PqcSealedReceipt::new`].
pub const fn seal_message(head: [u8; 32]) -> [u8; SEAL_MSG_LEN] {
    let mut msg = [0u8; SEAL_MSG_LEN];
    let mut i = 0;
    while i < DOMAIN_SEAL.len() {
        msg[i] = DOMAIN_SEAL[i];
        i += 1;
    }
    while i < SEAL_MSG_LEN {
        msg[i] = head[i - DOMAIN_SEAL.len()];
        i += 1;
    }
    msg
}

/// The domain-separated key fingerprint `blake3(DOMAIN_KEY_ID || pk)` of a
/// sealing key. Publish this alongside seals so verifiers can pin the
/// expected signer without trusting envelope-carried keys.
pub fn key_id(pk: &[u8]) -> [u8; 32] {
    let mut h = Hasher::new();
    h.update(DOMAIN_KEY_ID);
    h.update(pk);
    *h.finalize().as_bytes()
}

/// Typed refusal of a PQ seal, with the reason class and no cryptographic
/// claim beyond what was actually observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SealRefusal {
    /// The chain head is the all-zero encoding — the canonical refusal for
    /// sealing over an empty chain (a real `chain_hash` is never zero).
    EmptyChainHead,
    /// The public key is not a valid encoded ML-DSA-65 key length.
    WrongPkLength {
        /// The offending length in bytes.
        got: usize,
    },
    /// The signature is not a valid encoded ML-DSA-65 signature length.
    MalformedSignature {
        /// The offending length in bytes.
        got: usize,
    },
    /// The supplied verifier rejected the seal (or collapsed a malformed
    /// input to rejection) — the seal does not certify the chain head.
    MlDsa65Rejected,
    /// The key fingerprint of the sealing key does not match the expected
    /// `key_id` the verifier pinned.
    KeyIdMismatch {
        /// The fingerprint of the key actually supplied.
        got: [u8; 32],
    },
}

impl core::fmt::Display for SealRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SealRefusal::EmptyChainHead => f.write_str("refusing to seal over an empty chain head"),
            SealRefusal::WrongPkLength { got } => {
                write!(f, "ML-DSA-65 pk length {got} != {ML_DSA_65_PK_LEN}")
            }
            SealRefusal::MalformedSignature { got } => {
                write!(f, "ML-DSA-65 signature length {got} != {ML_DSA_65_SIG_LEN}")
            }
            SealRefusal::MlDsa65Rejected => f.write_str("ML-DSA-65 verifier rejected the seal"),
            SealRefusal::KeyIdMismatch { got } => write!(f, "key id mismatch: {got:02x?}"),
        }
    }
}

impl core::error::Error for SealRefusal {}

/// A PQ-sealed receipt envelope: a 32-byte chain head bound to an ML-DSA-65
/// signature over [`seal_message`]`(head)` under the encoded public key `pk`.
///
/// Borrowing, `no_std`, allocation-free: the signature and key are borrowed
/// for `'envelope`. Construction validates lengths and refuses an empty chain
/// head; certification happens only in [`PqcSealedReceipt::unseal`] through a
/// caller-supplied [`PqcCourt`]. This type never signs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PqcSealedReceipt<'envelope> {
    head: [u8; 32],
    sig: &'envelope [u8],
    pk: &'envelope [u8],
}

impl<'envelope> PqcSealedReceipt<'envelope> {
    /// Assemble a seal envelope, refusing the typed-malformed cases up front:
    /// zeroed chain head, wrong pk length, wrong signature length.
    pub fn new(
        head: [u8; 32],
        sig: &'envelope [u8],
        pk: &'envelope [u8],
    ) -> Result<Self, SealRefusal> {
        if head == [0u8; 32] {
            return Err(SealRefusal::EmptyChainHead);
        }
        if pk.len() != ML_DSA_65_PK_LEN {
            return Err(SealRefusal::WrongPkLength { got: pk.len() });
        }
        if sig.len() != ML_DSA_65_SIG_LEN {
            return Err(SealRefusal::MalformedSignature { got: sig.len() });
        }
        Ok(Self { head, sig, pk })
    }

    /// The sealed chain head (the receipt's `chain_hash()`).
    pub const fn chain_head(&self) -> [u8; 32] {
        self.head
    }

    /// The domain-separated sealed message — exactly the bytes whose
    /// ML-DSA-65 signature this envelope carries.
    pub const fn sealed_message(&self) -> [u8; SEAL_MSG_LEN] {
        seal_message(self.head)
    }

    /// The sealing key's domain-separated fingerprint ([`key_id`]).
    pub fn key_fingerprint(&self) -> [u8; 32] {
        key_id(self.pk)
    }

    /// Check the sealing key against a pinned, expected `key_id`.
    /// `Ok(())` iff [`PqcSealedReceipt::key_fingerprint`] equals `expected`.
    pub fn check_key_id(&self, expected: [u8; 32]) -> Result<(), SealRefusal> {
        let got = self.key_fingerprint();
        if got == expected {
            Ok(())
        } else {
            Err(SealRefusal::KeyIdMismatch { got })
        }
    }

    /// Certify the seal through the court: run the domain-separated sealed
    /// message, the signature, and the pk through the court's registered
    /// [`MlDsa65Verifier`](crate::pqc::MlDsa65Verifier). `Accept` means the external verifier accepted the
    /// binding; any rejection (including malformed bytes that collapse to
    /// `false` inside the verifier) is [`SealRefusal::MlDsa65Rejected`].
    ///
    /// The court is consulted, never bypassed: a [`PqcCourt`] with no
    /// ML-DSA-65 verifier registered refuses — the seal is never "accepted
    /// for lack of a checker".
    pub fn unseal(&self, court: &PqcCourt) -> Result<PqcVerdict, SealRefusal> {
        if !court.has_ml_dsa() {
            return Err(SealRefusal::MlDsa65Rejected);
        }
        let msg = self.sealed_message();
        match court.judge(
            PqcProfile::Pqc(Suite::MlDsa65),
            &msg,
            (self.sig, self.pk),
            (&[], &[]),
        ) {
            PqcVerdict::Accept => Ok(PqcVerdict::Accept),
            PqcVerdict::Refuse(_) => Err(SealRefusal::MlDsa65Rejected),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pqc::MlDsa65Verifier;

    /// In-test fake for the *seam* only — accepts exactly one signature over
    /// exactly one message. The REAL FIPS 204 validation lives in
    /// `tests/pqc_external_validation.rs` against the external `ml-dsa` crate;
    /// the malformed-refusal paths below additionally run through
    /// [`crate::pqc::ExternalMlDsa65`] directly (no real signing in-crate —
    /// signing belongs to consumers).
    struct FixedChecker {
        accept_msg: &'static [u8],
        accept_sig: &'static [u8],
        calls: core::sync::atomic::AtomicU32,
    }

    impl FixedChecker {
        const fn new(accept_msg: &'static [u8], accept_sig: &'static [u8]) -> Self {
            Self {
                accept_msg,
                accept_sig,
                calls: core::sync::atomic::AtomicU32::new(0),
            }
        }
    }

    impl MlDsa65Verifier for FixedChecker {
        fn verify(&self, msg: &[u8], sig: &[u8], _pk: &[u8]) -> bool {
            self.calls
                .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
            msg == self.accept_msg && sig == self.accept_sig
        }
    }

    static HEAD: [u8; 32] = [7u8; 32];
    static PK: [u8; ML_DSA_65_PK_LEN] = [3u8; ML_DSA_65_PK_LEN];
    static SIG: [u8; ML_DSA_65_SIG_LEN] = [5u8; ML_DSA_65_SIG_LEN];

    static CHECKER: FixedChecker = FixedChecker::new(&seal_message(HEAD), &SIG);

    fn seal() -> PqcSealedReceipt<'static> {
        PqcSealedReceipt::new(HEAD, &SIG, &PK).expect("valid envelope")
    }

    fn court() -> PqcCourt {
        PqcCourt::new().with_ml_dsa(&CHECKER)
    }

    #[test]
    fn seal_message_is_domain_separated_concat() {
        let msg = seal_message(HEAD);
        assert_eq!(msg.len(), SEAL_MSG_LEN);
        assert_eq!(&msg[..DOMAIN_SEAL.len()], DOMAIN_SEAL);
        assert_eq!(&msg[DOMAIN_SEAL.len()..], &HEAD);
        // a different head yields a different message; the tag is a prefix of
        // both, so the separation is what prevents cross-protocol reuse
        let other = seal_message([1u8; 32]);
        assert_ne!(msg, other);
        assert_eq!(&other[..DOMAIN_SEAL.len()], DOMAIN_SEAL);
    }

    #[test]
    fn key_id_is_domain_separated_and_key_sensitive() {
        let a = key_id(&PK);
        let b = key_id(&[4u8; ML_DSA_65_PK_LEN]);
        assert_ne!(a, b);
        // domain separation: same pk bytes under the other domain tag differ
        let mut h = Hasher::new();
        h.update(b"affidavit-core/other-domain");
        h.update(&PK);
        assert_ne!(a, *h.finalize().as_bytes());
    }

    #[test]
    fn empty_chain_head_is_refused() {
        assert_eq!(
            PqcSealedReceipt::new([0u8; 32], &SIG, &PK),
            Err(SealRefusal::EmptyChainHead)
        );
    }

    #[test]
    fn wrong_pk_length_is_refused() {
        assert_eq!(
            PqcSealedReceipt::new(HEAD, &SIG, &[3u8; 1951]),
            Err(SealRefusal::WrongPkLength { got: 1951 })
        );
        assert_eq!(
            PqcSealedReceipt::new(HEAD, &SIG, &[]),
            Err(SealRefusal::WrongPkLength { got: 0 })
        );
    }

    #[test]
    fn malformed_signature_length_is_refused() {
        assert_eq!(
            PqcSealedReceipt::new(HEAD, &[5u8; 10], &PK),
            Err(SealRefusal::MalformedSignature { got: 10 })
        );
    }

    #[test]
    fn unseal_accepts_through_the_registered_verifier() {
        let s = seal();
        assert_eq!(s.chain_head(), HEAD);
        assert_eq!(s.sealed_message(), seal_message(HEAD));
        assert_eq!(s.key_fingerprint(), key_id(&PK));
        assert_eq!(s.check_key_id(key_id(&PK)), Ok(()));
        assert_eq!(s.unseal(&court()), Ok(PqcVerdict::Accept));
        // the verifier behind the court was actually reached, not skipped
        assert!(CHECKER.calls.load(core::sync::atomic::Ordering::Relaxed) >= 1);
    }

    #[test]
    fn unseal_refuses_forged_signature_via_fixed_fake() {
        static FORGED: [u8; ML_DSA_65_SIG_LEN] = [6u8; ML_DSA_65_SIG_LEN];
        let s = PqcSealedReceipt::new(HEAD, &FORGED, &PK).expect("lengths valid");
        // fail-fast: exactly one verifier call for this judgement
        let before = CHECKER.calls.load(core::sync::atomic::Ordering::Relaxed);
        assert_eq!(s.unseal(&court()), Err(SealRefusal::MlDsa65Rejected));
        let after = CHECKER.calls.load(core::sync::atomic::Ordering::Relaxed);
        assert_eq!(after - before, 1);
    }

    #[test]
    fn unseal_refuses_when_court_has_no_verifier() {
        let court = PqcCourt::new();
        assert_eq!(seal().unseal(&court), Err(SealRefusal::MlDsa65Rejected));
    }

    #[test]
    fn key_id_mismatch_is_typed() {
        let s = seal();
        let expected = key_id(&[9u8; ML_DSA_65_PK_LEN]);
        assert_eq!(
            s.check_key_id(expected),
            Err(SealRefusal::KeyIdMismatch { got: key_id(&PK) })
        );
    }

    #[test]
    fn external_mldsa65_refuses_malformed_seal_without_panic() {
        // Real external adapter, hostile bytes: wrong pk length collapses to
        // rejection inside the verifier, and a length-valid but garbage
        // signature over the real sealed message is rejected by FIPS 204
        // arithmetic — no real signing in-crate, none needed for the refusal.
        let msg = seal_message(HEAD);
        assert!(!crate::pqc::ExternalMlDsa65.verify(&msg, &SIG, &[0u8; 1951]));
        assert!(!crate::pqc::ExternalMlDsa65.verify(&msg, &SIG, &PK));
    }

    #[test]
    fn refusal_display_is_nonempty() {
        assert!(!SealRefusal::EmptyChainHead.to_string().is_empty());
        assert!(!SealRefusal::WrongPkLength { got: 1 }.to_string().is_empty());
        assert!(!SealRefusal::MalformedSignature { got: 1 }
            .to_string()
            .is_empty());
        assert!(!SealRefusal::MlDsa65Rejected.to_string().is_empty());
        assert!(!SealRefusal::KeyIdMismatch { got: [0u8; 32] }
            .to_string()
            .is_empty());
    }
}
