//! # affidavit — The Provenance Layer
//!
//! `affidavit` is a high-assurance provenance engine that assembles and certifies
//! **provenance receipts**: append-only, content-addressed chains of operation-events.
//!
//! ## Philosophy: Certify, Don't Decide
//!
//! The core doctrine of `affidavit` is that the verifier never decides whether
//! a process is "honest" or "correct" — those questions are often undecidable.
//! Instead, it **checks a witness** (the receipt) against a fixed format standard.
//! Every check in the pipeline is decidable, providing a mathematical guarantee
//! of structural and cryptographic integrity.

#![deny(clippy::print_stdout)]
#![deny(unsafe_code)]

#[macro_use]
mod macros;

pub mod admission;
pub mod bench;
pub mod catalog;
pub mod chain;
pub mod cli;
#[cfg(feature = "discovery")]
pub mod discovery;
pub mod ecosystem;
pub mod errc;
pub mod errc_claim_assurance;
pub mod error;
pub mod fixture_db;
pub mod handlers;
#[cfg(feature = "lsp")]
pub mod lsp;
pub mod ocel;
pub mod portable_protocol;
pub mod quality;
pub mod quality_correlation;
pub mod quality_extended;
pub mod quality_object_level;
pub mod quality_ocel;
pub mod sbom;
pub mod sbom_artifacts;
pub mod sbom_compliance;
pub mod sbom_ocel;
pub mod sbom_supply_chain;
pub mod sbom_vulnerability;
#[cfg(feature = "predictive")]
pub mod predict_maximalist;
pub mod tracing;
pub mod types;
pub mod verbs;
pub mod verifier;
pub mod diag;
pub mod doctor_check;
pub mod output;
pub mod standing;
pub mod diff;
pub mod visualize;
#[cfg(feature = "mutation")]
pub mod mutate;
pub mod model_mining;
pub mod registry;
#[cfg(feature = "gpu")]
#[path = "1000x_gpu_verifier.rs"]
pub mod gpu_verifier;
#[cfg(feature = "remediation")]
#[path = "1000x_auto_remediate_dx.rs"]
pub mod auto_remediate;
#[cfg(feature = "pqc")]
#[path = "1000x_post_quantum_sealing.rs"]
pub mod pqc_sealing;

pub fn run() -> clap_noun_verb::Result<()> { clap_noun_verb::run() }

pub use ecosystem::{certify_ecosystem, EcosystemMember, EcosystemObservation, EcosystemReceipt, EcosystemRefusal,EcosystemRole, RoleCoverage, RoleRequirement, ECOSYSTEM_AUTHORITY_CEILING,ECOSYSTEM_CLAIM_CEILING, ECOSYSTEM_PROFILE};
pub use errc::{certify_errc, ErrcClaim, ErrcMeasure, ErrcObservation, ErrcQuadrant, ErrcReceipt, ErrcRefusal,ErrcSource, PreservedInvariant, QuadrantCounts, ERRC_CLAIM_CEILING, ERRC_PROFILE,ERRC_SOURCE_ARTIFACT, ERRC_SOURCE_COMMIT, ERRC_SOURCE_REPOSITORY};
pub use errc_claim_assurance::{certify_errc_claim_assurance, ErrcClaimAssuranceReceipt, ErrcClaimAssuranceRefusal,ErrcClaimWitness, ERRC_CLAIM_ASSURANCE_CEILING, ERRC_CLAIM_ASSURANCE_PROFILE,ERRC_CLAIM_ASSURANCE_SOURCE_ARTIFACT};
pub use error::AffidavitError;
pub use standing::{certify_standing, AuthorityBinding, ExecutionEvidence, ReplayEvidence, Standing,StandingObservation, StandingReceipt, StandingRefusal, SubjectIdentity, VerificationEvidence,STANDING_PROFILE};
pub use types::{canonical_bytes, Blake3Hash, CheckOutcome, ObjectRef, OperationEvent, ProfileId, Receipt,Verdict};
