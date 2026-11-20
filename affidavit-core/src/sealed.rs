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
//! ## Envelope format (v2 — FIPS 204 context-string sealing)
//!
//! The signed **message** is exactly the 32-byte BLAKE3 chain head (the
//! `Receipt::chain_hash()` of the receipt being sealed — wire the accessor to
//! `crate::chain::Receipt` at integration; this module stays standalone over
//! the raw `[u8; 32]` head). The **context string** is [`DOMAIN_SEAL`].
//!
//! FIPS 204 Pure ML-DSA separates `(M, ctx)` inside the signed pre-image. This
//! envelope uses that seam: consumers call
//! `sign_deterministic(seal_message(head), DOMAIN_SEAL)` (or the randomized
//! twin). Tag-stuffing `DOMAIN_SEAL || head` into `M` under an empty context
//! is the v1 anti-pattern this format replaces — a v1 signature MUST fail
//! v2 verification (and vice versa); that divergence is the context-string
//! sealing court.
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
//! * verifier rejects `(msg, ctx, sig, pk)` — [`SealRefusal::MlDsa65Rejected`] from
//!   [`PqcSealedReceipt::unseal`].
//! * key fingerprint mismatch under an expected id —
//!   [`SealRefusal::KeyIdMismatch`].
//!
//! integration: coordinator adds `pub mod sealed;` to src/lib.rs

use crate::pqc::{PqcCourt, PqcVerdict};
use blake3::Hasher;

/// FIPS 204 context string for every sealed message. Fixed forever as the
/// envelope format version; changing it invalidates every existing seal by
/// design.
///
/// v2 law: this byte string is the ML-DSA **context**, never concatenated
/// into the signed message. (v1 stuffed it into `M` under an empty context.)
pub const DOMAIN_SEAL: &[u8] = b"affidavit-core/seal/v2";

/// Domain-separation tag for the key fingerprint ([`key_id`]).
pub const DOMAIN_KEY_ID: &[u8] = b"affidavit-core/seal/key-id/v1";

/// The FIPS 204 ML-DSA-65 encoded public-key length in bytes.
pub const ML_DSA_65_PK_LEN: usize = 1952;

/// The FIPS 204 ML-DSA-65 encoded signature length in bytes.
pub const ML_DSA_65_SIG_LEN: usize = 3309;

/// Length of [`seal_message`] output: the 32-byte chain head alone.
pub const SEAL_MSG_LEN: usize = 32;

/// The FIPS 204 context string a consumer must pass alongside
/// [`seal_message`]. Alias of [`DOMAIN_SEAL`] for call-site clarity.
pub const fn seal_context() -> &'static [u8] {
    DOMAIN_SEAL
}

/// The signed message a consumer must seal over: the raw 32-byte chain head.
///
/// Pair with [`seal_context`] / [`DOMAIN_SEAL`] as the FIPS 204 context
/// string. Signing is out of scope here by operator law — feed these exact
/// bytes to your external ML-DSA-65 signer
/// (`sign_deterministic(seal_message(head), seal_context())`), then hand the
/// resulting signature to [`PqcSealedReceipt::new`].
pub const fn seal_message(head: [u8; 32]) -> [u8; SEAL_MSG_LEN] {
    head
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
/// signature over [`seal_message`]`(head)` under context [`DOMAIN_SEAL`] and
/// the encoded public key `pk`.
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

    /// The FIPS 204 signed message — exactly the chain head bytes.
    pub const fn sealed_message(&self) -> [u8; SEAL_MSG_LEN] {
        seal_message(self.head)
    }

    /// The FIPS 204 context string this envelope was sealed under.
    pub const fn sealed_context(&self) -> &'static [u8] {
        seal_context()
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

    /// Certify the seal through the court: run the sealed message under
    /// [`DOMAIN_SEAL`] context, the signature, and the pk through the court's
    /// registered [`MlDsa65Verifier`](crate::pqc::MlDsa65Verifier). `Accept`
    /// means the external verifier accepted the binding; any rejection
    /// (including malformed bytes that collapse to `false` inside the
    /// verifier) is [`SealRefusal::MlDsa65Rejected`].
    ///
    /// The court is consulted, never bypassed: a [`PqcCourt`] with no
    /// ML-DSA-65 verifier registered refuses — the seal is never "accepted
    /// for lack of a checker".
    pub fn unseal(&self, court: &PqcCourt) -> Result<PqcVerdict, SealRefusal> {
        if !court.has_ml_dsa() {
            return Err(SealRefusal::MlDsa65Rejected);
        }
        let msg = self.sealed_message();
        match court.judge_mldsa_with_context(&msg, self.sealed_context(), (self.sig, self.pk)) {
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
    /// exactly one `(msg, ctx)` pair. The REAL FIPS 204 validation lives in
    /// `tests/pqc_external_validation.rs` against the external `ml-dsa` crate;
    /// the malformed-refusal paths below additionally run through
    /// [`crate::pqc::ExternalMlDsa65`] directly (no real signing in-crate —
    /// signing belongs to consumers).
    struct FixedChecker {
        accept_msg: &'static [u8],
        accept_ctx: &'static [u8],
        accept_sig: &'static [u8],
        calls: core::sync::atomic::AtomicU32,
    }

    impl FixedChecker {
        const fn new(
            accept_msg: &'static [u8],
            accept_ctx: &'static [u8],
            accept_sig: &'static [u8],
        ) -> Self {
            Self {
                accept_msg,
                accept_ctx,
                accept_sig,
                calls: core::sync::atomic::AtomicU32::new(0),
            }
        }
    }

    impl MlDsa65Verifier for FixedChecker {
        fn verify_with_context(&self, msg: &[u8], ctx: &[u8], sig: &[u8], _pk: &[u8]) -> bool {
            self.calls
                .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
            msg == self.accept_msg && ctx == self.accept_ctx && sig == self.accept_sig
        }
    }

    static HEAD: [u8; 32] = [7u8; 32];
    static PK: [u8; ML_DSA_65_PK_LEN] = [3u8; ML_DSA_65_PK_LEN];
    static SIG: [u8; ML_DSA_65_SIG_LEN] = [5u8; ML_DSA_65_SIG_LEN];

    // seal_message(HEAD) == HEAD under v2 law; bind the static head directly.
    static CHECKER: FixedChecker = FixedChecker::new(&HEAD, DOMAIN_SEAL, &SIG);

    fn seal() -> PqcSealedReceipt<'static> {
        PqcSealedReceipt::new(HEAD, &SIG, &PK).expect("valid envelope")
    }

    fn court() -> PqcCourt {
        PqcCourt::new().with_ml_dsa(&CHECKER)
    }

    #[test]
    fn seal_message_is_the_raw_chain_head() {
        let msg = seal_message(HEAD);
        assert_eq!(msg.len(), SEAL_MSG_LEN);
        assert_eq!(msg, HEAD);
        // a different head yields a different message
        let other = seal_message([1u8; 32]);
        assert_ne!(msg, other);
        assert_eq!(other, [1u8; 32]);
    }

    #[test]
    fn seal_context_is_the_domain_tag_not_stuffed_into_msg() {
        assert_eq!(seal_context(), DOMAIN_SEAL);
        assert_eq!(DOMAIN_SEAL, b"affidavit-core/seal/v2");
        // anti-vacuity: the context bytes must NOT appear as a message prefix
        let msg = seal_message(HEAD);
        assert!(!msg.windows(DOMAIN_SEAL.len()).any(|w| w == DOMAIN_SEAL));
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
        assert_eq!(s.sealed_context(), DOMAIN_SEAL);
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
        assert!(!crate::pqc::ExternalMlDsa65.verify_with_context(
            &msg,
            DOMAIN_SEAL,
            &SIG,
            &[0u8; 1951]
        ));
        assert!(!crate::pqc::ExternalMlDsa65.verify_with_context(&msg, DOMAIN_SEAL, &SIG, &PK));
    }

    #[test]
    fn empty_ctx_verify_refuses_context_bound_accept_path() {
        // Seam-level falsifier: the FixedChecker only accepts DOMAIN_SEAL ctx.
        // An empty-ctx judge over the same bytes must refuse — proves the
        // court actually passes context through (not ignoring it).
        let s = seal();
        let c = court();
        assert_eq!(
            c.judge_mldsa_with_context(&s.sealed_message(), &[], (s.sig, s.pk)),
            PqcVerdict::Refuse(crate::pqc::PqcRefusal::MlDsa65Rejected)
        );
        assert_eq!(
            c.judge_mldsa_with_context(&s.sealed_message(), DOMAIN_SEAL, (s.sig, s.pk)),
            PqcVerdict::Accept
        );
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
