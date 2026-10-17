//! The post-quantum verifier *seam*: trait contracts for external FIPS 204 /
//! FIPS 205 validation dependencies, plus a profile-gated fail-fast court over
//! them.
//!
//! Operator law: **all post-quantum crypto comes from external validation
//! dependencies — no hand-rolled PQ arithmetic in this crate, ever.** This
//! crate carries no cryptographic arithmetic in-crate by invariant, so the core defines only the *seam*:
//!
//! * [`MlDsa65Verifier`] — the FIPS 204 ML-DSA-65 verification contract.
//! * [`SlhDsaVerifier`] — the FIPS 205 SLH-DSA-SHA2-128s verification
//!   contract.
//! * [`Suite`] — which algorithm(s) a [`PqcProfile::Pqc`] envelope demands:
//!   `MlDsa65`, `SlhDsa128s`, or `Dual` (both, fail-fast).
//! * [`PqcCourt`] — runs the fail-fast check chain over envelope bytes and
//!   the registered verifiers, returning a typed [`PqcVerdict`] /
//!   [`PqcRefusal`].
//!
//! The algorithm surface is fixed by operator law to **ML-DSA-65 +
//! SLH-DSA-SHA2-128s + BLAKE3 only**: the signature algorithms are FIPS 204
//! ML-DSA-65 and FIPS 205 SLH-DSA-SHA2-128s, and the sole cryptographic chain
//! hash is BLAKE3 (see `crate::digest`). [`ExternalMlDsa65`] adapts the
//! external `ml-dsa` crate and [`ExternalSlhDsa128s`] adapts the external
//! `slh-dsa` crate into the seam so the court can ship in real artifacts
//! (including `affidavit-wasm`). Nothing here performs arithmetic on lattice
//! or hash-tree internals; it routes bytes to the injected verifiers and
//! reports what they said.
//!
//! ## Doctrine, preserved
//!
//! *Certify, don't decide.* The court checks that the *right verifiers were
//! supplied and reached* for the declared profile/suite and reports their
//! answers. It never manufactures a cryptographic judgment of its own.
//!
//! integration: `pub mod pqc;` is declared in src/lib.rs (coordinator-owned);
//! the coordinator extends the lib.rs re-export with `SlhDsaVerifier`,
//! `ExternalSlhDsa128s`, and `Suite`.


use core::fmt;

/// The FIPS 204 (ML-DSA-65) verification contract, implemented by an external
/// validation dependency — never by this crate.
///
/// Implementations must hold: a `true` return means the external library
/// accepted `(msg, sig)` under `pk` per FIPS 204; any parse, length, or
/// arithmetic failure collapses to `false`. The court treats `false` as a
/// typed refusal, never retries, never infers.
pub trait MlDsa65Verifier {
    /// Verify a FIPS 204 ML-DSA-65 signature over `msg` against `pk`.
    /// Returns `false` for any malformed input or failed verification.
    fn verify(&self, msg: &[u8], sig: &[u8], pk: &[u8]) -> bool;
}

/// The FIPS 205 (SLH-DSA-SHA2-128s) verification contract, implemented by an
/// external validation dependency — never by this crate.
///
/// Same boolean-collapsed contract as [`MlDsa65Verifier`]: a `true` return
/// means the external library accepted `(msg, sig)` under `pk` per FIPS 205;
/// any parse, length, or hash-tree failure collapses to `false`. The court
/// treats `false` as a typed refusal, never retries, never infers.
pub trait SlhDsaVerifier {
    /// Verify a FIPS 205 SLH-DSA-SHA2-128s signature over `msg` against `pk`.
    /// Returns `false` for any malformed-input or failed verification.
    fn verify(&self, msg: &[u8], sig: &[u8], pk: &[u8]) -> bool;
}

/// Zero-size adapter implementing [`MlDsa65Verifier`] over the **external**
/// `ml-dsa` crate (FIPS 204). Unconditional by operator law (no PQC feature
/// flag); the algorithm is always the dependency's, never this crate's.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExternalMlDsa65;

impl MlDsa65Verifier for ExternalMlDsa65 {
    fn verify(&self, msg: &[u8], sig: &[u8], pk: &[u8]) -> bool {
        use ml_dsa::{MlDsa65, Signature, VerifyingKey};
        // The encoded verifying key is exactly 1952 bytes for ML-DSA-65;
        // anything else is malformed and collapses to `false`.
        let Ok(vk_enc) = <[u8; 1952]>::try_from(pk) else {
            return false;
        };
        let vk = VerifyingKey::<MlDsa65>::decode(&vk_enc.into());
        let Ok(sigma) = Signature::<MlDsa65>::try_from(sig) else {
            return false;
        };
        vk.verify_with_context(msg, &[], &sigma)
    }
}

/// Zero-size adapter implementing [`SlhDsaVerifier`] over the **external**
/// `slh-dsa` crate (FIPS 205, SLH-DSA-SHA2-128s parameter set). Unconditional
/// by operator law (no feature flag); the algorithm is always the
/// dependency's, never this crate's.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExternalSlhDsa128s;

impl SlhDsaVerifier for ExternalSlhDsa128s {
    fn verify(&self, msg: &[u8], sig: &[u8], pk: &[u8]) -> bool {
        use slh_dsa::{Sha2_128s, Signature, VerifyingKey};
        // SLH-DSA-SHA2-128s public key is 2n = 32 bytes; the crate's
        // `TryFrom<&[u8]>` for `VerifyingKey` enforces the exact length, and
        // anything else is malformed and collapses to `false`.
        let Ok(vk) = VerifyingKey::<Sha2_128s>::try_from(pk) else {
            return false;
        };
        // The encoded SLH-DSA-SHA2-128s signature is 7856 bytes; the crate's
        // `TryFrom<&[u8]>` enforces the exact length.
        let Ok(sigma) = Signature::<Sha2_128s>::try_from(sig) else {
            return false;
        };
        use slh_dsa::signature::Verifier;
        vk.verify(msg, &sigma).is_ok()
    }
}

/// Which post-quantum algorithm(s) a [`PqcProfile::Pqc`] envelope demands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suite {
    /// FIPS 204 ML-DSA-65 only.
    MlDsa65,
    /// FIPS 205 SLH-DSA-SHA2-128s only.
    SlhDsa128s,
    /// Both: ML-DSA-65 AND SLH-DSA-SHA2-128s must each verify (fail-fast).
    Dual,
}

impl fmt::Display for Suite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Suite::MlDsa65 => "ml-dsa-65",
            Suite::SlhDsa128s => "slh-dsa-sha2-128s",
            Suite::Dual => "dual (ml-dsa-65 + slh-dsa-sha2-128s)",
        };
        f.write_str(name)
    }
}

/// The declared post-quantum assurance level of an envelope.
///
/// `Classical` demands no PQ verifier. `Hybrid` demands the ML-DSA-65
/// verifier in addition to any classical check the caller performs itself.
/// `Pqc(suite)` demands exactly the verifier(s) the named [`Suite`] names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PqcProfile {
    /// Pre-quantum signatures only; no PQ verifier is consulted.
    Classical,
    /// Dual assurance: FIPS 204 ML-DSA-65 must verify in addition to any
    /// classical check performed by the caller.
    Hybrid,
    /// Post-quantum: the named [`Suite`]'s algorithm(s) must verify.
    Pqc(Suite),
}

impl fmt::Display for PqcProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            PqcProfile::Classical => "classical",
            PqcProfile::Hybrid => "hybrid",
            PqcProfile::Pqc(Suite::MlDsa65) => "pqc(ml-dsa-65)",
            PqcProfile::Pqc(Suite::SlhDsa128s) => "pqc(slh-dsa-sha2-128s)",
            PqcProfile::Pqc(Suite::Dual) => "pqc(dual)",
        };
        f.write_str(name)
    }
}

/// Typed refusal of an envelope, with the reason class and no cryptographic
/// claim beyond what the court actually observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PqcRefusal {
    /// The profile demands ML-DSA-65 but no [`MlDsa65Verifier`] was supplied.
    MissingMlDsa65Verifier,
    /// The supplied ML-DSA-65 verifier rejected the signature (or the inputs
    /// were malformed and it collapsed to `false`).
    MlDsa65Rejected,
    /// The suite demands SLH-DSA-SHA2-128s but no [`SlhDsaVerifier`] was
    /// supplied.
    MissingSlhDsaVerifier,
    /// The supplied SLH-DSA verifier rejected the signature (or the inputs
    /// were malformed and it collapsed to `false`).
    SlhDsaRejected,
}

impl fmt::Display for PqcRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PqcRefusal::MissingMlDsa65Verifier => {
                f.write_str("profile demands ML-DSA-65 but no verifier was supplied")
            }
            PqcRefusal::MlDsa65Rejected => f.write_str("ML-DSA-65 verifier rejected the signature"),
            PqcRefusal::MissingSlhDsaVerifier => {
                f.write_str("suite demands SLH-DSA-SHA2-128s but no verifier was supplied")
            }
            PqcRefusal::SlhDsaRejected => {
                f.write_str("SLH-DSA-SHA2-128s verifier rejected the signature")
            }
        }
    }
}

impl core::error::Error for PqcRefusal {}

/// The outcome of a [`PqcCourt`] run: accepted, or a typed [`PqcRefusal`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PqcVerdict {
    /// Every check the declared profile demands passed against the supplied
    /// external verifiers.
    Accept,
    /// The first failing check in the fail-fast chain, as a typed refusal.
    Refuse(PqcRefusal),
}

/// The fail-fast certification court over the PQ verifier seam.
///
/// Given a declared [`PqcProfile`], a message, its signature(s), public
/// key(s), and whatever boxed verifiers the caller injected, the court runs
/// only the checks the profile demands, in order, and stops at the first
/// failure. It *certifies* the outcome of the external verifiers; it never
/// *decides* cryptography itself.
#[derive(Default)]
pub struct PqcCourt {
    ml_dsa: Option<&'static dyn MlDsa65Verifier>,
    slh_dsa: Option<&'static dyn SlhDsaVerifier>,
}

impl fmt::Debug for PqcCourt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PqcCourt")
            .field("has_ml_dsa", &self.ml_dsa.is_some())
            .field("has_slh_dsa", &self.slh_dsa.is_some())
            .finish()
    }
}

impl PqcCourt {
    /// Build an empty court. Register verifiers with [`PqcCourt::with_ml_dsa`]
    /// and/or [`PqcCourt::with_slh_dsa`]; a profile demanding an algorithm
    /// with no verifier present is refused, never skipped.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register the FIPS 204 verifier (typically the [`ExternalMlDsa65`]
    /// adapter, or a `'static` adapter over an external validation
    /// dependency). Builder-style.
    pub fn with_ml_dsa(mut self, verifier: &'static dyn MlDsa65Verifier) -> Self {
        self.ml_dsa = Some(verifier);
        self
    }

    /// Register the FIPS 205 SLH-DSA-SHA2-128s verifier (typically the
    /// [`ExternalSlhDsa128s`] adapter, or a `'static` adapter over an
    /// external validation dependency). Builder-style.
    pub fn with_slh_dsa(mut self, verifier: &'static dyn SlhDsaVerifier) -> Self {
        self.slh_dsa = Some(verifier);
        self
    }

    /// Whether an ML-DSA-65 verifier has been registered.
    pub fn has_ml_dsa(&self) -> bool {
        self.ml_dsa.is_some()
    }

    /// Whether an SLH-DSA-SHA2-128s verifier has been registered.
    pub fn has_slh_dsa(&self) -> bool {
        self.slh_dsa.is_some()
    }

    /// Run the fail-fast chain: profile admission first (typed refusal when a
    /// demanded verifier is absent), then the external verification(s) the
    /// profile demands, stopping at the first refusal.
    ///
    /// Check order for [`Suite::Dual`]: ML-DSA-65 first, then
    /// SLH-DSA-SHA2-128s. No classical (non-PQC) algorithm is ever consulted
    /// — by operator law the signature surface is PQC-only; `Hybrid` means
    /// "ML-DSA-65 in addition to whatever classical check the caller
    /// performed itself", which for this court is the same check as
    /// [`PqcProfile::Pqc`]`(`[`Suite::MlDsa65`]`)`.
    pub fn judge(
        &self,
        profile: PqcProfile,
        msg: &[u8],
        ml_dsa: (&[u8], &[u8]),
        slh_dsa: (&[u8], &[u8]),
    ) -> PqcVerdict {
        match profile {
            PqcProfile::Classical => PqcVerdict::Accept,
            PqcProfile::Hybrid | PqcProfile::Pqc(Suite::MlDsa65) => self.judge_mldsa(msg, ml_dsa),
            PqcProfile::Pqc(Suite::SlhDsa128s) => self.judge_slh(msg, slh_dsa),
            PqcProfile::Pqc(Suite::Dual) => match self.judge_mldsa(msg, ml_dsa) {
                refuse @ PqcVerdict::Refuse(_) => refuse,
                PqcVerdict::Accept => self.judge_slh(msg, slh_dsa),
            },
        }
    }

    /// The ML-DSA-65 check that the `Hybrid` / `Pqc(MlDsa65)` / `Pqc(Dual)`
    /// profiles demand.
    fn judge_mldsa(&self, msg: &[u8], (sig, pk): (&[u8], &[u8])) -> PqcVerdict {
        let Some(v) = self.ml_dsa else {
            return PqcVerdict::Refuse(PqcRefusal::MissingMlDsa65Verifier);
        };
        if !v.verify(msg, sig, pk) {
            return PqcVerdict::Refuse(PqcRefusal::MlDsa65Rejected);
        }
        PqcVerdict::Accept
    }

    /// The SLH-DSA-SHA2-128s check that the `Pqc(SlhDsa128s)` / `Pqc(Dual)`
    /// profiles demand.
    fn judge_slh(&self, msg: &[u8], (sig, pk): (&[u8], &[u8])) -> PqcVerdict {
        let Some(v) = self.slh_dsa else {
            return PqcVerdict::Refuse(PqcRefusal::MissingSlhDsaVerifier);
        };
        if !v.verify(msg, sig, pk) {
            return PqcVerdict::Refuse(PqcRefusal::SlhDsaRejected);
        }
        PqcVerdict::Accept
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// In-test fakes for the *seam* only — fixed-response checkers that
    /// accept exactly one signature. The REAL FIPS 204/205 validation lives
    /// in `tests/pqc_external_validation.rs` against external dependencies.
    struct FixedChecker {
        accept_sig: &'static [u8],
        calls: core::sync::atomic::AtomicU32,
    }

    impl FixedChecker {
        const fn new(accept_sig: &'static [u8]) -> Self {
            Self {
                accept_sig,
                calls: core::sync::atomic::AtomicU32::new(0),
            }
        }

        fn count(&self) -> u32 {
            self.calls.load(core::sync::atomic::Ordering::Relaxed)
        }
    }

    impl MlDsa65Verifier for FixedChecker {
        fn verify(&self, _msg: &[u8], sig: &[u8], _pk: &[u8]) -> bool {
            self.calls
                .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
            sig == self.accept_sig
        }
    }

    impl SlhDsaVerifier for FixedChecker {
        fn verify(&self, _msg: &[u8], sig: &[u8], _pk: &[u8]) -> bool {
            self.calls
                .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
            sig == self.accept_sig
        }
    }

    static ML_OK: FixedChecker = FixedChecker::new(b"good-ml-dsa-sig");
    static SLH_OK: FixedChecker = FixedChecker::new(b"good-slh-dsa-sig");

    const MSG: &[u8] = b"envelope bytes";

    #[test]
    fn classical_profile_accepts_without_pq_verifiers() {
        let court = PqcCourt::new();
        assert_eq!(
            court.judge(PqcProfile::Classical, MSG, (b"x", b"y"), (b"x", b"y")),
            PqcVerdict::Accept
        );
    }

    #[test]
    fn pqc_profile_refuses_when_no_verifier_supplied() {
        let court = PqcCourt::new();
        for (profile, refusal) in [
            (
                PqcProfile::Pqc(Suite::MlDsa65),
                PqcRefusal::MissingMlDsa65Verifier,
            ),
            (PqcProfile::Hybrid, PqcRefusal::MissingMlDsa65Verifier),
            (
                PqcProfile::Pqc(Suite::SlhDsa128s),
                PqcRefusal::MissingSlhDsaVerifier,
            ),
            (
                PqcProfile::Pqc(Suite::Dual),
                PqcRefusal::MissingMlDsa65Verifier,
            ),
        ] {
            assert_eq!(
                court.judge(profile, MSG, (b"s", b"k"), (b"s", b"k")),
                PqcVerdict::Refuse(refusal)
            );
        }
    }

    #[test]
    fn forged_signature_is_refused() {
        let court = PqcCourt::new().with_ml_dsa(&ML_OK).with_slh_dsa(&SLH_OK);
        assert_eq!(
            court.judge(
                PqcProfile::Pqc(Suite::MlDsa65),
                MSG,
                (b"forged", b"pk"),
                (b"", b"")
            ),
            PqcVerdict::Refuse(PqcRefusal::MlDsa65Rejected)
        );
        assert_eq!(
            court.judge(
                PqcProfile::Pqc(Suite::SlhDsa128s),
                MSG,
                (b"", b""),
                (b"forged", b"pk")
            ),
            PqcVerdict::Refuse(PqcRefusal::SlhDsaRejected)
        );
        // fail-fast: the SLH verifier was reached for this judgement exactly
        // once and not retried — relative to the counter level before this
        // court run (other tests share the statics and run in parallel).
        let before = SLH_OK.count();
        let _ = court.judge(
            PqcProfile::Pqc(Suite::SlhDsa128s),
            MSG,
            (b"", b""),
            (b"forged", b"pk"),
        );
        assert_eq!(SLH_OK.count() - before, 1);
    }

    #[test]
    fn genuine_signature_path_is_reachable() {
        let court = PqcCourt::new().with_ml_dsa(&ML_OK).with_slh_dsa(&SLH_OK);
        assert_eq!(
            court.judge(
                PqcProfile::Pqc(Suite::MlDsa65),
                MSG,
                (b"good-ml-dsa-sig", b"pk"),
                (b"", b"")
            ),
            PqcVerdict::Accept
        );
        assert_eq!(
            court.judge(
                PqcProfile::Pqc(Suite::SlhDsa128s),
                MSG,
                (b"", b""),
                (b"good-slh-dsa-sig", b"pk")
            ),
            PqcVerdict::Accept
        );
    }

    #[test]
    fn dual_suite_demands_both_fail_fast() {
        let court = PqcCourt::new().with_ml_dsa(&ML_OK).with_slh_dsa(&SLH_OK);
        // Genuine both: accept.
        assert_eq!(
            court.judge(
                PqcProfile::Pqc(Suite::Dual),
                MSG,
                (b"good-ml-dsa-sig", b"pk"),
                (b"good-slh-dsa-sig", b"pk")
            ),
            PqcVerdict::Accept
        );
        // Forged SLH half after a genuine ML-DSA half: SLH refusal.
        assert_eq!(
            court.judge(
                PqcProfile::Pqc(Suite::Dual),
                MSG,
                (b"good-ml-dsa-sig", b"pk"),
                (b"forged", b"pk")
            ),
            PqcVerdict::Refuse(PqcRefusal::SlhDsaRejected)
        );
        // Fail-fast admission: missing ML-DSA verifier stops BEFORE the SLH
        // verifier is consulted at all.
        let bare = PqcCourt::new().with_slh_dsa(&SLH_OK);
        let before = SLH_OK.count();
        assert_eq!(
            bare.judge(
                PqcProfile::Pqc(Suite::Dual),
                MSG,
                (b"s", b"k"),
                (b"s", b"k")
            ),
            PqcVerdict::Refuse(PqcRefusal::MissingMlDsa65Verifier)
        );
        assert_eq!(SLH_OK.count(), before);
    }

    #[test]
    fn hybrid_pass_is_the_mldsa65_check() {
        // Hybrid consults exactly the ML-DSA-65 verifier — the same check as
        // `Pqc(Suite::MlDsa65)` — and nothing else.
        let court = PqcCourt::new().with_ml_dsa(&ML_OK);
        assert_eq!(
            court.judge(
                PqcProfile::Hybrid,
                MSG,
                (b"good-ml-dsa-sig", b"pk"),
                (b"", b"")
            ),
            PqcVerdict::Accept
        );
    }

    #[test]
    fn external_mldsa65_refuses_malformed_inputs_without_panic() {
        // The unconditional external adapter must collapse every malformed
        // input (bad key length, bad signature encoding) to `false`, never
        // panic — hostile bytes are the normal case on the wire.
        assert!(!ExternalMlDsa65.verify(MSG, b"", &[0u8; 1952]));
        assert!(!ExternalMlDsa65.verify(MSG, &[0u8; 3309], &[1u8; 7]));
        assert!(!ExternalMlDsa65.verify(MSG, &[0u8; 3309], &[9u8; 1952]));
    }

    #[test]
    fn external_slh_dsa128s_refuses_malformed_inputs_without_panic() {
        // Same boolean-collapse contract for the FIPS 205 adapter: wrong key
        // length, wrong signature length, or a well-formed-length but invalid
        // key/sig pair all collapse to `false`, never panic.
        assert!(!ExternalSlhDsa128s.verify(MSG, b"", &[0u8; 32]));
        assert!(!ExternalSlhDsa128s.verify(MSG, &[0u8; 7856], &[1u8; 7]));
        assert!(!ExternalSlhDsa128s.verify(MSG, &[0u8; 7856], &[9u8; 32]));
    }

    #[test]
    fn refusal_display_is_nonempty_and_profile_display_names_agree() {
        for refusal in [
            PqcRefusal::MissingMlDsa65Verifier,
            PqcRefusal::MlDsa65Rejected,
            PqcRefusal::MissingSlhDsaVerifier,
            PqcRefusal::SlhDsaRejected,
        ] {
            assert!(!refusal.to_string().is_empty());
        }
        assert_eq!(PqcProfile::Classical.to_string(), "classical");
        assert_eq!(PqcProfile::Hybrid.to_string(), "hybrid");
        assert_eq!(
            PqcProfile::Pqc(Suite::MlDsa65).to_string(),
            "pqc(ml-dsa-65)"
        );
        assert_eq!(
            PqcProfile::Pqc(Suite::SlhDsa128s).to_string(),
            "pqc(slh-dsa-sha2-128s)"
        );
        assert_eq!(PqcProfile::Pqc(Suite::Dual).to_string(), "pqc(dual)");
    }
}
