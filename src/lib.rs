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
//! ```rust
//! use affidavit::{Receipt, Verdict, ProfileId};
//! use affidavit::chain::ChainAssembler;
//!
//! // Assemble a receipt (simplified example)
//! // let mut assembler = ChainAssembler::new();
//! // ... append events ...
//! // let receipt = assembler.finalize();
//!
//! // Verify a receipt
//! // let verdict = affidavit::verifier::verify(&receipt, ProfileId::CoreV1);
//! // assert!(verdict.accepted);
//! ```
//!
//! ## Feature Flags
//!
//! *   `default`: Includes the core library and standard profiles.
//! *   `gpu`: Enables the high-performance GPU-accelerated verifier.
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

pub mod admission;
pub mod bench;
pub mod catalog;
pub mod chain;
pub mod cli;

#[cfg(feature = "discovery")]
pub mod discovery;

pub mod error;
pub mod fixture_db;
pub mod handlers;

#[cfg(feature = "lsp")]
pub mod lsp;

pub mod ocel;
pub mod quality;
pub mod quality_extended;
pub mod quality_correlation;
pub mod quality_ocel;
pub mod quality_object_level;

#[cfg(feature = "predictive")]
pub mod predict_maximalist;

pub mod tracing;
pub mod types;
pub mod verbs;
pub mod verifier;

pub mod diff;
pub mod visualize;

#[cfg(feature = "mutation")]
pub mod mutate;

pub mod model_mining;

#[cfg(feature = "gpu")]
#[path = "1000x_gpu_verifier.rs"]
pub mod gpu_verifier;

#[cfg(feature = "remediation")]
#[path = "1000x_auto_remediate_dx.rs"]
pub mod auto_remediate;

#[cfg(feature = "pqc")]
#[path = "1000x_post_quantum_sealing.rs"]
pub mod pqc_sealing;

/// Main entry point for the `affi` CLI application.
///
/// # Errors
///
/// Returns an error if command-line arguments are invalid or if the
/// requested operation fails.
pub fn run() -> clap_noun_verb::Result<()> {
    clap_noun_verb::run()
}

pub use error::AffidavitError;
pub use types::{
    canonical_bytes, Blake3Hash, CheckOutcome, ObjectRef, OperationEvent, ProfileId, Receipt,
    Verdict,
};
