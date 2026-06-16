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
pub mod bench;
pub mod catalog;
pub mod chain;
pub mod cli;
pub mod diff;
pub mod discovery;
pub mod error;
pub mod fixture_db;
pub mod handlers;
pub mod lsp;
pub mod model_mining;
pub mod mutate;
pub mod ocel;
pub mod predict_maximalist;
pub mod tracing;
pub mod types;
pub mod verifier;
pub mod verbs;
pub mod visualize;

#[cfg(test)]
#[path = "../wip/1000x_receipt_to_wasm_qol.rs"]
mod receipt_to_wasm_qol;

pub fn run() -> clap_noun_verb::Result<()> {
    clap_noun_verb::run()
}

pub use error::AffidavitError;
pub use types::{
    canonical_bytes, Blake3Hash, CheckOutcome, ObjectRef, OperationEvent, ProfileId, Receipt,
    Verdict,
};
