//! Federation courts — the reachable CLI surface over the v26.9.x evidence
//! federation kernel.
//!
//! The kernel modules ([`crate::standing`], [`crate::ecosystem`],
//! [`crate::errc`], [`crate::errc_claim_assurance`]) are pure certifiers: each
//! takes an already-admitted Affidavit receipt plus a bounded observation and
//! seals it into a deterministic, content-addressed profile receipt. Before
//! v26.9.6 those certifiers had no operator-reachable seam — the kernel could
//! only be driven from Rust.
//!
//! This module is that seam, and nothing more. It performs **no** adjudication
//! of its own: every accept/refuse decision below is delegated to
//! [`crate::admission::admit`] and to the kernel's own `certify_*` / `verify`
//! laws. `certify, don't decide` holds here exactly as it does in
//! [`crate::verifier`].
//!
//! ## The two court shapes
//!
//! | Shape | Input | Law that runs |
//! |-------|-------|---------------|
//! | `certify` | a `core/v1` receipt + an observation JSON | [`crate::admission::admit`], then the profile's `certify_*` |
//! | `verify` | a sealed profile receipt JSON | the profile's manual `Deserialize`, which re-runs `verify()` |
//!
//! The `verify` shape is load-bearing rather than decorative: each sealed
//! receipt type has a hand-written `Deserialize` that ends in
//! `receipt.verify().map_err(D::Error::custom)?`, so a tampered receipt cannot
//! be parsed into existence at all. Loading *is* the court.
//!
//! ## Exit-code contract
//!
//! Every court returns a [`CourtOutcome`] carrying one of the stable codes from
//! [`crate::diag::exit_codes`]:
//!
//! * `OK` (0) — the law held and a sealed receipt was produced or re-verified.
//! * `REJECT` (2) — a named refusal: admission refused the source receipt, the
//!   kernel refused the observation, or a sealed receipt failed to survive its
//!   own deserialization law.
//! * `USAGE_ERROR` (3) — an *observation* (never a receipt) was not
//!   well-formed JSON for its profile. An observation is an operator input, not
//!   a witness, so a malformed one is a usage error and not a refusal.
//! * `IO_ERROR` (4) — a named path could not be read.
//!
//! A refusal is always a first-class outcome carrying its typed reason, never a
//! bare error. That is why the courts below return [`CourtOutcome`] directly
//! instead of `Result`.

use std::fs;
use std::path::Path;

use serde_json::json;

use crate::diag::exit_codes;
use crate::ecosystem::{certify_ecosystem, EcosystemObservation, EcosystemReceipt};
use crate::errc::{certify_errc, ErrcObservation, ErrcReceipt};
use crate::errc_claim_assurance::{
    certify_errc_claim_assurance, ErrcClaimAssuranceReceipt, ErrcClaimWitness,
};
use crate::standing::{certify_standing, StandingObservation, StandingReceipt};
use crate::types::{AdmittedReceipt, Receipt};

use wasm4pm_compat::authority::{AuthorityConstraint, AuthorityEnvelope, Capability};
use wasm4pm_compat::receipt::Digest;
use wasm4pm_compat::witness::RustTypestateLaw;

/// Capability name the CLI declares when it drives standing certification.
///
/// The CLI does not inherit the subject's authority; it exercises its own
/// bounded certification capability, which wasm4pm-compat validates
/// structurally before [`crate::standing::certify_standing`] will project it.
pub const STANDING_CAPABILITY: &str = "affidavit.certify-standing";

/// Digest pin for [`STANDING_CAPABILITY`]. Carried, never computed.
pub const STANDING_CAPABILITY_DIGEST: &str = "blake3:affidavit-standing-v2";

/// The outcome of one federation court.
///
/// `code` is a stable [`crate::diag::exit_codes`] value; `report` is the
/// machine-readable record of what the court did, suitable for `--format json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourtOutcome {
    /// Stable exit code from [`crate::diag::exit_codes`].
    pub code: i32,
    /// Machine-readable court report.
    pub report: serde_json::Value,
}

impl CourtOutcome {
    /// Whether this court accepted (exit code 0).
    pub fn accepted(&self) -> bool {
        self.code == exit_codes::OK
    }

    /// The court's stated reason, for human-facing diagnostics.
    pub fn reason(&self) -> &str {
        self.report
            .get("reason")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
    }
}

/// Build an accepting report carrying the sealed receipt.
fn accepted(court: &str, profile: &str, receipt: serde_json::Value) -> CourtOutcome {
    CourtOutcome {
        code: exit_codes::OK,
        report: json!({
            "court": court,
            "profile": profile,
            "accepted": true,
            "reason": "certified",
            "receipt": receipt,
        }),
    }
}

/// Build a refusal report. `code` distinguishes a refusal (REJECT) from an
/// operator input problem (USAGE_ERROR / IO_ERROR).
fn refused(court: &str, profile: &str, code: i32, reason: String) -> CourtOutcome {
    CourtOutcome {
        code,
        report: json!({
            "court": court,
            "profile": profile,
            "accepted": false,
            "reason": reason,
            "receipt": serde_json::Value::Null,
        }),
    }
}

/// Read a UTF-8 file, or describe the I/O failure by path.
fn read_file(path: &str) -> Result<String, String> {
    fs::read_to_string(Path::new(path)).map_err(|err| format!("cannot read {path}: {err}"))
}

/// Load a `core/v1` receipt and drive it through the real Layer 2 admission
/// gate. Both courts inside [`crate::admission::admit`] run: the
/// wasm4pm-compat OCEL structural law and the affidavit certify pipeline.
fn admit_source(path: &str) -> Result<AdmittedReceipt, (i32, String)> {
    let bytes = read_file(path).map_err(|err| (exit_codes::IO_ERROR, err))?;
    let receipt: Receipt = serde_json::from_str(&bytes).map_err(|err| {
        (
            exit_codes::REJECT,
            format!("source receipt decode failed: {err}"),
        )
    })?;
    crate::admission::admit(receipt)
        .map_err(|refusal| (exit_codes::REJECT, format!("admission refused: {refusal}")))
}

/// Parse an operator-supplied observation. A malformed observation is a usage
/// error: it never claimed to be a witness, so refusing it is not a verdict.
fn load_observation<T: serde::de::DeserializeOwned>(
    path: &str,
    what: &str,
) -> Result<T, (i32, String)> {
    let bytes = read_file(path).map_err(|err| (exit_codes::IO_ERROR, err))?;
    serde_json::from_str(&bytes).map_err(|err| {
        (
            exit_codes::USAGE_ERROR,
            format!("{what} is not well-formed: {err}"),
        )
    })
}

/// Load a sealed profile receipt. Deserialization re-runs the profile's own
/// `verify()`, so a failure here is a genuine REJECT, not a parse complaint.
fn load_sealed<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, (i32, String)> {
    let bytes = read_file(path).map_err(|err| (exit_codes::IO_ERROR, err))?;
    serde_json::from_str(&bytes)
        .map_err(|err| (exit_codes::REJECT, format!("sealed receipt refused: {err}")))
}

/// Serialize a sealed receipt for the court report.
fn seal_to_json<T: serde::Serialize>(receipt: &T) -> Result<serde_json::Value, (i32, String)> {
    serde_json::to_value(receipt).map_err(|err| {
        (
            exit_codes::INTERNAL,
            format!("sealed receipt could not be serialized: {err}"),
        )
    })
}

/// The bounded authority envelope the CLI declares for standing certification.
///
/// `scope` is operator-supplied and must be non-empty: the envelope declares
/// [`AuthorityConstraint::RequiresBoundedScope`], so wasm4pm-compat refuses an
/// unbounded scope before any receipt is sealed.
pub fn cli_standing_authority(scope: &str) -> AuthorityEnvelope<RustTypestateLaw> {
    let capability = Capability::<RustTypestateLaw>::new(
        STANDING_CAPABILITY,
        Digest::new(STANDING_CAPABILITY_DIGEST),
    );
    AuthorityEnvelope::new(
        capability,
        vec![
            AuthorityConstraint::RequiresWitness,
            AuthorityConstraint::RequiresDigestPin,
            AuthorityConstraint::RequiresBoundedScope,
        ],
        scope,
    )
}

// ---------------------------------------------------------------------------
// standing — affidavit/standing/v2
// ---------------------------------------------------------------------------

/// `affi standing certify` — seal a standing claim over an admitted receipt.
///
/// ALIVE is not grantable here: [`crate::standing::certify_standing`] refuses an
/// ALIVE observation that lacks execution, verification, and replay evidence.
/// This court cannot widen that law; it only reaches it.
pub fn standing_certify(receipt: &str, observation: &str, scope: &str) -> CourtOutcome {
    let court = "standing/certify";
    let profile = crate::standing::STANDING_PROFILE;

    let admitted = match admit_source(receipt) {
        Ok(admitted) => admitted,
        Err((code, reason)) => return refused(court, profile, code, reason),
    };
    let observation: StandingObservation =
        match load_observation(observation, "standing observation") {
            Ok(observation) => observation,
            Err((code, reason)) => return refused(court, profile, code, reason),
        };

    let authority = cli_standing_authority(scope);
    match certify_standing(&admitted, &authority, observation) {
        Ok(sealed) => match seal_to_json(&sealed) {
            Ok(value) => accepted(court, profile, value),
            Err((code, reason)) => refused(court, profile, code, reason),
        },
        Err(refusal) => refused(
            court,
            profile,
            exit_codes::REJECT,
            format!("standing refused: {refusal}"),
        ),
    }
}

/// `affi standing verify` — re-run the standing law over a sealed receipt.
pub fn standing_verify(receipt: &str) -> CourtOutcome {
    let court = "standing/verify";
    let profile = crate::standing::STANDING_PROFILE;

    match load_sealed::<StandingReceipt>(receipt) {
        Ok(sealed) => match seal_to_json(&sealed) {
            Ok(value) => accepted(court, profile, value),
            Err((code, reason)) => refused(court, profile, code, reason),
        },
        Err((code, reason)) => refused(court, profile, code, reason),
    }
}

// ---------------------------------------------------------------------------
// ecosystem — affidavit/ecosystem/v1
// ---------------------------------------------------------------------------

/// `affi ecosystem certify` — federate exact, already-sealed member standing
/// receipts into one quorum-scored ecosystem receipt.
pub fn ecosystem_certify(receipt: &str, observation: &str) -> CourtOutcome {
    let court = "ecosystem/certify";
    let profile = crate::ecosystem::ECOSYSTEM_PROFILE;

    let admitted = match admit_source(receipt) {
        Ok(admitted) => admitted,
        Err((code, reason)) => return refused(court, profile, code, reason),
    };
    // Each member's sealed standing receipt is re-verified during this parse:
    // `EcosystemMember` embeds a `StandingReceipt`, whose `Deserialize` runs the
    // standing law. A federation cannot be assembled from unverifiable members.
    let observation: EcosystemObservation =
        match load_observation(observation, "ecosystem observation") {
            Ok(observation) => observation,
            Err((code, reason)) => return refused(court, profile, code, reason),
        };

    match certify_ecosystem(&admitted, observation) {
        Ok(sealed) => match seal_to_json(&sealed) {
            Ok(value) => accepted(court, profile, value),
            Err((code, reason)) => refused(court, profile, code, reason),
        },
        Err(refusal) => refused(
            court,
            profile,
            exit_codes::REJECT,
            format!("ecosystem refused: {refusal}"),
        ),
    }
}

/// `affi ecosystem verify` — re-run the federation law over a sealed receipt.
pub fn ecosystem_verify(receipt: &str) -> CourtOutcome {
    let court = "ecosystem/verify";
    let profile = crate::ecosystem::ECOSYSTEM_PROFILE;

    match load_sealed::<EcosystemReceipt>(receipt) {
        Ok(sealed) => match seal_to_json(&sealed) {
            Ok(value) => accepted(court, profile, value),
            Err((code, reason)) => refused(court, profile, code, reason),
        },
        Err((code, reason)) => refused(court, profile, code, reason),
    }
}

// ---------------------------------------------------------------------------
// errc — affidavit/errc/v1
// ---------------------------------------------------------------------------

/// `affi errc certify` — seal a declared ERRC transformation.
///
/// The directional law (ELIMINATE / REDUCE / RAISE / CREATE), the mandatory
/// non-empty preservation fence, and the no-cross-unit-summation rule are all
/// enforced by [`crate::errc::certify_errc`]; this court only reaches them.
pub fn errc_certify(receipt: &str, observation: &str) -> CourtOutcome {
    let court = "errc/certify";
    let profile = crate::errc::ERRC_PROFILE;

    let admitted = match admit_source(receipt) {
        Ok(admitted) => admitted,
        Err((code, reason)) => return refused(court, profile, code, reason),
    };
    let observation: ErrcObservation = match load_observation(observation, "errc observation") {
        Ok(observation) => observation,
        Err((code, reason)) => return refused(court, profile, code, reason),
    };

    match certify_errc(&admitted, observation) {
        Ok(sealed) => match seal_to_json(&sealed) {
            Ok(value) => accepted(court, profile, value),
            Err((code, reason)) => refused(court, profile, code, reason),
        },
        Err(refusal) => refused(
            court,
            profile,
            exit_codes::REJECT,
            format!("errc refused: {refusal}"),
        ),
    }
}

/// `affi errc verify` — re-run the ERRC law over a sealed receipt.
pub fn errc_verify(receipt: &str) -> CourtOutcome {
    let court = "errc/verify";
    let profile = crate::errc::ERRC_PROFILE;

    match load_sealed::<ErrcReceipt>(receipt) {
        Ok(sealed) => match seal_to_json(&sealed) {
            Ok(value) => accepted(court, profile, value),
            Err((code, reason)) => refused(court, profile, code, reason),
        },
        Err((code, reason)) => refused(court, profile, code, reason),
    }
}

// ---------------------------------------------------------------------------
// errc claim assurance — affidavit/errc-claim-assurance/v1
// ---------------------------------------------------------------------------

/// `affi errc assure` — seal a one-witness-per-claim assurance ledger over a
/// sealed ERRC receipt.
///
/// The bijection law is total: every claim in the parent needs exactly one
/// witness, and a witness naming an unknown claim is refused.
pub fn errc_assure(parent: &str, witnesses: &str) -> CourtOutcome {
    let court = "errc/assure";
    let profile = crate::errc_claim_assurance::ERRC_CLAIM_ASSURANCE_PROFILE;

    let parent: ErrcReceipt = match load_sealed(parent) {
        Ok(parent) => parent,
        Err((code, reason)) => return refused(court, profile, code, reason),
    };
    let witnesses: Vec<ErrcClaimWitness> = match load_observation(witnesses, "claim witness ledger")
    {
        Ok(witnesses) => witnesses,
        Err((code, reason)) => return refused(court, profile, code, reason),
    };

    match certify_errc_claim_assurance(&parent, witnesses) {
        Ok(sealed) => match seal_to_json(&sealed) {
            Ok(value) => accepted(court, profile, value),
            Err((code, reason)) => refused(court, profile, code, reason),
        },
        Err(refusal) => refused(
            court,
            profile,
            exit_codes::REJECT,
            format!("claim assurance refused: {refusal}"),
        ),
    }
}

/// `affi errc verify-assurance` — re-run the assurance law over a sealed
/// assurance receipt, optionally binding it to its exact parent.
///
/// Without `parent`, this proves canonical structure and content identity.
/// With `parent`, it additionally proves parent-hash identity and exact
/// claim-set correspondence via
/// [`ErrcClaimAssuranceReceipt::verify_against`].
pub fn errc_verify_assurance(receipt: &str, parent: Option<&str>) -> CourtOutcome {
    let court = "errc/verify-assurance";
    let profile = crate::errc_claim_assurance::ERRC_CLAIM_ASSURANCE_PROFILE;

    let sealed: ErrcClaimAssuranceReceipt = match load_sealed(receipt) {
        Ok(sealed) => sealed,
        Err((code, reason)) => return refused(court, profile, code, reason),
    };

    if let Some(parent) = parent {
        let parent: ErrcReceipt = match load_sealed(parent) {
            Ok(parent) => parent,
            Err((code, reason)) => return refused(court, profile, code, reason),
        };
        if let Err(refusal) = sealed.verify_against(&parent) {
            return refused(
                court,
                profile,
                exit_codes::REJECT,
                format!("claim assurance refused: {refusal}"),
            );
        }
    }

    match seal_to_json(&sealed) {
        Ok(value) => accepted(court, profile, value),
        Err((code, reason)) => refused(court, profile, code, reason),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::ChainAssembler;
    use crate::ocel::{build_event, object_ref, SeqCounter};
    use crate::standing::{ExecutionEvidence, ReplayEvidence, Standing, SubjectIdentity};
    use crate::types::Blake3Hash;

    /// Write `contents` to `dir/name` and return the path as a String.
    fn write(dir: &std::path::Path, name: &str, contents: &str) -> String {
        let path = dir.join(name);
        std::fs::write(&path, contents).expect("write fixture");
        path.to_string_lossy().into_owned()
    }

    /// A minimal admissible `core/v1` receipt on disk.
    fn source_receipt(dir: &std::path::Path) -> String {
        let mut assembler = ChainAssembler::new();
        let mut counter = SeqCounter::new();
        let event = build_event(
            "build",
            vec![object_ref("repo", "git")],
            b"exact-head execution",
            &mut counter,
        )
        .expect("event is well-formed");
        assembler.append(event).expect("event admitted");
        let receipt = assembler.finalize();
        let json = serde_json::to_string(&receipt).expect("receipt serializes");
        write(dir, "source.json", &json)
    }

    fn subject() -> SubjectIdentity {
        SubjectIdentity {
            subject: "seanchatmangpt/affidavit".to_string(),
            base: "a".repeat(40),
            tree: "b".repeat(40),
            candidate: "c".repeat(40),
        }
    }

    fn alive_observation() -> StandingObservation {
        StandingObservation {
            subject: subject(),
            observation_commitment: Blake3Hash::from_bytes(b"o-star"),
            standing: Standing::Alive,
            execution: Some(ExecutionEvidence {
                command: "cargo test --all-targets".to_string(),
                exit_code: 0,
                result_commitment: Blake3Hash::from_bytes(b"execution"),
            }),
            verification: Some(crate::standing::VerificationEvidence {
                command: "cargo clippy --all-targets -- -D warnings".to_string(),
                exit_code: 0,
                report_commitment: Blake3Hash::from_bytes(b"verification"),
            }),
            replay: Some(ReplayEvidence {
                command: "just validate".to_string(),
                environment_commitment: Blake3Hash::from_bytes(b"capsule"),
            }),
            previous_receipt: None,
        }
    }

    #[test]
    fn standing_certify_seals_an_alive_claim_and_verify_reaccepts_it() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = source_receipt(dir.path());
        let observation = write(
            dir.path(),
            "observation.json",
            &serde_json::to_string(&alive_observation()).expect("observation serializes"),
        );

        let certified = standing_certify(&source, &observation, "repo:seanchatmangpt/affidavit");
        assert!(
            certified.accepted(),
            "certify should accept: {}",
            certified.reason()
        );

        // Round-trip the sealed receipt through disk. `verify` only accepts it
        // because the hand-written Deserialize re-ran the standing law.
        let sealed = write(
            dir.path(),
            "standing.json",
            &serde_json::to_string(&certified.report["receipt"]).expect("sealed serializes"),
        );
        let verified = standing_verify(&sealed);
        assert!(
            verified.accepted(),
            "verify should accept: {}",
            verified.reason()
        );
        assert_eq!(
            verified.report["profile"],
            crate::standing::STANDING_PROFILE
        );
    }

    #[test]
    fn standing_certify_refuses_alive_without_replay_evidence() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = source_receipt(dir.path());
        let mut observation = alive_observation();
        observation.replay = None;
        let observation_path = write(
            dir.path(),
            "observation.json",
            &serde_json::to_string(&observation).expect("observation serializes"),
        );

        let outcome = standing_certify(&source, &observation_path, "repo:affidavit");
        assert_eq!(outcome.code, exit_codes::REJECT);
        assert!(
            outcome.reason().contains("standing refused"),
            "expected a named standing refusal, got: {}",
            outcome.reason()
        );
    }

    #[test]
    fn standing_certify_refuses_an_unbounded_authority_scope() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = source_receipt(dir.path());
        let observation = write(
            dir.path(),
            "observation.json",
            &serde_json::to_string(&alive_observation()).expect("observation serializes"),
        );

        // An empty scope violates RequiresBoundedScope. wasm4pm-compat refuses
        // the envelope before any receipt is sealed.
        let outcome = standing_certify(&source, &observation, "   ");
        assert_eq!(outcome.code, exit_codes::REJECT);
        assert!(
            outcome.reason().contains("authority"),
            "expected an authority-envelope refusal, got: {}",
            outcome.reason()
        );
    }

    #[test]
    fn a_tampered_sealed_standing_receipt_cannot_be_verified() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = source_receipt(dir.path());
        let observation = write(
            dir.path(),
            "observation.json",
            &serde_json::to_string(&alive_observation()).expect("observation serializes"),
        );
        let certified = standing_certify(&source, &observation, "repo:affidavit");
        assert!(certified.accepted());

        // Flip one field the receipt hash covers. The stored hash no longer
        // matches the material, so the receipt cannot be deserialized at all.
        let mut tampered = certified.report["receipt"].clone();
        tampered["subject"]["candidate"] = json!("d".repeat(40));
        let path = write(
            dir.path(),
            "tampered.json",
            &serde_json::to_string(&tampered).expect("tampered serializes"),
        );

        let outcome = standing_verify(&path);
        assert_eq!(outcome.code, exit_codes::REJECT);
        assert!(
            outcome.reason().contains("sealed receipt refused"),
            "expected a sealed-receipt refusal, got: {}",
            outcome.reason()
        );
    }

    #[test]
    fn a_source_receipt_that_fails_admission_is_refused_not_certified() {
        use crate::types::OperationEvent;

        let dir = tempfile::tempdir().expect("tempdir");
        // A chain-consistent but OBJECTLESS receipt: every affidavit verifier
        // stage passes, so only the wasm4pm-compat OCEL court catches it. If
        // this court skipped `admission::admit` and called the certifier
        // directly, the receipt below would be certified.
        let objectless = OperationEvent {
            id: "evt-0".to_string(),
            seq: 0,
            event_type: "build".to_string(),
            objects: vec![],
            payload_commitment: Blake3Hash::from_bytes(b"content"),
        };
        let chain_hash = crate::chain::recompute_chain(std::slice::from_ref(&objectless))
            .expect("recompute chain");
        let receipt = crate::types::Receipt::sealed(
            crate::chain::FORMAT_VERSION.to_string(),
            vec![objectless],
            chain_hash,
        );
        assert!(
            crate::verifier::verify(&receipt).accepted,
            "the verifier alone accepts it — the OCEL court is what refuses it"
        );
        let source = write(
            dir.path(),
            "objectless.json",
            &serde_json::to_string(&receipt).expect("receipt serializes"),
        );
        let observation = write(
            dir.path(),
            "observation.json",
            &serde_json::to_string(&alive_observation()).expect("observation serializes"),
        );

        let outcome = standing_certify(&source, &observation, "repo:affidavit");
        assert_eq!(outcome.code, exit_codes::REJECT);
        assert!(
            outcome.reason().contains("admission refused"),
            "expected an admission refusal, got: {}",
            outcome.reason()
        );
    }

    #[test]
    fn a_malformed_observation_is_a_usage_error_not_a_refusal() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = source_receipt(dir.path());
        let observation = write(dir.path(), "observation.json", "{ not json }");

        let outcome = standing_certify(&source, &observation, "repo:affidavit");
        assert_eq!(
            outcome.code,
            exit_codes::USAGE_ERROR,
            "a malformed operator input is not a verdict on a witness"
        );
    }

    #[test]
    fn a_missing_path_is_an_io_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let observation = write(
            dir.path(),
            "observation.json",
            &serde_json::to_string(&alive_observation()).expect("observation serializes"),
        );
        let outcome = standing_certify("/nonexistent/receipt.json", &observation, "repo:affidavit");
        assert_eq!(outcome.code, exit_codes::IO_ERROR);
    }

    #[test]
    fn errc_certify_seals_a_directional_claim_and_assurance_binds_to_it() {
        use crate::errc::{ErrcClaim, ErrcMeasure, ErrcQuadrant, PreservedInvariant};

        let dir = tempfile::tempdir().expect("tempdir");
        let source = source_receipt(dir.path());

        let observation = ErrcObservation {
            subject: subject(),
            observation_commitment: Blake3Hash::from_bytes(b"errc-o-star"),
            claims: vec![ErrcClaim {
                id: "eliminate-unreachable-kernel".to_string(),
                target: "affi CLI surface over the federation kernel".to_string(),
                quadrant: ErrcQuadrant::Eliminate,
                measure: ErrcMeasure {
                    metric: "kernel_certifiers_without_cli_surface".to_string(),
                    unit: "certifiers".to_string(),
                    baseline: 4,
                    candidate: 0,
                },
                evidence_commitment: Blake3Hash::from_bytes(b"errc-evidence"),
            }],
            preserved_invariants: vec![PreservedInvariant {
                id: "certify-not-decide".to_string(),
                statement: "The CLI reaches the kernel's laws; it never adds one.".to_string(),
                evidence_commitment: Blake3Hash::from_bytes(b"fence-evidence"),
            }],
            replay: ReplayEvidence {
                command: "cargo test --all-targets".to_string(),
                environment_commitment: Blake3Hash::from_bytes(b"capsule"),
            },
            previous_receipt: None,
        };
        let observation_path = write(
            dir.path(),
            "errc-observation.json",
            &serde_json::to_string(&observation).expect("observation serializes"),
        );

        let certified = errc_certify(&source, &observation_path);
        assert!(
            certified.accepted(),
            "errc certify should accept: {}",
            certified.reason()
        );

        let parent = write(
            dir.path(),
            "errc.json",
            &serde_json::to_string(&certified.report["receipt"]).expect("sealed serializes"),
        );
        assert!(errc_verify(&parent).accepted());

        // One witness per claim satisfies the bijection.
        let witnesses = vec![ErrcClaimWitness {
            claim_id: "eliminate-unreachable-kernel".to_string(),
            verifier: "cargo test --test federation_cli".to_string(),
            evidence_locator: "tests/federation_cli.rs".to_string(),
            observed_result: "8 federation verbs reachable from affi".to_string(),
            evidence_commitment: Blake3Hash::from_bytes(b"witness-evidence"),
            exclusions: vec!["Does not establish operational utility.".to_string()],
        }];
        let witnesses_path = write(
            dir.path(),
            "witnesses.json",
            &serde_json::to_string(&witnesses).expect("witnesses serialize"),
        );

        let assured = errc_assure(&parent, &witnesses_path);
        assert!(
            assured.accepted(),
            "assurance should accept: {}",
            assured.reason()
        );

        let assurance = write(
            dir.path(),
            "assurance.json",
            &serde_json::to_string(&assured.report["receipt"]).expect("sealed serializes"),
        );
        assert!(errc_verify_assurance(&assurance, Some(&parent)).accepted());
    }

    #[test]
    fn errc_assure_refuses_an_incomplete_witness_ledger() {
        use crate::errc::{ErrcClaim, ErrcMeasure, ErrcQuadrant, PreservedInvariant};

        let dir = tempfile::tempdir().expect("tempdir");
        let source = source_receipt(dir.path());
        // Distinct (target, metric, unit) coordinates: ERRC refuses two claims
        // that share one factor coordinate, so the ledger below must not.
        let claim = |id: &str, target: &str| ErrcClaim {
            id: id.to_string(),
            target: target.to_string(),
            quadrant: ErrcQuadrant::Reduce,
            measure: ErrcMeasure {
                metric: "unreachable_certifiers".to_string(),
                unit: "certifiers".to_string(),
                baseline: 4,
                candidate: 2,
            },
            evidence_commitment: Blake3Hash::from_bytes(id.as_bytes()),
        };
        let observation = ErrcObservation {
            subject: subject(),
            observation_commitment: Blake3Hash::from_bytes(b"errc-o-star"),
            claims: vec![
                claim("claim-a", "standing surface"),
                claim("claim-b", "ecosystem surface"),
            ],
            preserved_invariants: vec![PreservedInvariant {
                id: "fence".to_string(),
                statement: "Preserved.".to_string(),
                evidence_commitment: Blake3Hash::from_bytes(b"fence"),
            }],
            replay: ReplayEvidence {
                command: "cargo test".to_string(),
                environment_commitment: Blake3Hash::from_bytes(b"capsule"),
            },
            previous_receipt: None,
        };
        let observation_path = write(
            dir.path(),
            "errc-observation.json",
            &serde_json::to_string(&observation).expect("observation serializes"),
        );
        let certified = errc_certify(&source, &observation_path);
        assert!(certified.accepted(), "{}", certified.reason());
        let parent = write(
            dir.path(),
            "errc.json",
            &serde_json::to_string(&certified.report["receipt"]).expect("sealed serializes"),
        );

        // Two claims, one witness: the bijection cannot hold.
        let witnesses = vec![ErrcClaimWitness {
            claim_id: "claim-a".to_string(),
            verifier: "cargo test".to_string(),
            evidence_locator: "tests/".to_string(),
            observed_result: "ok".to_string(),
            evidence_commitment: Blake3Hash::from_bytes(b"w"),
            exclusions: vec!["Not a truth claim.".to_string()],
        }];
        let witnesses_path = write(
            dir.path(),
            "witnesses.json",
            &serde_json::to_string(&witnesses).expect("witnesses serialize"),
        );

        let outcome = errc_assure(&parent, &witnesses_path);
        assert_eq!(outcome.code, exit_codes::REJECT);
        assert!(
            outcome.reason().contains("claim assurance refused"),
            "expected a named assurance refusal, got: {}",
            outcome.reason()
        );
    }

    #[test]
    fn ecosystem_certify_federates_sealed_member_standing_receipts() {
        use crate::ecosystem::{EcosystemMember, EcosystemRole, RoleRequirement};

        let dir = tempfile::tempdir().expect("tempdir");
        let source = source_receipt(dir.path());
        let observation_path = write(
            dir.path(),
            "observation.json",
            &serde_json::to_string(&alive_observation()).expect("observation serializes"),
        );
        let member = standing_certify(&source, &observation_path, "repo:affidavit");
        assert!(member.accepted(), "{}", member.reason());
        let member: StandingReceipt =
            serde_json::from_value(member.report["receipt"].clone()).expect("member deserializes");

        let federation = EcosystemObservation {
            subject: subject(),
            observation_commitment: Blake3Hash::from_bytes(b"federation-o-star"),
            requirements: vec![RoleRequirement {
                role: EcosystemRole::RuntimeExecution,
                minimum_alive: 1,
            }],
            members: vec![EcosystemMember {
                role: EcosystemRole::RuntimeExecution,
                standing_receipt: member,
            }],
            previous_receipt: None,
        };
        let federation_path = write(
            dir.path(),
            "federation.json",
            &serde_json::to_string(&federation).expect("federation serializes"),
        );

        let certified = ecosystem_certify(&source, &federation_path);
        assert!(
            certified.accepted(),
            "ecosystem certify should accept: {}",
            certified.reason()
        );

        let sealed = write(
            dir.path(),
            "ecosystem.json",
            &serde_json::to_string(&certified.report["receipt"]).expect("sealed serializes"),
        );
        assert!(ecosystem_verify(&sealed).accepted());
    }
}
