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
//!
//! ## Key Concepts
//!
//! *   **Receipt:** An immutable, append-only BLAKE3 chain of operation-events.
//! *   **Operation-Event:** A discrete record of a process action, including
//!     logical sequence numbers and commitments to payloads.
//! *   **Certify Pipeline:** A 7-stage decidable process that validates receipts
//!     from raw bytes to a final verdict.
//! *   **Unconstructable Bypass:** The library uses the "Seal" pattern to ensure
//!     that valid receipts can only be constructed through canonical, auditable seams.
//!
//! ## Quick Start
//!
//! ```no_run
//! use affidavit::chain::ChainAssembler;
//! use affidavit::ocel::{build_event, object_ref, SeqCounter};
//! use affidavit::verifier::verify;
//!
//! // Build a receipt by appending events.
//! let mut assembler = ChainAssembler::new();
//! let mut counter = SeqCounter::new();
//! let event = build_event(
//!     "build",
//!     vec![object_ref("repo", "git")],
//!     b"payload data",
//!     &mut counter,
//! )
//! .expect("event is well-formed");
//! assembler.append(event).expect("event admitted");
//!
//! // Finalize the receipt (produces an immutable, sealed chain).
//! let receipt = assembler.finalize();
//!
//! // Verify the receipt against the core standard.
//! let verdict = verify(&receipt);
//! assert!(verdict.accepted);
//! ```
//!
//! ## Integrated Verticals
//!
//! **Quality & Monitoring:** Real-time statistical process control (Western Electric rules) via [`quality`].
//!
//! **SBOM & Supply Chain:** Software Bill of Materials generation, compliance checking (NTIA), and
//! vulnerability aggregation via [`sbom`], [`sbom_compliance`], [`sbom_vulnerability`].
//!
//! **Object-Centric Event Logs:** OCEL integration and conversion via [`ocel`].
//!
//! ## Feature Flags
//!
//! *   `default`: Includes the core library and standard profiles.
//! *   `discovery`: Enables type schema discovery and introspection.
//! *   `lsp`: Exposes LSP diagnostics for receipt verification.
//! *   `predictive`: Enables predictive analysis and trend forecasting.
//!
//! # Errors
//!
//! Most operations return a [`crate::error::AffidavitError`] which encapsulates
//! various failure modes including I/O, serialization, and cryptographic mismatches.
//!
//! # Examples
//!
//! For a full end-to-end example of the provenance pipeline, see
//! `examples/full_pipeline.rs`.
//!
//! # Panics
//!
//! This crate is designed to be panic-free in production paths. Invariant checks
//! that could lead to panics are isolated to unreachable code branches or
//! explicit boundary checks.

#![deny(clippy::print_stdout)]
#![deny(unsafe_code)]

// Internal `outln!` / `out!` macros (route stdout through `output.rs`). Declared
// with `#[macro_use]` before the modules below so they are in scope crate-wide.
#[macro_use]
mod macros;

pub mod admission;
pub mod architecture;
pub mod bench;
pub mod brce;
pub mod catalog;
pub mod chain;
pub mod crypto_trust;
pub mod cli;

#[cfg(feature = "discovery")]
pub mod discovery;

pub mod dfcm;
pub mod ecosystem;
pub mod errc;
pub mod errc_claim_assurance;
pub mod error;
pub mod execution_manifest;
pub mod federation;
pub mod fixture_db;
/// GALL-007 evidence-only twelve-gate Chicago crown certification.
pub mod gall;
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

// Software Bill of Materials (SBOM) — supply-chain provenance layer.
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

// --- Cryptographic trust plane (affidavit-trust-plane-pack projections) ---
// Rendered by `ggen sync run` from
// ../ggen-marketplace/packs/affidavit-trust-plane-pack — never hand-edited.
#[cfg(feature = "crypto-trust")]
pub mod crypto_trust_canonical;
#[cfg(feature = "crypto-trust")]
pub mod crypto_trust_doctor;
#[cfg(feature = "secure-enclave")]
pub mod crypto_trust_enclave;
#[cfg(feature = "crypto-trust")]
pub mod crypto_trust_envelope;
#[cfg(feature = "crypto-trust")]
pub mod crypto_trust_es256;
#[cfg(feature = "crypto-trust")]
pub mod crypto_trust_journal;
#[cfg(feature = "crypto-trust")]
pub mod crypto_trust_kat;
#[cfg(feature = "crypto-trust")]
pub mod crypto_trust_keys;
#[cfg(feature = "crypto-trust")]
pub mod crypto_trust_lifecycle;
#[cfg(feature = "crypto-trust")]
pub mod crypto_trust_log;
#[cfg(feature = "crypto-trust")]
pub mod crypto_trust_pqc;
#[cfg(feature = "crypto-trust")]
pub mod crypto_trust_quorum;
#[cfg(feature = "crypto-trust")]
pub mod crypto_trust_revocation;
#[cfg(feature = "crypto-trust")]
pub mod crypto_trust_rotation;
#[cfg(feature = "crypto-trust")]
pub mod crypto_trust_sa2a;
#[cfg(feature = "crypto-trust")]
pub mod crypto_trust_seal;
#[cfg(feature = "crypto-trust")]
pub mod crypto_trust_store;
#[cfg(feature = "crypto-trust")]
pub mod crypto_trust_transparency;
#[cfg(feature = "crypto-trust")]
pub mod crypto_trust_verify;

pub mod model_mining;
pub mod registry;

#[cfg(feature = "gpu")]
#[path = "1000x_gpu_verifier.rs"]
pub mod gpu_verifier;

#[cfg(feature = "remediation")]
#[path = "1000x_auto_remediate_dx.rs"]
pub mod auto_remediate;

/// Main entry point for the `affi` CLI application.
///
/// # Errors
///
/// Returns an error if command-line arguments are invalid or if the
/// requested operation fails.
pub fn run() -> clap_noun_verb::Result<()> {
    clap_noun_verb::run()
}

pub use architecture::{
    ArchitectureQualificationReceipt, ArchitectureRefusal, ArchitectureStanding,
    ArchitectureStandingLedger, EvidenceSource, QualificationEvidence, Supersession,
    ARCHITECTURE_QUERY_SCHEMA, ARCHITECTURE_RECEIPT_SCHEMA,
};
pub use ecosystem::{
    certify_ecosystem, EcosystemMember, EcosystemObservation, EcosystemReceipt, EcosystemRefusal,
    EcosystemRole, RoleCoverage, RoleRequirement, ECOSYSTEM_AUTHORITY_CEILING,
    ECOSYSTEM_CLAIM_CEILING, ECOSYSTEM_PROFILE,
};
pub use errc::{
    certify_errc, ErrcClaim, ErrcMeasure, ErrcObservation, ErrcQuadrant, ErrcReceipt, ErrcRefusal,
    ErrcSource, PreservedInvariant, QuadrantCounts, ERRC_CLAIM_CEILING, ERRC_PROFILE,
    ERRC_SOURCE_ARTIFACT, ERRC_SOURCE_COMMIT, ERRC_SOURCE_REPOSITORY,
};
pub use errc_claim_assurance::{
    certify_errc_claim_assurance, ErrcClaimAssuranceReceipt, ErrcClaimAssuranceRefusal,
    ErrcClaimWitness, ERRC_CLAIM_ASSURANCE_CEILING, ERRC_CLAIM_ASSURANCE_PROFILE,
    ERRC_CLAIM_ASSURANCE_SOURCE_ARTIFACT,
};
pub use error::AffidavitError;
pub use crypto_trust::{
    sign_with_provider, verify_and_record, CryptoRefusal, KeyCustody, KeyRecord, KeyRegistry,
    KeyState, NonceLedger, SignatureAlgorithm, SignatureEnvelope, SigningMaterial, SigningProvider,
    VerificationContext, VerifiedSignature, JCS_SAFE_INTEGER_MAX, SIGNATURE_PROFILE,
    SIGNING_DOMAIN,
};
pub use federation::{CourtOutcome, STANDING_CAPABILITY, STANDING_CAPABILITY_DIGEST};
pub use standing::{
    certify_standing, AuthorityBinding, ExecutionEvidence, ReplayEvidence, Standing,
    StandingObservation, StandingReceipt, StandingRefusal, SubjectIdentity, VerificationEvidence,
    STANDING_PROFILE,
};
pub use types::{
    canonical_bytes, Blake3Hash, CheckOutcome, ObjectRef, OperationEvent, ProfileId, Receipt,
    Verdict,
};
