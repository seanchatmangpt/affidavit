//! AOD → standing federation adapter (AFFI-SPEC-26.11.19).
//!
//! Thin certifier: proves a sealed `.aod` is **Live-Gate-admissible** under a
//! bound [`CompiledOntology`] and records the ontology-hash witness. It does
//! **not** decide honesty of payload content, optimality, or authority to act.
//!
//! ## Standing ceiling
//!
//! The hard claim ceiling is [`Standing::PartialAlive`]. Format presence of a
//! Live-Gate-admitted artifact is necessary but not sufficient for
//! [`Standing::Alive`] (which still requires independent execution,
//! verification, and replay under [`crate::standing::certify_standing`]).
//! Requesting ALIVE through this adapter is a typed refusal:
//! `REFUSED_AOD_FEDERATION_ALIVE_CEILING`.
//!
//! ## Live Gate path
//!
//! Admission uses the real [`map_live`] verifier with an
//! [`InMemoryLeaseRegistry`]. That exercises the Live Gate law (envelope +
//! ontology + fixed-stride layout + monoatomic burn) **without** writing the
//! durable AFAL store — federation certify must not side-effect production
//! lease state. Durable burn remains `affi aod verify --live`.

use crate::aod_ontology::{compiled_fixture, FIXTURE_ONTOLOGY_HASH_HEX};
use crate::standing::{
    ExecutionEvidence, Standing, StandingObservation, SubjectIdentity, VerificationEvidence,
};
use crate::types::Blake3Hash;
use affidavit_core::{map_live, AodRefusal, CompiledOntology, InMemoryLeaseRegistry, AOD_MAGIC};

/// Adapter identity carried in court reports (not a sealed standing profile).
pub const AOD_FEDERATION_ADAPTER: &str = "affidavit/aod-federation-adapter/v1";

/// Hard standing ceiling for AOD format-presence federation.
pub const AOD_FEDERATION_STANDING_CEILING: Standing = Standing::PartialAlive;

/// Typed refusal from the AOD federation adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AodFederationRefusal {
    /// Path could not be read / mmap'd.
    Io(String),
    /// Bytes are not a sealed AOD envelope (magic / layout).
    NotAodArtifact(String),
    /// Live Gate (`map_live`) refused the artifact.
    LiveGate(AodRefusal),
    /// Ontology hash in the artifact / binding disagrees with the witness.
    OntologyHashMismatch {
        /// Expected ontology hash (lowercase hex).
        expected: String,
        /// Observed ontology hash (lowercase hex).
        got: String,
    },
    /// Caller asked for ALIVE; this adapter's ceiling is PARTIAL_ALIVE.
    AliveCeiling,
    /// Subject identity field empty.
    EmptySubjectField(&'static str),
}

impl core::fmt::Display for AodFederationRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Io(reason) => write!(f, "REFUSED_AOD_FEDERATION_IO: {reason}"),
            Self::NotAodArtifact(reason) => {
                write!(f, "REFUSED_AOD_FEDERATION_NOT_AOD: {reason}")
            }
            Self::LiveGate(e) => write!(
                f,
                "REFUSED_AOD_FEDERATION_LIVE_GATE [0x{:04X} {:?}]",
                e.machine_code(),
                e
            ),
            Self::OntologyHashMismatch { expected, got } => write!(
                f,
                "REFUSED_AOD_FEDERATION_ONTOLOGY_MISMATCH: expected={expected} got={got}"
            ),
            Self::AliveCeiling => f.write_str(
                "REFUSED_AOD_FEDERATION_ALIVE_CEILING: AOD format-presence certifies PARTIAL_ALIVE only; ALIVE requires independent execution+verification+replay outside this adapter",
            ),
            Self::EmptySubjectField(field) => {
                write!(f, "REFUSED_AOD_FEDERATION_EMPTY_FIELD: {field}")
            }
        }
    }
}

/// Facts certified by a successful Live Gate format-presence check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AodFormatWitness {
    /// BLAKE3 of the exact `.aod` bytes admitted.
    pub artifact_digest: Blake3Hash,
    /// Ontology hash that admitted the artifact (lowercase hex).
    pub ontology_hash_hex: String,
    /// Row count from the Live Gate view.
    pub row_count: u64,
    /// Buffer count from the Live Gate view.
    pub buffer_count: usize,
    /// Lease epoch burned in the in-memory registry (format court only).
    pub lease_epoch: u64,
    /// Adapter identity.
    pub adapter: &'static str,
    /// Standing ceiling sealed into the subsequent standing observation.
    pub standing_ceiling: Standing,
}

/// Run Live Gate format admission over `raw` under `compiled`.
///
/// Uses [`InMemoryLeaseRegistry`] — no durable AFAL write.
pub fn certify_aod_live_format(
    raw: &[u8],
    compiled: &CompiledOntology<'_>,
) -> Result<AodFormatWitness, AodFederationRefusal> {
    if raw.len() < 4 || raw[0..4] != AOD_MAGIC {
        return Err(AodFederationRefusal::NotAodArtifact(
            "missing AOD magic (0x9F1AC401)".into(),
        ));
    }

    let expected_hex = hex32(compiled.ontology_hash);
    let mut leases = InMemoryLeaseRegistry::new();
    let view = map_live(raw, compiled, &mut leases).map_err(AodFederationRefusal::LiveGate)?;

    let got_hex = hex32(view.ontology_hash);
    if got_hex != expected_hex {
        return Err(AodFederationRefusal::OntologyHashMismatch {
            expected: expected_hex,
            got: got_hex,
        });
    }

    Ok(AodFormatWitness {
        artifact_digest: Blake3Hash::from_bytes(raw),
        ontology_hash_hex: got_hex,
        row_count: view.row_count,
        buffer_count: view.buffer_count(),
        lease_epoch: view.lease_epoch,
        adapter: AOD_FEDERATION_ADAPTER,
        standing_ceiling: AOD_FEDERATION_STANDING_CEILING,
    })
}

/// Bind production fixture ontology (default for federation certify).
pub fn fixture_compiled() -> CompiledOntology<'static> {
    compiled_fixture()
}

/// Fixture ontology hash hex (witness pin).
pub fn fixture_ontology_hex() -> &'static str {
    FIXTURE_ONTOLOGY_HASH_HEX
}

/// Build a PARTIAL_ALIVE standing observation from an AOD format witness.
///
/// Carries Live Gate admit as execution evidence and ontology-hash match as
/// verification evidence. Replay is intentionally absent — that is why the
/// standing is capped at PARTIAL_ALIVE.
pub fn partial_alive_observation(
    subject: SubjectIdentity,
    witness: &AodFormatWitness,
) -> Result<StandingObservation, AodFederationRefusal> {
    require_subject(&subject)?;

    // Observation commitment binds adapter + digests (deterministic).
    let mut material = Vec::new();
    material.extend_from_slice(AOD_FEDERATION_ADAPTER.as_bytes());
    material.push(0);
    material.extend_from_slice(witness.artifact_digest.as_hex().as_bytes());
    material.push(0);
    material.extend_from_slice(witness.ontology_hash_hex.as_bytes());
    material.push(0);
    material.extend_from_slice(b"PARTIAL_ALIVE");
    let observation_commitment = Blake3Hash::from_bytes(&material);

    Ok(StandingObservation {
        subject,
        observation_commitment,
        standing: AOD_FEDERATION_STANDING_CEILING,
        execution: Some(ExecutionEvidence {
            command: "affidavit::federation_aod::live_gate_format_admit".into(),
            exit_code: 0,
            result_commitment: witness.artifact_digest.clone(),
        }),
        verification: Some(VerificationEvidence {
            command: "affidavit::federation_aod::ontology_hash_witness".into(),
            exit_code: 0,
            report_commitment: Blake3Hash::from_hex(witness.ontology_hash_hex.clone()),
        }),
        replay: None,
        previous_receipt: None,
    })
}

/// Refuse ALIVE requests by name (adapter ceiling).
pub fn refuse_alive_ceiling(standing: Standing) -> Result<(), AodFederationRefusal> {
    if standing == Standing::Alive {
        return Err(AodFederationRefusal::AliveCeiling);
    }
    Ok(())
}

fn require_subject(subject: &SubjectIdentity) -> Result<(), AodFederationRefusal> {
    for (name, value) in [
        ("subject.subject", subject.subject.as_str()),
        ("subject.base", subject.base.as_str()),
        ("subject.tree", subject.tree.as_str()),
        ("subject.candidate", subject.candidate.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(AodFederationRefusal::EmptySubjectField(name));
        }
    }
    Ok(())
}

fn hex32(bytes: &[u8; 32]) -> String {
    let mut out = String::with_capacity(64);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use affidavit_core::{seal_aod, AodBufferSpec, SealParams, FIXTURE_ONTOLOGY_HASH};

    fn sealed_fixture_aod() -> Vec<u8> {
        let page = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        seal_aod(
            &[AodBufferSpec { bytes: &page }],
            &SealParams {
                ontology_hash: &FIXTURE_ONTOLOGY_HASH,
                row_count: 2,
                lease_epoch: 1,
                lease_nullifier: &[0x11; 32],
                folded_receipt: &[0u8; 32],
            },
        )
        .expect("seal")
    }

    #[test]
    fn live_format_admits_fixture_sealed_aod() {
        let raw = sealed_fixture_aod();
        let compiled = fixture_compiled();
        let witness = certify_aod_live_format(&raw, &compiled).expect("admit");
        assert_eq!(witness.ontology_hash_hex, fixture_ontology_hex());
        assert_eq!(witness.standing_ceiling, Standing::PartialAlive);
        assert_eq!(witness.row_count, 2);
        assert_eq!(witness.buffer_count, 1);
        assert_eq!(witness.adapter, AOD_FEDERATION_ADAPTER);
    }

    #[test]
    fn live_format_refuses_wrong_ontology() {
        let raw = sealed_fixture_aod();
        let wrong = [0xAAu8; 32];
        let compiled = CompiledOntology {
            ontology_hash: &wrong,
            strides: &[8],
        };
        let err = certify_aod_live_format(&raw, &compiled).expect_err("mismatch");
        match err {
            AodFederationRefusal::LiveGate(AodRefusal::UnknownOntology)
            | AodFederationRefusal::OntologyHashMismatch { .. } => {}
            other => panic!("unexpected refusal: {other}"),
        }
    }

    #[test]
    fn live_format_refuses_non_aod_bytes() {
        let err = certify_aod_live_format(b"not-an-aod", &fixture_compiled()).unwrap_err();
        assert!(matches!(err, AodFederationRefusal::NotAodArtifact(_)));
    }

    #[test]
    fn alive_ceiling_is_typed_refusal() {
        assert!(refuse_alive_ceiling(Standing::PartialAlive).is_ok());
        let err = refuse_alive_ceiling(Standing::Alive).unwrap_err();
        assert_eq!(err, AodFederationRefusal::AliveCeiling);
        assert!(err
            .to_string()
            .contains("REFUSED_AOD_FEDERATION_ALIVE_CEILING"));
    }

    #[test]
    fn partial_alive_observation_has_no_replay() {
        let raw = sealed_fixture_aod();
        let witness = certify_aod_live_format(&raw, &fixture_compiled()).unwrap();
        let obs = partial_alive_observation(
            SubjectIdentity {
                subject: "seanchatmangpt/affidavit".into(),
                base: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                tree: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
                candidate: "cccccccccccccccccccccccccccccccccccccccc".into(),
            },
            &witness,
        )
        .unwrap();
        assert_eq!(obs.standing, Standing::PartialAlive);
        assert!(obs.execution.is_some());
        assert!(obs.verification.is_some());
        assert!(obs.replay.is_none());
    }
}
