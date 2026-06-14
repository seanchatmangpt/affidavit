//! affidavit — the Provenance Layer.
//!
//! Make the unverifiable unconstructable; certify rather than decide; relocate
//! the undecidable to one auditable place. The verifier never decides whether
//! code is honest — it checks a receipt (a witness) against a format standard,
//! and each check is decidable.
//!
//! # Cross-product example
//!
//! For the whole provenance pipeline composing every module through one receipt
//! (ocel -> chain -> verifier -> admission -> discovery -> lsp), see
//! `examples/full_pipeline.rs` (run: `cargo run --example full_pipeline`).

#![deny(clippy::print_stdout)]

pub mod admission;
pub mod chain;
pub mod cli;
pub mod discovery;
pub mod event_builder;
pub mod lsp;
pub mod ocel;
pub mod otel_sdk;
pub mod types;
pub mod verifier;
pub mod tracing;

pub use types::{
    canonical_bytes, Blake3Hash, CheckOutcome, ObjectRef, OperationEvent, ProfileId, Receipt,
    Verdict,
};
